use serde_json::json;
use nova_launcher_lib::infrastructure::minecraft::rule_evaluator::{ArgumentRuleEvaluator, PlatformEnvironment};

#[test]
fn test_rule_evaluator() {
    let win_env = PlatformEnvironment {
        os_name: "windows".to_string(),
        arch: "x86_64".to_string(),
        is_demo_user: false,
        has_custom_resolution: false,
    };

    let osx_env = PlatformEnvironment {
        os_name: "osx".to_string(),
        arch: "x86_64".to_string(),
        is_demo_user: false,
        has_custom_resolution: false,
    };

    // 1. No rules -> allowed
    assert!(ArgumentRuleEvaluator::is_allowed(None, &win_env));
    assert!(ArgumentRuleEvaluator::is_allowed(Some(&vec![]), &win_env));

    // 2. Allow only windows
    let win_only_rules = vec![
        json!({
            "action": "allow",
            "os": { "name": "windows" }
        })
    ];
    assert!(ArgumentRuleEvaluator::is_allowed(Some(&win_only_rules), &win_env));
    assert!(!ArgumentRuleEvaluator::is_allowed(Some(&win_only_rules), &osx_env));

    // 3. Allow all, but disallow osx
    let disallow_osx = vec![
        json!({ "action": "allow" }),
        json!({
            "action": "disallow",
            "os": { "name": "osx" }
        })
    ];
    assert!(ArgumentRuleEvaluator::is_allowed(Some(&disallow_osx), &win_env));
    assert!(!ArgumentRuleEvaluator::is_allowed(Some(&disallow_osx), &osx_env));

    // 4. Feature check (demo user)
    let demo_rules = vec![
        json!({
            "action": "allow",
            "features": { "is_demo_user": true }
        })
    ];
    assert!(!ArgumentRuleEvaluator::is_allowed(Some(&demo_rules), &win_env));

    let mut demo_env = win_env.clone();
    demo_env.is_demo_user = true;
    assert!(ArgumentRuleEvaluator::is_allowed(Some(&demo_rules), &demo_env));
}