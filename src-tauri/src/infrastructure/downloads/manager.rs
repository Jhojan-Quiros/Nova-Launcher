use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;
use crate::application::ports::{DownloadItem, DownloadManagerPort};
use crate::domain::entities::download_task::DownloadProgress;
use crate::domain::errors::LauncherError;
use crate::shared::utils::FileVerifier;

pub type ProgressCallback = Arc<dyn Fn(DownloadProgress) + Send + Sync>;

pub struct DownloadManager {
    client: Client,
    max_concurrent: usize,
    progress_callback: Option<ProgressCallback>,
}

impl DownloadManager {
    pub fn new(max_concurrent: usize, progress_callback: Option<ProgressCallback>) -> Self {
        Self {
            client: Client::builder()
                .user_agent("NovaLauncher/1.0")
                .connect_timeout(std::time::Duration::from_secs(10))
                .tcp_keepalive(std::time::Duration::from_secs(15))
                .pool_idle_timeout(std::time::Duration::from_secs(90))
                .build()
                .unwrap_or_default(),
            max_concurrent,
            progress_callback,
        }
    }

    async fn download_single_item(
        client: Client,
        item: DownloadItem,
        instance_id: Option<String>,
        total_bytes: u64,
        downloaded_so_far: Arc<AtomicU64>,
        start_time: Instant,
        progress_cb: Option<ProgressCallback>,
    ) -> Result<(), LauncherError> {
        // Skip if already downloaded and valid
        if let Some(expected_sha1) = &item.sha1 {
            if FileVerifier::verify_sha1(&item.destination, expected_sha1) {
                if let Some(size) = item.size {
                    downloaded_so_far.fetch_add(size, Ordering::Relaxed);
                }
                return Ok(());
            }
        } else if item.destination.exists() {
            return Ok(());
        }

        let parent = item.destination.parent().ok_or_else(|| {
            LauncherError::filesystem(format!("Invalid destination path: {:?}", item.destination))
        })?;
        fs::create_dir_all(parent).map_err(|e| {
            LauncherError::filesystem(format!("Failed to create parent dir {:?}: {}", parent, e))
        })?;

        let tmp_path = item.destination.with_extension("tmp_download");
        let raw_filename = item
            .destination
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let display_name = if raw_filename.len() == 40 && raw_filename.chars().all(|c| c.is_ascii_hexdigit()) {
            format!("Asset {}", &raw_filename[..8])
        } else {
            raw_filename
        };

        // Retry loop
        let mut attempts = 0;
        loop {
            attempts += 1;
            match Self::perform_download(
                &client,
                &item.url,
                &tmp_path,
                &display_name,
                &instance_id,
                total_bytes,
                downloaded_so_far.clone(),
                start_time,
                progress_cb.clone(),
            ).await {
                Ok(()) => break,
                Err(e) if attempts < 3 => {
                    tracing::warn!("Download attempt {} failed for {}: {}. Retrying...", attempts, item.url, e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(500 * attempts as u64)).await;
                }
                Err(e) => {
                    let _ = fs::remove_file(&tmp_path);
                    return Err(e);
                }
            }
        }

        // Verify SHA1 if provided
        if let Some(expected_sha1) = &item.sha1 {
            if !FileVerifier::verify_sha1(&tmp_path, expected_sha1) {
                let _ = fs::remove_file(&tmp_path);
                return Err(LauncherError::download(
                    format!("SHA-1 mismatch for downloaded file {}", item.url),
                    Some(format!("Expected: {}", expected_sha1)),
                ));
            }
        }

        // Atomically rename tmp to destination
        fs::rename(&tmp_path, &item.destination).map_err(|e| {
            LauncherError::filesystem(format!("Failed to finalize downloaded file: {}", e))
        })?;

        Ok(())
    }

    async fn perform_download(
        client: &Client,
        url: &str,
        dest: &Path,
        display_name: &str,
        instance_id: &Option<String>,
        total_bytes: u64,
        downloaded_so_far: Arc<AtomicU64>,
        start_time: Instant,
        progress_cb: Option<ProgressCallback>,
    ) -> Result<(), LauncherError> {
        let response = client.get(url).send().await.map_err(|e| {
            LauncherError::network(format!("Failed to connect to {}: {}", url, e))
        })?;

        if !response.status().is_success() {
            return Err(LauncherError::network(format!("HTTP error {} downloading {}", response.status(), url)));
        }

        let mut file = File::create(dest).map_err(|e| {
            LauncherError::filesystem(format!("Failed to create temporary file {:?}: {}", dest, e))
        })?;

        let mut stream = response.bytes_stream();

        loop {
            let chunk_opt = match tokio::time::timeout(
                std::time::Duration::from_secs(20),
                stream.next(),
            ).await {
                Ok(Some(res)) => res.map_err(|e| LauncherError::network(format!("Stream error reading {}: {}", url, e)))?,
                Ok(None) => break,
                Err(_) => return Err(LauncherError::network(format!("Connection timed out reading chunks for {}", url))),
            };

            file.write_all(&chunk_opt).map_err(|e| {
                LauncherError::filesystem(format!("Failed to write chunk: {}", e))
            })?;

            let prev = downloaded_so_far.fetch_add(chunk_opt.len() as u64, Ordering::Relaxed);
            let current = prev + chunk_opt.len() as u64;

            if let Some(cb) = &progress_cb {
                let elapsed = start_time.elapsed().as_secs_f64();
                let speed = if elapsed > 0.0 { current as f64 / elapsed } else { 0.0 };
                let percentage = if total_bytes > 0 {
                    ((current as f64 / total_bytes as f64) * 100.0).min(100.0) as f32
                } else {
                    0.0
                };

                cb(DownloadProgress {
                    instance_id: instance_id.clone(),
                    file: display_name.to_string(),
                    downloaded_bytes: current,
                    total_bytes,
                    percentage,
                    speed_bytes_per_sec: speed,
                });
            }
        }

        file.flush().map_err(|e| LauncherError::filesystem(format!("Flush error: {}", e)))?;
        Ok(())
    }
}

#[async_trait]
impl DownloadManagerPort for DownloadManager {
    async fn download_batch(
        &self,
        instance_id: Option<String>,
        items: Vec<DownloadItem>,
    ) -> Result<(), LauncherError> {
        if items.is_empty() {
            return Ok(());
        }

        let mut total_bytes: u64 = 0;
        let mut items_to_download = Vec::with_capacity(items.len());

        for item in items {
            // Pre-check SHA-1 to avoid counting already completed files
            if let Some(sha1) = &item.sha1 {
                if FileVerifier::verify_sha1(&item.destination, sha1) {
                    continue;
                }
            } else if item.destination.exists() {
                continue;
            }

            if let Some(s) = item.size {
                total_bytes += s;
            }
            items_to_download.push(item);
        }

        if items_to_download.is_empty() {
            // All items already present and verified
            return Ok(());
        }

        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
        let downloaded_so_far = Arc::new(AtomicU64::new(0));
        let start_time = Instant::now();

        let mut handles = Vec::with_capacity(items_to_download.len());

        for item in items_to_download {
            let permit = semaphore.clone().acquire_owned().await.map_err(|e| {
                LauncherError::internal(format!("Semaphore error: {}", e))
            })?;
            let client = self.client.clone();
            let inst_id = instance_id.clone();
            let dl_counter = downloaded_so_far.clone();
            let cb = self.progress_callback.clone();

            let handle = tokio::spawn(async move {
                let res = Self::download_single_item(
                    client,
                    item,
                    inst_id,
                    total_bytes,
                    dl_counter,
                    start_time,
                    cb,
                ).await;
                drop(permit);
                res
            });

            handles.push(handle);
        }

        for h in handles {
            match h.await {
                Ok(res) => res?,
                Err(join_err) => return Err(LauncherError::internal(format!("Download task panicked: {}", join_err))),
            }
        }

        Ok(())
    }
}