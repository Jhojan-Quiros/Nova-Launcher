use nova_launcher_lib::domain::modpacks::*;
use nova_launcher_lib::infrastructure::modpacks::{
    ModpackFileVerifier, ModpackUpdatePlanner, SafePathResolver,
};
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_safe_path_resolver_security() {
    let temp = tempdir().unwrap();
    let game_dir = temp.path();

    // 1. Valid paths should resolve cleanly
    let valid_mod = SafePathResolver::resolve_safe_path(game_dir, "mods/jei-1.20.1.jar");
    assert!(valid_mod.is_ok());
    assert!(valid_mod.unwrap().starts_with(game_dir));

    let valid_config = SafePathResolver::resolve_safe_path(game_dir, "config/sub/myconfig.json");
    assert!(valid_config.is_ok());

    // 2. Traversal attempts MUST be blocked
    let traversal_1 = SafePathResolver::resolve_safe_path(game_dir, "../outside.txt");
    assert!(traversal_1.is_err());

    let traversal_2 = SafePathResolver::resolve_safe_path(game_dir, "mods/../../windows/system32/cmd.exe");
    assert!(traversal_2.is_err());

    let traversal_3 = SafePathResolver::resolve_safe_path(game_dir, "/etc/passwd");
    assert!(traversal_3.is_err());

    let traversal_null = SafePathResolver::resolve_safe_path(game_dir, "mods/file\0evil.jar");
    assert!(traversal_null.is_err());


    // 3. Protected user paths MUST be protected from deletion
    assert!(SafePathResolver::is_protected_path("saves/World_1/level.dat"));
    assert!(SafePathResolver::is_protected_path("screenshots/2026-09-04.png"));
    assert!(SafePathResolver::is_protected_path("logs/latest.log"));
    assert!(SafePathResolver::is_protected_path("crash-reports/crash-2026.txt"));
    assert!(SafePathResolver::is_protected_path("options.txt"));
    assert!(SafePathResolver::is_protected_path(".nova/manifest.json"));

    // Normal modpack files must NOT be marked as protected
    assert!(!SafePathResolver::is_protected_path("mods/sodium.jar"));
    assert!(!SafePathResolver::is_protected_path("config/fabric.json"));
    assert!(!SafePathResolver::is_protected_path("resourcepacks/faithful.zip"));
}

#[test]
fn test_manifest_schema_validation_and_semver() {
    // 1. Schema version validation
    assert!(ManifestSchemaValidator::validate_schema_version(1).is_ok());
    assert!(ManifestSchemaValidator::validate_schema_version(0).is_err());
    assert!(ManifestSchemaValidator::validate_schema_version(2).is_err());

    // 2. Semver comparisons
    assert!(ManifestSchemaValidator::is_newer_version("1.0.0", "1.1.0"));
    assert!(ManifestSchemaValidator::is_newer_version("1.9.9", "1.10.0"));
    assert!(ManifestSchemaValidator::is_newer_version("1.0.0-beta.1", "1.0.0"));
    assert!(!ManifestSchemaValidator::is_newer_version("2.0.0", "1.9.9"));
    assert!(!ManifestSchemaValidator::is_newer_version("1.5.0", "1.5.0"));
}

#[test]
fn test_modpack_file_verifier_sha256() {
    let temp = tempdir().unwrap();
    let file_path = temp.path().join("test_file.txt");
    let content = b"Hello Nova Minecraft Launcher Modpack Verifier!";

    {
        let mut file = File::create(&file_path).unwrap();
        file.write_all(content).unwrap();
    }

    let computed = ModpackFileVerifier::compute_sha256(&file_path).expect("hash computation failed");

    // Standard SHA-256 for this exact byte slice
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content);
    let expected = format!("{:x}", hasher.finalize());

    assert_eq!(computed, expected);
    assert!(ModpackFileVerifier::verify_file_sha256(&file_path, &expected));
    assert!(!ModpackFileVerifier::verify_file_sha256(&file_path, "0000000000000000000000000000000000000000000000000000000000000000"));
}

#[test]
fn test_modpack_update_planner_differential() {
    let temp = tempdir().unwrap();
    let game_dir = temp.path();

    let mods_dir = game_dir.join("mods");
    fs::create_dir_all(&mods_dir).unwrap();

    // 1. Existing mod A with known hash
    let mod_a_path = mods_dir.join("mod-a.jar");
    fs::write(&mod_a_path, b"Mod A Content").unwrap();
    let mod_a_hash = ModpackFileVerifier::compute_sha256(&mod_a_path).unwrap();

    // 2. Existing mod B that changed in target version
    let mod_b_path = mods_dir.join("mod-b.jar");
    fs::write(&mod_b_path, b"Old Mod B Content").unwrap();

    // 3. User mod C added manually
    let user_mod_path = mods_dir.join("user-custom-mod.jar");
    fs::write(&user_mod_path, b"User custom mod").unwrap();

    // Target version manifest: has mod A (same hash), mod B (new hash), and mod D (new file)
    let target = ModpackVersionManifest {
        schema_version: 1,
        pack_id: "test-pack".to_string(),
        version: "2.0.0".to_string(),
        minecraft_version: "1.20.1".to_string(),
        loader: "fabric".to_string(),
        loader_version: Some("0.15.11".to_string()),
        release_date: None,
        changelog: vec!["Updated Mod B".to_string(), "Added Mod D".to_string()],
        stats: None,
        files: vec![
            ManifestFileEntry {
                path: "mods/mod-a.jar".to_string(),
                file_name: "mod-a.jar".to_string(),
                category: "mods".to_string(),
                size: 13,
                sha256: mod_a_hash,
                url: "https://example.com/mod-a.jar".to_string(),
                mod_id: None,
            },
            ManifestFileEntry {
                path: "mods/mod-b.jar".to_string(),
                file_name: "mod-b.jar".to_string(),
                category: "mods".to_string(),
                size: 18,
                sha256: "new_sha256_for_mod_b".to_string(),
                url: "https://example.com/mod-b-v2.jar".to_string(),
                mod_id: None,
            },
            ManifestFileEntry {
                path: "mods/mod-d.jar".to_string(),
                file_name: "mod-d.jar".to_string(),
                category: "mods".to_string(),
                size: 25,
                sha256: "sha256_for_mod_d".to_string(),
                url: "https://example.com/mod-d.jar".to_string(),
                mod_id: None,
            },
        ],
        removed_files: vec![],
    };

    // Installed manifest had mod A and mod B
    let _installed_manifest = ModpackVersionManifest {

        schema_version: 1,
        pack_id: "test-pack".to_string(),
        version: "1.0.0".to_string(),
        minecraft_version: "1.20.1".to_string(),
        loader: "fabric".to_string(),
        loader_version: Some("0.15.11".to_string()),
        release_date: None,
        changelog: vec![],
        stats: None,
        files: vec![
            ManifestFileEntry {
                path: "mods/mod-a.jar".to_string(),
                file_name: "mod-a.jar".to_string(),
                category: "mods".to_string(),
                size: 13,
                sha256: "mod_a_hash".to_string(),
                url: "".to_string(),
                mod_id: None,
            },
            ManifestFileEntry {
                path: "mods/mod-b.jar".to_string(),
                file_name: "mod-b.jar".to_string(),
                category: "mods".to_string(),
                size: 18,
                sha256: "old_mod_b_hash".to_string(),
                url: "".to_string(),
                mod_id: None,
            },
        ],
        removed_files: vec![],
    };

    // Non-strict mode: user-custom-mod.jar should NOT be deleted
    let plan = ModpackUpdatePlanner::plan_update(
        game_dir,
        Some("1.0.0"),
        &target,
        false, // non-strict
    ).unwrap();


    assert_eq!(plan.from_version, Some("1.0.0".to_string()));
    assert_eq!(plan.to_version, "2.0.0".to_string());
    // mod-a has same hash -> unmodified
    assert_eq!(plan.unmodified_count, 1);
    // mod-b and mod-d need download
    assert_eq!(plan.downloads.len(), 2);
    assert!(plan.downloads.iter().any(|f| f.path == "mods/mod-b.jar"));
    assert!(plan.downloads.iter().any(|f| f.path == "mods/mod-d.jar"));
    // user mod C is preserved
    assert!(!plan.deletions.contains(&"mods/user-custom-mod.jar".to_string()));
}

#[test]
fn test_cloudflare_r2_manifest_compatibility_and_normalization() {
    // 1. Exact catalog JSON representation from Cloudflare R2
    let r2_catalog_json = r#"{
      "schemaVersion": 1,
      "updatedAt": "2026-09-05T00:13:33.997Z",
      "modpacks": [
        {
          "id": "cmtnlqmdy006rquim5ze2ij32",
          "slug": "survival",
          "name": "Wombat-land",
          "description": null,
          "minecraftVersion": "1.20.1",
          "loader": "forge",
          "loaderVersion": "47.4.0",
          "latestVersion": "1.0.0",
          "iconUrl": null,
          "bannerUrl": null,
          "manifestUrl": "https://pub-afdecd4c468d49edbd6b713db9fa3e7f.r2.dev/modpacks/survival/manifest.json"
        }
      ]
    }"#;

    let catalog: ModpackCatalog = serde_json::from_str(r2_catalog_json).expect("Failed to deserialize R2 catalog");
    assert_eq!(catalog.schema_version, 1);
    assert_eq!(catalog.generated_at, Some("2026-09-05T00:13:33.997Z".to_string()));
    assert_eq!(catalog.modpacks.len(), 1);
    let pack = &catalog.modpacks[0];
    assert_eq!(pack.name, "Wombat-land");
    assert_eq!(pack.description, None);
    assert_eq!(pack.icon_url, None);
    assert_eq!(pack.loader_version, Some("47.4.0".to_string()));

    // 2. Exact version manifest JSON representation with flat paths, null modId, and unchanged stats
    let r2_version_json = r#"{
      "schemaVersion": 1,
      "packId": "survival",
      "version": "1.0.0",
      "minecraftVersion": "1.20.1",
      "loader": "forge",
      "loaderVersion": "47.4.0",
      "releaseDate": "2026-09-04T23:46:20.169Z",
      "changelog": ["Added: AI-Improvements-1.20-0.5.2.jar", "Added: Detail-Brush-1.6-1.20.zip"],
      "stats": {
        "totalFiles": 2,
        "added": 2,
        "updated": 0,
        "removed": 0,
        "unchanged": 0,
        "totalSize": 105333,
        "downloadSize": 105333
      },
      "files": [
        {
          "path": "AI-Improvements-1.20-0.5.2.jar",
          "fileName": "AI-Improvements-1.20-0.5.2.jar",
          "category": "other",
          "size": 29553,
          "sha256": "575a4a8e00f982c064d54bb3e73e5cda2d14414de054a471a3daf3f58bdf560a",
          "url": "https://pub-afdecd4c468d49edbd6b713db9fa3e7f.r2.dev/modpacks/survival/files/AI-Improvements-1.20-0.5.2.jar",
          "modId": null
        },
        {
          "path": "Detail-Brush-1.6-1.20.zip",
          "fileName": "Detail-Brush-1.6-1.20.zip",
          "category": "other",
          "size": 75780,
          "sha256": "abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd",
          "url": "https://pub-afdecd4c468d49edbd6b713db9fa3e7f.r2.dev/modpacks/survival/files/Detail-Brush-1.6-1.20.zip",
          "modId": null
        }
      ]
    }"#;

    let mut ver_manifest: ModpackVersionManifest = serde_json::from_str(r2_version_json).expect("Failed to deserialize R2 version manifest");
    ver_manifest.normalize_paths();

    // Verify paths normalized into mods/ and resourcepacks/
    assert_eq!(ver_manifest.files[0].path, "mods/AI-Improvements-1.20-0.5.2.jar");
    assert_eq!(ver_manifest.files[1].path, "resourcepacks/Detail-Brush-1.6-1.20.zip");

    // Verify SafePathResolver places them in respective subdirectories
    let temp = tempdir().unwrap();
    let game_dir = temp.path();
    let mod_resolved = SafePathResolver::resolve_safe_path(game_dir, "AI-Improvements-1.20-0.5.2.jar").unwrap();
    assert_eq!(mod_resolved, game_dir.join("mods").join("AI-Improvements-1.20-0.5.2.jar"));

    let rp_resolved = SafePathResolver::resolve_safe_path(game_dir, "Detail-Brush-1.6-1.20.zip").unwrap();
    assert_eq!(rp_resolved, game_dir.join("resourcepacks").join("Detail-Brush-1.6-1.20.zip"));
}

#[tokio::test]
async fn test_live_r2_catalog_fetch() {
    use nova_launcher_lib::domain::modpacks::RemoteModpackRepository;
    let provider = nova_launcher_lib::infrastructure::modpacks::HttpModpackProvider::new();
    let catalog = provider.get_catalog("https://pub-afdecd4c468d49edbd6b713db9fa3e7f.r2.dev/modpacks/catalog.json", true).await;
    assert!(catalog.is_ok(), "Failed to fetch live catalog: {:?}", catalog.err());
    let cat = catalog.unwrap();
    assert_eq!(cat.modpacks.len(), 1);
    assert_eq!(cat.modpacks[0].name, "Wombat-land");
    assert_eq!(cat.modpacks[0].latest_version, "1.0.0");
}

