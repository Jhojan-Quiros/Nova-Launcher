use std::sync::Arc;
use tempfile::tempdir;
use nova_launcher_lib::application::dto::{CreateInstanceDto, UpdateInstanceDto};
use nova_launcher_lib::application::use_cases::{CreateInstanceUseCase, DeleteInstanceUseCase, GetInstanceUseCase, ListInstancesUseCase, UpdateInstanceUseCase};
use nova_launcher_lib::domain::entities::ModLoader;
use nova_launcher_lib::domain::repositories::{InstanceRepository, SettingsRepository};
use nova_launcher_lib::infrastructure::persistence::{DatabaseManager, SqliteInstanceRepository, SqliteSettingsRepository};
use nova_launcher_lib::shared::config::LauncherPaths;

#[tokio::test]
async fn test_instance_lifecycle_and_isolation() {
    let temp = tempdir().unwrap();
    let paths = LauncherPaths::new(temp.path().join("launcher-data"));
    paths.ensure_directories().unwrap();

    let pool = DatabaseManager::init_in_memory().unwrap();
    let instance_repo: Arc<dyn InstanceRepository> = Arc::new(SqliteInstanceRepository::new(pool.clone()));
    let settings_repo: Arc<dyn SettingsRepository> = Arc::new(SqliteSettingsRepository::new(pool.clone()));

    let create_uc = CreateInstanceUseCase::new(instance_repo.clone(), settings_repo.clone(), paths.clone());
    let list_uc = ListInstancesUseCase::new(instance_repo.clone());
    let get_uc = GetInstanceUseCase::new(instance_repo.clone());
    let update_uc = UpdateInstanceUseCase::new(instance_repo.clone(), paths.clone());
    let delete_uc = DeleteInstanceUseCase::new(instance_repo.clone(), paths.clone());

    // 1. Create instance
    let dto = CreateInstanceDto {
        name: "Survival World".into(),
        minecraft_version: "1.21.1".into(),
        loader: ModLoader::Vanilla,
        loader_version: None,
        min_ram: Some(2048),
        max_ram: Some(4096),
    };

    let created = create_uc.execute(dto).await.expect("Failed to create instance");
    assert_eq!(created.name, "Survival World");
    assert_eq!(created.minecraft_version, "1.21.1");
    assert_eq!(created.loader, ModLoader::Vanilla);

    // Verify isolation folder exists
    let game_dir = paths.instance_game_dir(&created.id);
    assert!(game_dir.exists());
    assert!(game_dir.join("saves").exists());
    assert!(game_dir.join("mods").exists());
    assert!(paths.instance_dir(&created.id).join("instance.json").exists());

    // 2. List instances
    let list = list_uc.execute().await.expect("Failed to list instances");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, created.id);

    // 3. Get by id
    let fetched = get_uc.execute(&created.id).await.expect("Failed to get instance");
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.ram.max_mb, 4096);

    // 4. Update instance
    let update_dto = UpdateInstanceDto {
        name: Some("Survival Pro".into()),
        java_path: Some("C:\\Program Files\\Java\\jdk-21\\bin\\java.exe".into()),
        min_ram: Some(3072),
        max_ram: Some(6144),
    };
    let updated = update_uc.execute(&created.id, update_dto).await.expect("Failed to update instance");
    assert_eq!(updated.name, "Survival Pro");
    assert_eq!(updated.ram.max_mb, 6144);
    assert_eq!(updated.java_path, Some("C:\\Program Files\\Java\\jdk-21\\bin\\java.exe".into()));

    // 5. Delete instance
    let deleted = delete_uc.execute(&created.id).await.expect("Failed to delete instance");
    assert!(deleted);
    let list_after = list_uc.execute().await.expect("Failed to list after delete");
    assert_eq!(list_after.len(), 0);
    assert!(!paths.instance_dir(&created.id).exists());
}