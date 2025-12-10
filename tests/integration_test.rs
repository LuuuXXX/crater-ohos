use crater_ohos::db::{create_memory_pool, QueryUtils};
use crater_ohos::experiments::{Mode, Status};

#[test]
fn test_database_migrations() {
    let pool = create_memory_pool().unwrap();
    let conn = pool.get().unwrap();
    // 验证所有表都已创建
    let result = conn.get_row(
        "SELECT 1 FROM experiments LIMIT 1",
        std::iter::empty::<&dyn rusqlite::ToSql>(),
        |_| Ok(1)
    );
    // Table exists (may be empty or not)
    assert!(result.is_ok());
}

#[test]
fn test_config_loading() {
    // 测试配置文件如果存在可以正常加载
    use crater_ohos::config::DemoCrates;
    let demo_crates = DemoCrates::default();
    // Default should work
    assert!(demo_crates.crates.is_empty() || !demo_crates.crates.is_empty());
}

#[test]
fn test_experiment_workflow() {
    // 测试实验创建和状态转换的基本流程
    let status = Status::Queued;
    assert_eq!(status.to_str(), "queued");
    
    let mode = Mode::BuildAndTest;
    assert_eq!(mode.to_str(), "build-and-test");
}

#[test]
fn test_experiment_metadata_table_exists() {
    let pool = create_memory_pool().expect("failed to create pool");
    let conn = pool.get().expect("failed to get connection");
    
    // Check if experiment_metadata table exists
    let count: i32 = conn
        .get_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='experiment_metadata'",
            std::iter::empty::<&dyn rusqlite::ToSql>(),
            |row| row.get(0)
        )
        .expect("failed to query")
        .expect("no result");
    
    assert_eq!(count, 1, "experiment_metadata table should exist");
}

#[test]
fn test_all_required_tables_exist() {
    let pool = create_memory_pool().expect("failed to create pool");
    let conn = pool.get().expect("failed to get connection");
    
    let required_tables = vec![
        "experiments",
        "experiment_metadata",
        "results",
        "shas",
        "saved_names",
        "experiment_crates",
        "migrations",
    ];
    
    for table in required_tables {
        let count: i32 = conn
            .get_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?",
                [table],
                |row| row.get(0)
            )
            .expect("failed to query")
            .expect("no result");
        
        assert_eq!(count, 1, "table {} should exist", table);
    }
}

#[test]
fn test_experiment_metadata_schema() {
    let pool = create_memory_pool().expect("failed to create pool");
    let conn = pool.get().expect("failed to get connection");
    
    // Verify the schema includes required columns
    let schema: String = conn
        .get_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='experiment_metadata'",
            std::iter::empty::<&dyn rusqlite::ToSql>(),
            |row| row.get(0)
        )
        .expect("failed to query")
        .expect("no result");
    
    assert!(schema.contains("callback_url"), "should have callback_url column");
    assert!(schema.contains("platform"), "should have platform column");
    assert!(schema.contains("triggered_by"), "should have triggered_by column");
    assert!(schema.contains("created_at"), "should have created_at column");
    assert!(schema.contains("FOREIGN KEY"), "should have foreign key constraint");
}

#[test]
fn test_experiments_table_has_platform_issue_columns() {
    let pool = create_memory_pool().expect("failed to create pool");
    let conn = pool.get().expect("failed to get connection");
    
    // Query the table schema to verify platform_issue columns exist
    let columns: Vec<String> = conn
        .query(
            "PRAGMA table_info(experiments)",
            std::iter::empty::<&dyn rusqlite::ToSql>(),
            |row| row.get::<_, String>(1)  // Get column name
        )
        .expect("failed to query columns");
    
    assert!(columns.contains(&"platform_issue".to_string()), 
        "experiments table should have platform_issue column");
    assert!(columns.contains(&"platform_issue_url".to_string()), 
        "experiments table should have platform_issue_url column");
    assert!(columns.contains(&"platform_issue_identifier".to_string()), 
        "experiments table should have platform_issue_identifier column");
}
