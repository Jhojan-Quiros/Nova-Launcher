use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use crate::domain::entities::{Instance, MinecraftAccount};
use crate::domain::errors::LauncherError;
use crate::infrastructure::minecraft::rule_evaluator::{ArgumentRuleEvaluator, PlatformEnvironment};
use crate::shared::config::LauncherPaths;

pub struct LaunchArguments {
    pub main_class: String,
    pub jvm_args: Vec<String>,
    pub game_args: Vec<String>,
}

pub struct ArgumentBuilder;

impl ArgumentBuilder {
    pub fn build(
        version_json: &Value,
        instance: &Instance,
        paths: &LauncherPaths,
        account: &MinecraftAccount,
        env: &PlatformEnvironment,
    ) -> Result<LaunchArguments, LauncherError> {
        let main_class = version_json
            .get("mainClass")
            .and_then(|m| m.as_str())
            .unwrap_or("net.minecraft.client.main.Main")
            .to_string();

        let asset_index_id = version_json
            .pointer("/assetIndex/id")
            .and_then(|i| i.as_str())
            .unwrap_or(&instance.minecraft_version)
            .to_string();

        let natives_dir = paths.instance_natives_dir(&instance.id).to_string_lossy().to_string();
        let game_dir = if std::path::Path::new(&instance.game_directory).is_absolute() {
            instance.game_directory.clone()
        } else {
            paths.instance_game_dir(&instance.id).to_string_lossy().to_string()
        };
        let assets_dir = paths.assets_dir().to_string_lossy().to_string();

        // Build classpath
        let classpath = Self::build_classpath(version_json, instance, paths, env)?;

        // Setup placeholder mappings
        let mut placeholders = HashMap::new();
        placeholders.insert("natives_directory", natives_dir.clone());
        placeholders.insert("launcher_name", "nova-launcher".to_string());
        placeholders.insert("launcher_version", "1.0.0".to_string());
        placeholders.insert("classpath", classpath.clone());
        let classpath_sep = if cfg!(target_os = "windows") { ";" } else { ":" };
        placeholders.insert("classpath_separator", classpath_sep.to_string());
        placeholders.insert("library_directory", paths.libraries_dir().to_string_lossy().to_string());
        placeholders.insert("auth_player_name", account.username.clone());
        placeholders.insert("version_name", instance.minecraft_version.clone());
        placeholders.insert("game_directory", game_dir.clone());
        placeholders.insert("assets_root", assets_dir.clone());
        placeholders.insert("assets_index_name", asset_index_id.clone());
        placeholders.insert("auth_uuid", account.uuid.clone());
        placeholders.insert("auth_access_token", account.access_token.clone());
        placeholders.insert("clientid", "0".to_string());
        placeholders.insert("auth_xuid", "0".to_string());
        placeholders.insert("user_type", "mojang".to_string());
        placeholders.insert("version_type", "release".to_string());

        // Build JVM arguments
        let mut jvm_args = Vec::new();
        // Base memory configuration
        jvm_args.push(format!("-Xms{}M", instance.ram.min_mb));
        jvm_args.push(format!("-Xmx{}M", instance.ram.max_mb));

        // Modern JVM flags (Java 17/21/25) to suppress restricted native access warnings
        let is_modern_java = version_json
            .pointer("/javaVersion/majorVersion")
            .and_then(|v| v.as_i64())
            .map(|m| m >= 17)
            .unwrap_or(true);

        if is_modern_java {
            jvm_args.push("-XX:+IgnoreUnrecognizedVMOptions".to_string());
            jvm_args.push("--enable-native-access=ALL-UNNAMED".to_string());
        }

        if let Some(jvm_array) = version_json.pointer("/arguments/jvm").and_then(|j| j.as_array()) {
            for arg_entry in jvm_array {
                Self::process_arg_entry(arg_entry, &mut jvm_args, &placeholders, env);
            }
        } else {
            // Default JVM arguments for older versions
            jvm_args.push(format!("-Djava.library.path={}", natives_dir));
            jvm_args.push("-Dminecraft.launcher.brand=nova-launcher".to_string());
            jvm_args.push("-Dminecraft.launcher.version=1.0.0".to_string());
            jvm_args.push("-cp".to_string());
            jvm_args.push(classpath.clone());
        }

        // Build Game arguments
        let mut game_args = Vec::new();

        if let Some(game_array) = version_json.pointer("/arguments/game").and_then(|g| g.as_array()) {
            for arg_entry in game_array {
                Self::process_arg_entry(arg_entry, &mut game_args, &placeholders, env);
            }
        } else if let Some(legacy_args) = version_json.get("minecraftArguments").and_then(|m| m.as_str()) {
            for token in legacy_args.split_whitespace() {
                game_args.push(Self::substitute_placeholders(token, &placeholders));
            }
        }

        Ok(LaunchArguments {
            main_class,
            jvm_args,
            game_args,
        })
    }

    fn build_classpath(
        version_json: &Value,
        instance: &Instance,
        paths: &LauncherPaths,
        env: &PlatformEnvironment,
    ) -> Result<String, LauncherError> {
        let mut parts: Vec<PathBuf> = Vec::new();

        if let Some(libraries) = version_json.get("libraries").and_then(|l| l.as_array()) {
            for lib in libraries {
                let rules = lib.get("rules").and_then(|r| r.as_array());
                if !ArgumentRuleEvaluator::is_allowed(rules, env) {
                    continue;
                }

                if let Some(path_str) = lib.pointer("/downloads/artifact/path").and_then(|p| p.as_str()) {
                    let full_path = paths.libraries_dir().join(path_str);
                    parts.push(full_path);
                }
            }
        }

        // Add client jar
        let client_jar = paths
            .versions_dir()
            .join(&instance.minecraft_version)
            .join(format!("{}.jar", &instance.minecraft_version));

        if !client_jar.exists() {
            return Err(LauncherError::minecraft(
                format!("Minecraft client JAR not found at {:?}. Please install the instance first.", client_jar),
                None,
            ));
        }

        parts.push(client_jar);

        let sep = if cfg!(target_os = "windows") { ";" } else { ":" };
        let cp_str = parts
            .into_iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<String>>()
            .join(sep);

        Ok(cp_str)
    }

    fn process_arg_entry(
        entry: &Value,
        target_list: &mut Vec<String>,
        placeholders: &HashMap<&str, String>,
        env: &PlatformEnvironment,
    ) {
        match entry {
            Value::String(s) => {
                target_list.push(Self::substitute_placeholders(s, placeholders));
            }
            Value::Object(obj) => {
                let rules = obj.get("rules").and_then(|r| r.as_array());
                if ArgumentRuleEvaluator::is_allowed(rules, env) {
                    if let Some(value) = obj.get("value") {
                        match value {
                            Value::String(val_str) => {
                                target_list.push(Self::substitute_placeholders(val_str, placeholders));
                            }
                            Value::Array(val_arr) => {
                                for item in val_arr {
                                    if let Some(item_str) = item.as_str() {
                                        target_list.push(Self::substitute_placeholders(item_str, placeholders));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub fn substitute_placeholders(template: &str, placeholders: &HashMap<&str, String>) -> String {
        let mut result = template.to_string();
        for (key, val) in placeholders {
            let placeholder_pattern = format!("${{{}}}", key);
            result = result.replace(&placeholder_pattern, val);
        }
        result
    }
}