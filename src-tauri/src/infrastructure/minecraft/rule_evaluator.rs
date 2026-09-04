use serde_json::Value;

#[derive(Debug, Clone)]
pub struct PlatformEnvironment {
    pub os_name: String,   // "windows", "linux", "osx"
    pub arch: String,      // "x86", "x86_64", "arm64"
    pub is_demo_user: bool,
    pub has_custom_resolution: bool,
    pub is_quick_play_singleplayer: bool,
    pub is_quick_play_multiplayer: bool,
    pub is_quick_play_realms: bool,
    pub has_quick_plays_support: bool,
}

impl Default for PlatformEnvironment {
    fn default() -> Self {
        Self::current()
    }
}

impl PlatformEnvironment {
    pub fn current() -> Self {
        let os_name = if cfg!(target_os = "windows") {
            "windows".to_string()
        } else if cfg!(target_os = "macos") {
            "osx".to_string()
        } else {
            "linux".to_string()
        };

        let arch = if cfg!(target_arch = "x86_64") {
            "x86_64".to_string()
        } else if cfg!(target_arch = "x86") {
            "x86".to_string()
        } else if cfg!(target_arch = "aarch64") {
            "arm64".to_string()
        } else {
            "unknown".to_string()
        };

        Self {
            os_name,
            arch,
            is_demo_user: false,
            has_custom_resolution: false,
            is_quick_play_singleplayer: false,
            is_quick_play_multiplayer: false,
            is_quick_play_realms: false,
            has_quick_plays_support: false,
        }
    }
}

pub struct ArgumentRuleEvaluator;

impl ArgumentRuleEvaluator {
    pub fn is_allowed(rules: Option<&Vec<Value>>, env: &PlatformEnvironment) -> bool {
        let rules = match rules {
            Some(r) if !r.is_empty() => r,
            _ => return true, // Default allowed if no rules specified
        };

        let mut allowed = false;

        for rule in rules {
            if let Some(action) = rule.get("action").and_then(|a| a.as_str()) {
                let matches_rule = Self::eval_rule_conditions(rule, env);
                if matches_rule {
                    allowed = action == "allow";
                }
            }
        }

        allowed
    }

    fn eval_rule_conditions(rule: &Value, env: &PlatformEnvironment) -> bool {
        // Evaluate OS condition
        if let Some(os) = rule.get("os") {
            if let Some(name) = os.get("name").and_then(|n| n.as_str()) {
                if name != env.os_name {
                    return false;
                }
            }
            if let Some(arch) = os.get("arch").and_then(|a| a.as_str()) {
                if arch != env.arch && !(arch == "x64" && env.arch == "x86_64") {
                    return false;
                }
            }
        }

        // Evaluate features condition strictly: if a rule specifies a feature,
        // it must match the environment state; unknown/inactive features evaluate to false.
        if let Some(features) = rule.get("features").and_then(|f| f.as_object()) {
            for (feature_key, required_val) in features {
                let required_bool = required_val.as_bool().unwrap_or(false);
                let actual_bool = match feature_key.as_str() {
                    "is_demo_user" => env.is_demo_user,
                    "has_custom_resolution" => env.has_custom_resolution,
                    "is_quick_play_singleplayer" => env.is_quick_play_singleplayer,
                    "is_quick_play_multiplayer" => env.is_quick_play_multiplayer,
                    "is_quick_play_realms" => env.is_quick_play_realms,
                    "has_quick_plays_support" => env.has_quick_plays_support,
                    _ => false,
                };
                if actual_bool != required_bool {
                    return false;
                }
            }
        }

        true
    }
}