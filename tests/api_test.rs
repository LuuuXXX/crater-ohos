use crater_ohos::actions::experiments::{CreateExperiment, ExperimentActions};
use crater_ohos::db::Database;
use crater_ohos::experiments::{CrateSelect, Mode};
use crater_ohos::server::tokens::{Permission, TokenManager};
use crater_ohos::toolchain::Toolchain;
use std::str::FromStr;

#[test]
fn test_api_module_exists() {
    // Simple test to verify the API module compiles and is accessible
    use crater_ohos::api;
    
    // This test ensures the API module structure is correct
    assert!(true);
}

#[tokio::test]
async fn test_experiment_creation_for_api() {
    let db = Database::temp().unwrap();
    
    // Create an experiment (this would be done via API in real usage)
    let req = CreateExperiment {
        name: "test-api-experiment".to_string(),
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
    
    let experiment = db.create(req).unwrap();
    assert_eq!(experiment.name, "test-api-experiment");
    
    // List experiments
    let experiments = db.list().unwrap();
    assert_eq!(experiments.len(), 1);
    
    // Get experiment
    let retrieved = db.get("test-api-experiment").unwrap();
    assert!(retrieved.is_some());
}

#[tokio::test]
async fn test_token_creation_for_api() {
    let db = Database::temp().unwrap();
    
    // Create a test token (used for API authentication)
    let token = db
        .create_token("test-api-token", vec![Permission::ReadExperiments, Permission::WriteExperiments])
        .unwrap();

    assert!(token.token.starts_with("crt_"));
    assert_eq!(token.name, "test-api-token");
    assert_eq!(token.permissions.len(), 2);
    
    // Validate token
    let validated = db.validate_token(&token.token).unwrap();
    assert!(validated.is_some());
}

#[tokio::test]
async fn test_agent_registration_for_api() {
    use crater_ohos::server::agents::{AgentManager, RegisterAgent};
    
    let db = Database::temp().unwrap();
    
    let req = RegisterAgent {
        name: "test-agent".to_string(),
        capabilities: vec!["build".to_string(), "test".to_string()],
    };
    
    let agent = db.register_agent(req).unwrap();
    assert!(agent.id.starts_with("agent-"));
    assert_eq!(agent.name, "test-agent");
    assert_eq!(agent.capabilities.len(), 2);
    
    // List agents
    let agents = db.list_agents().unwrap();
    assert_eq!(agents.len(), 1);
}

#[test]
fn test_cli_module_exists() {
    // Simple test to verify the CLI module compiles and is accessible
    use crater_ohos::cli;
    
    // This test ensures the CLI module structure is correct
    assert!(true);
}

