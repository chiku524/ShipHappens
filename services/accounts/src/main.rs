//! PudgyMon accounts API — email/password auth + profile.

use std::{env, net::SocketAddr, time::Duration};

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::State,
    http::{header, HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{Duration as ChronoDuration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    jwt_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    email: String,
    exp: i64,
}

#[derive(Debug, FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    password_hash: String,
    display_name: String,
    boing_wallet: Option<String>,
    created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize, Clone)]
struct Profile {
    id: Uuid,
    email: String,
    display_name: String,
    boing_wallet: Option<String>,
    created_at: chrono::DateTime<Utc>,
}

impl From<UserRow> for Profile {
    fn from(u: UserRow) -> Self {
        Self {
            id: u.id,
            email: u.email,
            display_name: u.display_name,
            boing_wallet: u.boing_wallet,
            created_at: u.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
struct SignupRequest {
    email: String,
    password: String,
    display_name: String,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct PatchMeRequest {
    display_name: Option<String>,
    boing_wallet: Option<String>,
}

#[derive(Debug, Serialize)]
struct AuthResponse {
    access_token: String,
    profile: Profile,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://pudgymon:pudgymon@127.0.0.1:5434/pudgymon_accounts".into()
    });
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "dev-only-change-me".into());
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8787);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await?;

    run_migrations(&pool).await?;

    let state = AppState {
        pool,
        jwt_secret,
    };

    // Open CORS for local web + Vercel static site talking to this API.
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/v1/auth/signup", post(signup))
        .route("/v1/auth/login", post(login))
        .route("/v1/me", get(me).patch(patch_me))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("pudgymon-accounts listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    // sqlx prepared statements allow one command each.
    sqlx::query(r#"CREATE EXTENSION IF NOT EXISTS "pgcrypto""#)
        .execute(pool)
        .await?;
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            display_name TEXT NOT NULL,
            boing_wallet TEXT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;
    sqlx::query(r#"CREATE INDEX IF NOT EXISTS users_email_idx ON users (email)"#)
        .execute(pool)
        .await?;
    Ok(())
}

async fn signup(
    State(state): State<AppState>,
    Json(body): Json<SignupRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let email = normalize_email(&body.email)?;
    let display_name = body.display_name.trim().to_string();
    if display_name.is_empty() || display_name.len() > 48 {
        return Err(ApiError::bad("display_name must be 1–48 characters"));
    }
    if body.password.len() < 8 {
        return Err(ApiError::bad("password must be at least 8 characters"));
    }

    let hash = hash_password(&body.password)?;
    let row = sqlx::query_as::<_, UserRow>(
        r#"
        INSERT INTO users (email, password_hash, display_name)
        VALUES ($1, $2, $3)
        RETURNING id, email, password_hash, display_name, boing_wallet, created_at
        "#,
    )
    .bind(&email)
    .bind(&hash)
    .bind(&display_name)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(db) = &e {
            if db.constraint() == Some("users_email_key") {
                return ApiError::conflict("email already registered");
            }
        }
        ApiError::internal(e.to_string())
    })?;

    let profile = Profile::from(row);
    let token = issue_token(&state.jwt_secret, &profile)?;
    Ok(Json(AuthResponse {
        access_token: token,
        profile,
    }))
}

async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let email = normalize_email(&body.email)?;
    let row = sqlx::query_as::<_, UserRow>(
        r#"
        SELECT id, email, password_hash, display_name, boing_wallet, created_at
        FROM users WHERE email = $1
        "#,
    )
    .bind(&email)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::internal(e.to_string()))?
    .ok_or_else(|| ApiError::unauthorized("invalid email or password"))?;

    verify_password(&body.password, &row.password_hash)?;
    let profile = Profile::from(row);
    let token = issue_token(&state.jwt_secret, &profile)?;
    Ok(Json(AuthResponse {
        access_token: token,
        profile,
    }))
}

async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Profile>, ApiError> {
    let user_id = auth_user_id(&state, &headers)?;
    let row = sqlx::query_as::<_, UserRow>(
        r#"
        SELECT id, email, password_hash, display_name, boing_wallet, created_at
        FROM users WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::internal(e.to_string()))?
    .ok_or_else(|| ApiError::unauthorized("user not found"))?;
    Ok(Json(Profile::from(row)))
}

async fn patch_me(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PatchMeRequest>,
) -> Result<Json<Profile>, ApiError> {
    let user_id = auth_user_id(&state, &headers)?;

    if let Some(name) = body.display_name.as_ref() {
        let name = name.trim();
        if name.is_empty() || name.len() > 48 {
            return Err(ApiError::bad("display_name must be 1–48 characters"));
        }
        sqlx::query("UPDATE users SET display_name = $1 WHERE id = $2")
            .bind(name)
            .bind(user_id)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::internal(e.to_string()))?;
    }

    if let Some(wallet) = body.boing_wallet.as_ref() {
        let wallet = wallet.trim();
        let value = if wallet.is_empty() {
            None
        } else if wallet.starts_with("0x") && wallet.len() >= 42 {
            Some(wallet.to_string())
        } else {
            return Err(ApiError::bad("boing_wallet must be 0x… address"));
        };
        sqlx::query("UPDATE users SET boing_wallet = $1 WHERE id = $2")
            .bind(value)
            .bind(user_id)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::internal(e.to_string()))?;
    }

    let row = sqlx::query_as::<_, UserRow>(
        r#"
        SELECT id, email, password_hash, display_name, boing_wallet, created_at
        FROM users WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(Profile::from(row)))
}

fn normalize_email(email: &str) -> Result<String, ApiError> {
    let email = email.trim().to_lowercase();
    if !email.contains('@') || email.len() < 5 || email.len() > 254 {
        return Err(ApiError::bad("invalid email"));
    }
    Ok(email)
}

fn hash_password(password: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| ApiError::internal(e.to_string()))
}

fn verify_password(password: &str, hash: &str) -> Result<(), ApiError> {
    let parsed = PasswordHash::new(hash).map_err(|_| ApiError::unauthorized("invalid email or password"))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| ApiError::unauthorized("invalid email or password"))
}

fn issue_token(secret: &str, profile: &Profile) -> Result<String, ApiError> {
    let exp = (Utc::now() + ChronoDuration::days(7)).timestamp();
    let claims = Claims {
        sub: profile.id.to_string(),
        email: profile.email.clone(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| ApiError::internal(e.to_string()))
}

fn auth_user_id(state: &AppState, headers: &HeaderMap) -> Result<Uuid, ApiError> {
    let auth = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::unauthorized("missing Authorization"))?;
    let token = auth
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::unauthorized("expected Bearer token"))?;
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| ApiError::unauthorized("invalid or expired token"))?;
    Uuid::parse_str(&data.claims.sub).map_err(|_| ApiError::unauthorized("invalid token subject"))
}

struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: msg.into(),
        }
    }
    fn unauthorized(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: msg.into(),
        }
    }
    fn conflict(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            message: msg.into(),
        }
    }
    fn internal(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: msg.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorBody {
                error: self.message,
            }),
        )
            .into_response()
    }
}
