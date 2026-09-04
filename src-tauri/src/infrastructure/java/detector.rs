use async_trait::async_trait;
use regex::Regex;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;
use crate::application::ports::JavaDetectorPort;
use crate::domain::entities::JavaRuntime;
use crate::domain::errors::LauncherError;

pub struct JavaVersionParser;

impl JavaVersionParser {
    pub fn parse_version(output: &str) -> Option<(u32, String)> {
        // Regex handles:
        // java version "25.0.2"
        // openjdk version "21.0.2"
        // java version "1.8.0_391"
        let re = Regex::new(r#"(?:java|openjdk)\s+version\s+"([^"]+)""#).ok()?;
        let captures = re.captures(output)?;
        let raw_version = captures.get(1)?.as_str().to_string();

        let major_version = if raw_version.starts_with("1.") {
            // Legacy Java 1.8 -> 8
            raw_version.chars().nth(2)?.to_digit(10)?
        } else {
            // Modern Java 17.0.2, 21.0.1, 25.0.2 -> prefix before dot
            raw_version.split('.').next()?.parse::<u32>().ok()?
        };

        Some((major_version, raw_version))
    }
}

pub struct JavaDetector;

impl JavaDetector {
    pub fn new() -> Self {
        Self
    }

    fn find_candidates() -> Vec<PathBuf> {
        let mut candidates = HashSet::new();

        // 1. JAVA_HOME
        if let Ok(java_home) = std::env::var("JAVA_HOME") {
            let p = PathBuf::from(java_home).join("bin").join("java.exe");
            if p.exists() {
                candidates.insert(p);
            }
        }

        // 2. PATH
        if let Ok(path_var) = std::env::var("PATH") {
            for part in std::env::split_paths(&path_var) {
                let p = part.join("java.exe");
                if p.exists() {
                    candidates.insert(p);
                }
            }
        }

        // 3. Common Windows Program Files locations
        let program_files = [
            r"C:\Program Files\Java",
            r"C:\Program Files\Eclipse Adoptium",
            r"C:\Program Files\Microsoft",
            r"C:\Program Files\BellSoft",
            r"C:\Program Files\Zulu",
            r"C:\Program Files (x86)\Java",
        ];

        for base in program_files {
            let base_path = Path::new(base);
            if base_path.exists() {
                if let Ok(entries) = std::fs::read_dir(base_path) {
                    for entry in entries.flatten() {
                        let java_exe = entry.path().join("bin").join("java.exe");
                        if java_exe.exists() {
                            candidates.insert(java_exe);
                        }
                    }
                }
            }
        }

        candidates.into_iter().collect()
    }
}

#[async_trait]
impl JavaDetectorPort for JavaDetector {
    async fn detect_installed_runtimes(&self) -> Result<Vec<JavaRuntime>, LauncherError> {
        let candidates = Self::find_candidates();
        let mut runtimes = Vec::new();

        for candidate in candidates {
            let path_str = candidate.to_string_lossy().to_string();
            if let Ok(runtime) = self.probe_executable(&path_str).await {
                runtimes.push(runtime);
            }
        }

        // Sort by major version descending (prefer newest by default)
        runtimes.sort_by(|a, b| b.major_version.cmp(&a.major_version));
        Ok(runtimes)
    }

    async fn probe_executable(&self, path: &str) -> Result<JavaRuntime, LauncherError> {
        let path_buf = PathBuf::from(path);
        if !path_buf.exists() {
            return Err(LauncherError::java(format!("Java executable not found at {}", path)));
        }

        let output = Command::new(&path_buf)
            .arg("-version")
            .output()
            .map_err(|e| LauncherError::java(format!("Failed to execute {:?}: {}", path_buf, e)))?;

        // java -version writes to stderr!
        let output_text = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let (major_version, raw_version) = JavaVersionParser::parse_version(&output_text)
            .ok_or_else(|| LauncherError::java(format!("Could not parse Java version output: {}", output_text)))?;

        let name = format!("Java {} ({})", major_version, raw_version);
        let id = format!("java-{}-{}", major_version, &Uuid::new_v4().to_string()[..6]);

        Ok(JavaRuntime {
            id,
            name,
            path: path.to_string(),
            major_version,
            raw_version,
            is_valid: true,
        })
    }
}