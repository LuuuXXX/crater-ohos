use crater_ohos::actions::experiments::{CreateExperiment, EditExperiment, ExperimentActions};
use crater_ohos::db::Database;
use crater_ohos::experiments::{CrateSelect, Mode, PlatformIssue, Status};
use crater_ohos::server::agents::{AgentManager, RegisterAgent};
use crater_ohos::server::callback::{CallbackEvent, CallbackPayload, CallbackService};
use crater_ohos::server::tokens::{Permission, TokenManager};
use crater_ohos::toolchain::Toolchain;
use std::str::FromStr;

#[test]
fn test_experiment_workflow() {
    // 创建 -> 运行 -> 完成 的完整流程
    let db = Database::temp().unwrap();

    // Create experiment
    let req = CreateExperiment {
        name: "integration-test".to_string(),
        toolchains: [
            Toolchain::from_str("stable").unwrap(),
            Toolchain::from_str("beta").unwrap(),
        ],
        mode: Mode::BuildAndTest,
        crate_select: CrateSelect::Demo,
        platform_issue: Some(PlatformIssue {
            platform: "github".to_string(),
            api_url: "https://api.github.com/repos/test/test/issues/1".to_string(),
            html_url: "https://github.com/test/test/issues/1".to_string(),
            identifier: "1".to_string(),
        }),
        callback_url: Some("https://example.com/callback".to_string()),
        priority: 0,
    };

    let exp = db.create(req).unwrap();
    assert_eq!(exp.name, "integration-test");
    assert_eq!(exp.status, Status::Queued);

    // Edit experiment (change priority)
    let edit_req = EditExperiment {
        priority: Some(10),
        ..Default::default()
    };
    let exp = db.edit("integration-test", edit_req).unwrap();
    assert_eq!(exp.priority, 10);

    // Run experiment
    db.run("integration-test").unwrap();
    let exp = db.get("integration-test").unwrap().unwrap();
    assert_eq!(exp.status, Status::Running);

    // Complete experiment
    db.complete("integration-test").unwrap();
    let exp = db.get("integration-test").unwrap().unwrap();
    assert_eq!(exp.status, Status::Completed);
    assert!(exp.started_at.is_some());
    assert!(exp.completed_at.is_some());
}

#[test]
fn test_agent_task_assignment() {
    // Agent 注册 -> 分配任务 -> 完成任务
    let db = Database::temp().unwrap();

    // Register agent
    let agent_req = RegisterAgent {
        name: "integration-agent".to_string(),
        capabilities: vec!["build".to_string(), "test".to_string()],
    };
    let agent = db.register_agent(agent_req).unwrap();
    assert_eq!(agent.name, "integration-agent");
    assert_eq!(agent.capabilities.len(), 2);

    // Create experiment
    let exp_req = CreateExperiment {
        name: "agent-test-exp".to_string(),
        toolchains: [
            Toolchain::from_str("stable").unwrap(),
            Toolchain::from_str("beta").unwrap(),
        ],
        mode: Mode::BuildAndTest,
        crate_select: CrateSelect::Demo,
        platform_issue: None,
        callback_url: None,
        priority: 0,
    };
    db.create(exp_req).unwrap();

    // Assign task to agent
    db.assign_task(&agent.id, "agent-test-exp").unwrap();
    let updated_agent = db.get_agent(&agent.id).unwrap().unwrap();
    assert_eq!(
        updated_agent.current_experiment,
        Some("agent-test-exp".to_string())
    );

    // Send heartbeat
    db.heartbeat(&agent.id).unwrap();

    // Complete task
    db.complete_task(&agent.id).unwrap();
    let updated_agent = db.get_agent(&agent.id).unwrap().unwrap();
    assert_eq!(updated_agent.current_experiment, None);
}

#[test]
fn test_token_management() {
    let db = Database::temp().unwrap();

    // Create token
    let permissions = vec![Permission::ReadExperiments, Permission::WriteExperiments];
    let token = db.create_token("integration-token", permissions).unwrap();
    assert!(token.token.starts_with("crt_"));

    // Validate token
    let validated = db.validate_token(&token.token).unwrap();
    assert!(validated.is_some());

    // List tokens
    let tokens = db.list_tokens().unwrap();
    assert_eq!(tokens.len(), 1);

    // Revoke token
    db.revoke_token(&token.token).unwrap();

    // Validate revoked token
    let validated = db.validate_token(&token.token).unwrap();
    assert!(validated.is_none());
}

#[test]
fn test_callback_payload_creation() {
    // Test callback payload serialization and deserialization
    let payload = CallbackPayload::new(
        "test-exp".to_string(),
        CallbackEvent::ExperimentCompleted,
        Status::Completed.to_string(),
        Some("https://example.com/report".to_string()),
        None,
    );

    let json = serde_json::to_string(&payload).unwrap();
    let parsed: CallbackPayload = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.experiment, "test-exp");
    assert_eq!(parsed.status, Status::Completed.to_string());
    assert_eq!(
        parsed.report_url,
        Some("https://example.com/report".to_string())
    );
}

#[test]
fn test_callback_service_creation() {
    use crater_ohos::config::CallbackConfig;

    let config = CallbackConfig {
        timeout_secs: Some(30),
        retry_count: Some(3),
    };

    let _service = CallbackService::new(config);
    // Just verify the service can be created without errors
}

#[test]
fn test_full_lifecycle_with_agents_and_tokens() {
    let db = Database::temp().unwrap();

    // Create a token
    let token = db
        .create_token("admin-token", vec![Permission::Admin])
        .unwrap();

    // Validate token
    let validated = db.validate_token(&token.token).unwrap();
    assert!(validated.is_some());
    let validated_token = validated.unwrap();
    assert!(validated_token.permissions.contains(&Permission::Admin));

    // Register an agent
    let agent = db
        .register_agent(RegisterAgent {
            name: "worker-1".to_string(),
            capabilities: vec!["build".to_string()],
        })
        .unwrap();

    // Create an experiment
    let exp = db
        .create(CreateExperiment {
            name: "full-test".to_string(),
            toolchains: [
                Toolchain::from_str("stable").unwrap(),
                Toolchain::from_str("beta").unwrap(),
            ],
            mode: Mode::BuildAndTest,
            crate_select: CrateSelect::Demo,
            platform_issue: None,
            callback_url: Some("https://example.com/webhook".to_string()),
            priority: 0,
        })
        .unwrap();
    assert_eq!(exp.status, Status::Queued);

    // Assign experiment to agent
    db.assign_task(&agent.id, "full-test").unwrap();

    // Run experiment
    db.run("full-test").unwrap();
    let exp = db.get("full-test").unwrap().unwrap();
    assert_eq!(exp.status, Status::Running);

    // Complete experiment
    db.complete("full-test").unwrap();

    // Mark agent task as complete
    db.complete_task(&agent.id).unwrap();

    // Verify final state
    let exp = db.get("full-test").unwrap().unwrap();
    assert_eq!(exp.status, Status::Completed);

    let agent = db.get_agent(&agent.id).unwrap().unwrap();
    assert_eq!(agent.current_experiment, None);
}
