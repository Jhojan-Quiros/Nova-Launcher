use serde_json::Value;

#[derive(Debug, Clone)]
pub struct PlatformEnvironment {
    pub os_name: String,   // "windows", "linux", "osx"
    pub arch: String,      // "x86", "x86_64", "arm64"
    pub is_demo_user: bool,
    pub has_custom_resolution: bool,
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

        // Evaluate features condition
        if let Some(features) = rule.get("features") {
            if let Some(is_demo) = features.get("is_demo_user").and_then(|d| d.as_bool()) {
                if is_demo != env.is_demo_user {
                    return false;
                }
            }
            if let Some(has_res) = features.get("has_custom_resolution").and_then(|r| r.as_bool()) {
                if has_res != env.has_custom_resolution {
                    return false;
                }
            }
        }

        true
    }
}