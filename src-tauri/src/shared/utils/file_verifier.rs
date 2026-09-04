use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub struct FileVerifier;

impl FileVerifier {
    pub fn verify_sha1(path: &Path, expected_sha1: &str) -> bool {
        if !path.exists() {
            return false;
        }

        let file = match File::open(path) {
            Ok(f) => f,
            Err(_) => return false,
        };

        let mut reader = BufReader::new(file);
        let mut hasher = Sha1::new();
        let mut buffer = [0u8; 64 * 1024];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => hasher.update(&buffer[..n]),
                Err(_) => return false,
            }
        }

        let result = hasher.finalize();
        let calculated = format!("{:x}", result);
        calculated.eq_ignore_ascii_case(expected_sha1)
    }

    pub fn compute_sha1(path: &Path) -> Option<String> {
        let file = File::open(path).ok()?;
        let mut reader = BufReader::new(file);
        let mut hasher = Sha1::new();
        let mut buffer = [0u8; 64 * 1024];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => hasher.update(&buffer[..n]),
                Err(_) => return None,
            }
        }

        Some(format!("{:x}", hasher.finalize()))
    }
}