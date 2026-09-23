use rusqlite::Connection;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::domain::errors::LauncherError;

pub type DbConn = Arc<Mutex<Connection>>;

pub struct DatabaseManager;

impl DatabaseManager {
    pub fn init(db_path: &Path) -> Result<DbConn, LauncherError> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                LauncherError::filesystem(format!("Failed to create db dir: {}", e))
            })?;
        }

        let conn = Connection::open(db_path).map_err(|e| {
            LauncherError::database(format!("Failed to open SQLite database at {:?}: {}", db_path, e))
        })?;

        // Enable WAL mode and foreign keys for performance and reliability
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;"
        ).map_err(|e| LauncherError::database(format!("Failed to configure SQLite pragmas: {}", e)))?;

        Self::run_migrations(&conn)?;

        Ok(Arc::new(Mutex::new(conn)))
    }

    pub fn init_in_memory() -> Result<DbConn, LauncherError> {
        let conn = Connection::open_in_memory().map_err(|e| {
            LauncherError::database(format!("Failed to open in-memory SQLite database: {}", e))
        })?;

        Self::run_migrations(&conn)?;
        Ok(Arc::new(Mutex::new(conn)))
    }

    fn run_migrations(conn: &Connection) -> Result<(), LauncherError> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS instances (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                minecraft_version TEXT NOT NULL,
                loader TEXT NOT NULL,
                loader_version TEXT,
                game_directory TEXT NOT NULL,
                java_path TEXT,
                java_version INTEGER,
                min_ram INTEGER NOT NULL,
                max_ram INTEGER NOT NULL,
                icon TEXT,
                status TEXT NOT NULL,
                status_details TEXT,
                total_play_time_seconds INTEGER NOT NULL DEFAULT 0,
                last_played_at TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS java_runtimes (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT NOT NULL UNIQUE,
                major_version INTEGER NOT NULL,
                raw_version TEXT NOT NULL,
                is_valid INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS offline_profiles (
                username TEXT PRIMARY KEY,
                generated_local_uuid TEXT NOT NULL,
                created_at TEXT NOT NULL,
                last_used_at TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS microsoft_account (
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL,
                uuid TEXT NOT NULL,
                minecraft_access_token TEXT NOT NULL,
                minecraft_token_expires_at TEXT NOT NULL,
                ms_refresh_token TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS installed_modpacks (
                instance_id TEXT PRIMARY KEY,
                modpack_id TEXT NOT NULL,
                name TEXT NOT NULL,
                installed_version TEXT NOT NULL,
                latest_known_version TEXT,
                manifest_url TEXT NOT NULL,
                icon_url TEXT,
                banner_url TEXT,
                installed_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                last_verified_at TEXT,
                status TEXT NOT NULL,
                strict_mode INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY(instance_id) REFERENCES instances(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS modpack_versions (
                pack_id TEXT NOT NULL,
                version TEXT NOT NULL,
                minecraft_version TEXT NOT NULL,
                loader TEXT NOT NULL,
                loader_version TEXT,
                changelog_json TEXT NOT NULL DEFAULT '[]',
                stats_json TEXT NOT NULL DEFAULT '{}',
                manifest_json TEXT NOT NULL,
                cached_at TEXT NOT NULL,
                PRIMARY KEY (pack_id, version)
            );

            CREATE TABLE IF NOT EXISTS modpack_update_history (
                id TEXT PRIMARY KEY,
                instance_id TEXT NOT NULL,
                pack_id TEXT NOT NULL,
                from_version TEXT NOT NULL,
                to_version TEXT NOT NULL,
                status TEXT NOT NULL,
                download_size INTEGER NOT NULL DEFAULT 0,
                started_at TEXT NOT NULL,
                completed_at TEXT,
                error_code TEXT,
                FOREIGN KEY(instance_id) REFERENCES instances(id) ON DELETE CASCADE
            );"
        ).map_err(|e| LauncherError::database(format!("Failed to run database migrations: {}", e)))?;

        Ok(())
    }
}