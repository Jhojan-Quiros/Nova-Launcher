use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use crate::domain::errors::LauncherError;

pub type LogListener = Arc<dyn Fn(&str, &str, &str) + Send + Sync>; // (instance_id, level, message)

pub struct ProcessSupervisor;

impl ProcessSupervisor {
    pub async fn spawn_and_monitor(
        executable: &Path,
        args: &[String],
        current_dir: &Path,
        instance_id: &str,
        log_listener: Option<LogListener>,
    ) -> Result<i32, LauncherError> {
        let mut cmd = Command::new(executable);
        cmd.args(args);
        cmd.current_dir(current_dir);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        tracing::info!("Launching process: {:?} in {:?}", executable, current_dir);

        let mut child = cmd.spawn().map_err(|e| {
            LauncherError::process(format!("Failed to start Java process {:?}: {}", executable, e))
        })?;

        // Prepare instance game logs folder
        let logs_dir = current_dir.join("logs");
        let _ = fs::create_dir_all(&logs_dir);
        let log_file = logs_dir.join("latest.log");
        let _ = fs::write(&log_file, ""); // clear previous session log

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let inst_id_out = instance_id.to_string();
        let log_file_out = log_file.clone();
        let listener_out = log_listener.clone();

        let stdout_handle = tokio::spawn(async move {
            if let Some(out) = stdout {
                let mut reader = BufReader::new(out).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    let level = if line.contains("ERROR") || line.contains("FATAL") {
                        "ERROR"
                    } else if line.contains("WARN") {
                        "WARN"
                    } else {
                        "INFO"
                    };

                    if let Some(ref l) = listener_out {
                        l(&inst_id_out, level, &line);
                    }

                    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&log_file_out) {
                        let _ = writeln!(f, "[STDOUT] {}", line);
                    }
                }
            }
        });

        let inst_id_err = instance_id.to_string();
        let log_file_err = log_file.clone();
        let listener_err = log_listener.clone();

        let stderr_handle = tokio::spawn(async move {
            if let Some(err) = stderr {
                let mut reader = BufReader::new(err).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    if let Some(ref l) = listener_err {
                        l(&inst_id_err, "ERROR", &line);
                    }

                    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&log_file_err) {
                        let _ = writeln!(f, "[STDERR] {}", line);
                    }
                }
            }
        });

        let status = child.wait().await.map_err(|e| {
            LauncherError::process(format!("Error waiting for process: {}", e))
        })?;

        let _ = stdout_handle.await;
        let _ = stderr_handle.await;

        let exit_code = status.code().unwrap_or(-1);
        tracing::info!("Instance {} process exited with code {}", instance_id, exit_code);

        Ok(exit_code)
    }
}