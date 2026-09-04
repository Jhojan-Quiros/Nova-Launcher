use std::sync::Arc;
use nova_launcher_lib::domain::entities::OfflineProfile;
use nova_launcher_lib::domain::repositories::OfflineProfileRepository;
use nova_launcher_lib::infrastructure::auth::OfflineAuthenticationProvider;
use nova_launcher_lib::application::ports::AuthenticationProviderPort;
use nova_launcher_lib::infrastructure::persistence::{DatabaseManager, SqliteOfflineProfileRepository};

#[tokio::test]
async fn test_deterministic_offline_uuid() {
    let uuid_steve_1 = OfflineProfile::generate_deterministic_uuid("Steve");
    let uuid_steve_2 = OfflineProfile::generate_deterministic_uuid("Steve");
    let uuid_steve_trimmed = OfflineProfile::generate_deterministic_uuid("  Steve  ");

    // 1. Must be deterministic
    assert_eq!(uuid_steve_1, uuid_steve_2);
    assert_eq!(uuid_steve_1, uuid_steve_trimmed);

    // 2. Must differ for different usernames
    let uuid_alex = OfflineProfile::generate_deterministic_uuid("Alex");
    assert_ne!(uuid_steve_1, uuid_alex);

    // 3. Must be valid 36-char hyphenated UUID string
    assert_eq!(uuid_steve_1.len(), 36);
    assert!(uuid::Uuid::parse_str(&uuid_steve_1).is_ok());

    // 4. Verify version is 3 (RFC 4122 MD5-based UUID, 13th character is '3')
    let parsed = uuid::Uuid::parse_str(&uuid_steve_1).unwrap();
    assert_eq!(parsed.get_version_num(), 3);
}

#[tokio::test]
async fn test_offline_profile_lifecycle_and_sqlite_persistence() {
    let pool = DatabaseManager::init_in_memory().expect("In-memory SQLite should initialize");
    let repo = Arc::new(SqliteOfflineProfileRepository::new(pool));

    // 1. First get_active should auto-seed default NovaPlayer
    let initial = repo.get_active().await.expect("Should return active profile");
    assert_eq!(initial.username, "NovaPlayer");
    assert_eq!(initial.generated_local_uuid, OfflineProfile::generate_deterministic_uuid("NovaPlayer"));
    assert!(!initial.created_at.is_empty());
    assert!(!initial.last_used_at.is_empty());

    // 2. Create a new offline profile
    let custom_profile = OfflineProfile::new("MasterCrafter");
    assert_eq!(custom_profile.username, "MasterCrafter");
    assert_eq!(
        custom_profile.generated_local_uuid,
        OfflineProfile::generate_deterministic_uuid("MasterCrafter")
    );

    repo.save(&custom_profile, true).await.expect("Should save custom profile");

    // 3. Newly saved profile was set active
    let active = repo.get_active().await.expect("Should get active");
    assert_eq!(active.username, "MasterCrafter");

    // 4. List profiles
    let all = repo.list().await.expect("Should list profiles");
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].username, "MasterCrafter"); // active is first
    assert_eq!(all[1].username, "NovaPlayer");

    // 5. Switch active back to NovaPlayer
    let switched = repo.set_active("NovaPlayer").await.expect("Should switch to NovaPlayer");
    assert_eq!(switched.username, "NovaPlayer");

    let current_active = repo.get_active().await.expect("Should get active");
    assert_eq!(current_active.username, "NovaPlayer");

    // 6. Test OfflineAuthenticationProvider port integration
    let auth_provider = OfflineAuthenticationProvider::new(repo.clone());
    let account = auth_provider.get_active_account().await.expect("Should get active account");
    assert_eq!(account.username, "NovaPlayer");
    assert_eq!(account.uuid, OfflineProfile::generate_deterministic_uuid("NovaPlayer"));
    assert_eq!(account.access_token, "0");

    // 7. Delete custom profile
    repo.delete("MasterCrafter").await.expect("Should delete profile");
    let remaining = repo.list().await.expect("Should list profiles");
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].username, "NovaPlayer");

    // 8. Deleting the last remaining profile must fail
    let err = repo.delete("NovaPlayer").await;
    assert!(err.is_err());
}
