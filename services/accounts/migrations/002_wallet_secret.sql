-- Custodial Boing wallet secret (encrypted at rest by the API).
ALTER TABLE users
ADD COLUMN IF NOT EXISTS boing_wallet_secret_enc TEXT NULL;
