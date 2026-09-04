use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
}

#[derive(Clone)]
pub struct LogBuffer {
    buffer: Arc<Mutex<Vec<LogEntry>>>,
    max_entries: usize,
}

impl LogBuffer {
    pub fn new(max_entries: usize) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::with_capacity(max_entries))),
            max_entries,
        }
    }

    pub fn push(&self, level: &str, target: &str, message: &str) {
        let entry = LogEntry {
            timestamp: Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
            level: level.to_string(),
            target: target.to_string(),
            message: message.to_string(),
        };

        if let Ok(mut lock) = self.buffer.lock() {
            if lock.len() >= self.max_entries {
                lock.remove(0);
            }
            lock.push(entry);
        }
    }

    pub fn get_logs(&self) -> Vec<LogEntry> {
        self.buffer.lock().map(|l| l.clone()).unwrap_or_default()
    }

    pub fn clear(&self) {
        if let Ok(mut lock) = self.buffer.lock() {
            lock.clear();
        }
    }
}

pub struct LauncherLogger;

impl LauncherLogger {
    pub fn append_to_file(logs_dir: &Path, level: &str, target: &str, message: &str) {
        let _ = fs::create_dir_all(logs_dir);
        let file_path = logs_dir.join("launcher.log");
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(file_path) {
            let now = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let _ = writeln!(f, "[{}] [{}] [{}] {}", now, level, target, message);
        }
    }
}