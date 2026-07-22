//! Crash logging + playtester log directory helpers.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// `%LOCALAPPDATA%/PugdyMon/logs` on Windows, else `./logs`.
pub fn log_dir() -> PathBuf {
    if let Ok(base) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(base)
            .join(crate::brand::APP_DATA_DIR)
            .join("logs")
    } else {
        PathBuf::from("logs")
    }
}

pub fn ensure_log_dir() -> PathBuf {
    let dir = log_dir();
    let _ = fs::create_dir_all(&dir);
    dir
}

/// Install a panic hook that appends to `crash.log` under the log dir.
pub fn install_crash_hook() {
    let dir = ensure_log_dir();
    let crash_path = dir.join("crash.log");
    std::panic::set_hook(Box::new(move |info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown".into());
        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Box<Any>".into()
        };
        let line = format!(
            "[{}] PANIC at {location}: {payload}\n",
            chrono_like_timestamp()
        );
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&crash_path)
        {
            let _ = file.write_all(line.as_bytes());
        }
        eprintln!("{line}");
    }));
}

fn chrono_like_timestamp() -> String {
    // Avoid chrono crate — use system time seconds.
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".into())
}
