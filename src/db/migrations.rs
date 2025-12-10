use crate::prelude::*;
use rusqlite::Connection;

pub fn execute(conn: &Connection) -> Fallible<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS migrations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            executed_at TEXT NOT NULL
        );
        ",
    )?;

    let mut executed_migrations = std::collections::HashSet::new();
    {
        let mut stmt = conn.prepare("SELECT name FROM migrations")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        for row in rows {
            executed_migrations.insert(row?);
        }
    }

    for migration in MIGRATIONS {
        if !executed_migrations.contains(migration.name) {
            info!("executing migration: {}", migration.name);
            conn.execute_batch(migration.sql)?;
            conn.execute(
                "INSERT INTO migrations (name, executed_at) VALUES (?, datetime('now'))",
                [migration.name],
            )?;
        }
    }

    Ok(())
}

struct Migration {
    name: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        name: "create_experiments_table",
        sql: "
            CREATE TABLE experiments (
                name TEXT PRIMARY KEY,
                mode TEXT NOT NULL,
                cap_lints TEXT NOT NULL,
                toolchain_start TEXT,
                toolchain_end TEXT,
                priority INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                started_at TEXT,
                completed_at TEXT,
                github_issue TEXT,
                github_issue_url TEXT,
                status TEXT NOT NULL,
                assigned_to TEXT,
                report_url TEXT,
                ignore_blacklist INTEGER NOT NULL DEFAULT 0
            );

            CREATE INDEX experiments__status ON experiments (status);
            CREATE INDEX experiments__assigned_to ON experiments (assigned_to);
        ",
    },
    Migration {
        name: "create_results_table",
        sql: "
            CREATE TABLE results (
                experiment TEXT NOT NULL,
                crate TEXT NOT NULL,
                toolchain TEXT NOT NULL,
                result TEXT NOT NULL,
                log BLOB,
                PRIMARY KEY (experiment, crate, toolchain),
                FOREIGN KEY (experiment) REFERENCES experiments(name) ON DELETE CASCADE
            );

            CREATE INDEX results__experiment ON results (experiment);
        ",
    },
    Migration {
        name: "create_shas_table",
        sql: "
            CREATE TABLE shas (
                experiment TEXT NOT NULL,
                org TEXT NOT NULL,
                name TEXT NOT NULL,
                sha TEXT NOT NULL,
                PRIMARY KEY (experiment, org, name),
                FOREIGN KEY (experiment) REFERENCES experiments(name) ON DELETE CASCADE
            );
        ",
    },
    Migration {
        name: "create_saved_names_table",
        sql: "
            CREATE TABLE saved_names (
                experiment TEXT NOT NULL,
                toolchain TEXT NOT NULL,
                name TEXT NOT NULL,
                PRIMARY KEY (experiment, toolchain),
                FOREIGN KEY (experiment) REFERENCES experiments(name) ON DELETE CASCADE
            );
        ",
    },
    Migration {
        name: "create_experiment_crates_table",
        sql: "
            CREATE TABLE experiment_crates (
                experiment TEXT NOT NULL,
                crate TEXT NOT NULL,
                skipped INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (experiment, crate),
                FOREIGN KEY (experiment) REFERENCES experiments(name) ON DELETE CASCADE
            );

            CREATE INDEX experiment_crates__experiment ON experiment_crates (experiment);
        ",
    },
    Migration {
        name: "create_experiment_metadata_table",
        sql: "
            CREATE TABLE IF NOT EXISTS experiment_metadata (
                experiment TEXT PRIMARY KEY,
                callback_url TEXT,
                platform TEXT,
                triggered_by TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (experiment) REFERENCES experiments(name) ON DELETE CASCADE
            );

            CREATE INDEX experiment_metadata__experiment ON experiment_metadata(experiment);
        ",
    },
    Migration {
        name: "rename_github_issue_to_platform_issue",
        sql: "
            -- Add new platform_issue columns for platform-agnostic design
            -- platform_issue: stores platform identifier (github, gitcode, gitlab, etc.)
            -- platform_issue_url: stores the HTML URL for the issue/PR
            -- platform_issue_identifier: stores the issue/PR number or identifier
            ALTER TABLE experiments ADD COLUMN platform_issue TEXT;
            ALTER TABLE experiments ADD COLUMN platform_issue_url TEXT;
            ALTER TABLE experiments ADD COLUMN platform_issue_identifier TEXT;
            
            -- Migrate data from old GitHub-specific columns
            -- Note: platform_issue_identifier is a new field, so no migration needed
            UPDATE experiments SET platform_issue = github_issue WHERE github_issue IS NOT NULL;
            UPDATE experiments SET platform_issue_url = github_issue_url WHERE github_issue_url IS NOT NULL;
        ",
    },
];
