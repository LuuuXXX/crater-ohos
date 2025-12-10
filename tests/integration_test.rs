use crater_ohos::db::{create_memory_pool, QueryUtils};

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
