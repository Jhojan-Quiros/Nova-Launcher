use serde_json::json;
use std::collections::HashMap;
use tempfile::tempdir;
use nova_launcher_lib::domain::entities::{Instance, MinecraftAccount, ModLoader, AccountType};
use nova_launcher_lib::domain::value_objects::ram_config::RamConfig;
use nova_launcher_lib::infrastructure::minecraft::argument_builder::ArgumentBuilder;
use nova_launcher_lib::infrastructure::minecraft::rule_evaluator::PlatformEnvironment;
use nova_launcher_lib::shared::config::LauncherPaths;

#[test]
fn test_substitute_placeholders() {
    let mut map = HashMap::new();
    map.insert("auth_player_name", "Steve".to_string());
    map.insert("version_name", "1.21.1".to_string());

    let template = "--username ${auth_player_name} --version ${version_name}";
    let res = ArgumentBuilder::substitute_placeholders(template, &map);
    assert_eq!(res, "--username Steve --version 1.21.1");
}

#[test]
fn test_build_modern_arguments() {
    let temp = tempdir().unwrap();
    let paths = LauncherPaths::new(temp.path().join("launcher-data"));
    let version_dir = paths.versions_dir().join("1.21.1");
    std::fs::create_dir_all(&version_dir).unwrap();
    std::fs::write(version_dir.join("1.21.1.jar"), b"dummy jar").unwrap();
    let mut env = PlatformEnvironment::current();
    env.os_name = "windows".to_string();
    env.arch = "x86_64".to_string();
    env.is_demo_user = false;
    env.has_custom_resolution = false;

    let instance = Instance::new(
        "inst-123".into(),
        "Test Instance".into(),
        "1.21.1".into(),
        ModLoader::Vanilla,
        None,
        temp.path().join("game").to_string_lossy().to_string(),
        RamConfig::new(2048, 4096).unwrap(),
    );

    let account = MinecraftAccount {
        id: "acc-1".into(),
        username: "Alex".into(),
        uuid: "00000000-0000-0000-0000-000000000000".into(),
        access_token: "mock-token".into(),
        account_type: AccountType::Offline,
    };

    let version_json = json!({
        "id": "1.21.1",
        "mainClass": "net.minecraft.client.main.Main",
        "arguments": {
            "game": [
                "--username", "${auth_player_name}",
                "--version", "${version_name}",
                "--gameDir", "${game_directory}"
            ],
            "jvm": [
                "-Djava.library.path=${natives_directory}",
                "-Dminecraft.launcher.brand=${launcher_name}",
                "-cp", "${classpath}"
            ]
        },
        "libraries": []
    });

    let args = ArgumentBuilder::build(&version_json, &instance, &paths, &account, &env).unwrap();

    assert_eq!(args.main_class, "net.minecraft.client.main.Main");
    assert!(args.jvm_args.contains(&"-Xms2048M".to_string()));
    assert!(args.jvm_args.contains(&"-Xmx4096M".to_string()));
    assert!(args.jvm_args.iter().any(|a| a.starts_with("-Djava.library.path=")));
    assert!(args.jvm_args.contains(&"-Dminecraft.launcher.brand=nova-launcher".to_string()));

    assert_eq!(args.game_args[0], "--username");
    assert_eq!(args.game_args[1], "Alex");
    assert_eq!(args.game_args[2], "--version");
    assert_eq!(args.game_args[3], "1.21.1");
}