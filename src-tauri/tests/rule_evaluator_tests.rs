use serde_json::json;
use nova_launcher_lib::infrastructure::minecraft::rule_evaluator::{ArgumentRuleEvaluator, PlatformEnvironment};

#[test]
fn test_rule_evaluator() {
    let mut win_env = PlatformEnvironment::current();
    win_env.os_name = "windows".to_string();
    win_env.arch = "x86_64".to_string();
    win_env.is_demo_user = false;
    win_env.has_custom_resolution = false;

    let mut osx_env = PlatformEnvironment::current();
    osx_env.os_name = "osx".to_string();
    osx_env.arch = "x86_64".to_string();
    osx_env.is_demo_user = false;
    osx_env.has_custom_resolution = false;

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

    // 5. Quick play feature rules - must be rejected when not enabled
    let qp_rules = vec![
        json!({
            "action": "allow",
            "features": { "is_quick_play_singleplayer": true }
        })
    ];
    assert!(!ArgumentRuleEvaluator::is_allowed(Some(&qp_rules), &win_env));
}