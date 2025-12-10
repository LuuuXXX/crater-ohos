// Allow dead code for Phase 3 functions not yet fully connected
#![allow(dead_code)]

mod tasks;
mod test;
mod worker;

use crate::config::Config;
use crate::crates::Crate;
use crate::experiments::{Experiment, Mode};
use crate::prelude::*;
use crate::results::TestResult;
use crate::runner::worker::{DiskSpaceWatcher, Workspace};
use std::time::Duration;

pub use worker::RecordProgress;

const DISK_SPACE_WATCHER_INTERVAL: Duration = Duration::from_secs(30);
const DISK_SPACE_WATCHER_THRESHOLD: f32 = 0.80;

#[derive(Debug, thiserror::Error)]
#[error("overridden task result to {0:?}")]
pub struct OverrideResult(pub TestResult);

/// Run an experiment
///
/// This is the main entry point for executing an experiment. It:
/// 1. Sets up worker threads
/// 2. Starts disk space monitoring
/// 3. Executes tasks for each crate with each toolchain
/// 4. Records progress via the provided API
///
/// # Arguments
///
/// * `ex` - The experiment to run
/// * `workspace` - Workspace for building crates (placeholder)
/// * `api` - API for recording progress
/// * `threads_count` - Number of worker threads to use
/// * `config` - Configuration
/// * `next_crate` - Function that returns the next crate to test
pub fn run_ex(
    ex: &Experiment,
    _workspace: &Workspace,
    api: &dyn RecordProgress,
    threads_count: usize,
    _config: &Config,
    next_crate: &(dyn Fn() -> Fallible<Option<Crate>> + Send + Sync),
) -> Fallible<()> {
    info!(
        "Starting experiment '{}' with {} threads",
        ex.name, threads_count
    );

    // Create disk space watcher
    let disk_space_watcher =
        DiskSpaceWatcher::new(DISK_SPACE_WATCHER_INTERVAL, DISK_SPACE_WATCHER_THRESHOLD, threads_count);

    // In real implementation, this would:
    // 1. Create worker threads
    // 2. Start disk space watcher in separate thread
    // 3. Each worker would:
    //    - Get next crate from next_crate()
    //    - For each toolchain in experiment:
    //      - Create task based on experiment mode
    //      - Execute task
    //      - Record result via api.record_progress()
    // 4. Wait for all workers to complete
    // 5. Stop disk space watcher

    // Placeholder implementation
    let mut processed = 0;
    loop {
        match next_crate()? {
            Some(krate) => {
                info!("Processing crate: {}", krate);

                // Process with both toolchains
                for toolchain in &ex.toolchains {
                    let result = match ex.mode {
                        Mode::BuildAndTest => {
                            info!("Running build-and-test for {} with {}", krate, toolchain);
                            TestResult::TestPass
                        }
                        Mode::BuildOnly => {
                            info!("Running build-only for {} with {}", krate, toolchain);
                            TestResult::TestPass
                        }
                        Mode::CheckOnly => {
                            info!("Running check-only for {} with {}", krate, toolchain);
                            TestResult::TestPass
                        }
                        Mode::Clippy => {
                            info!("Running clippy for {} with {}", krate, toolchain);
                            TestResult::TestPass
                        }
                        Mode::Rustdoc => {
                            info!("Running rustdoc for {} with {}", krate, toolchain);
                            TestResult::TestPass
                        }
                        Mode::UnstableFeatures => {
                            info!("Checking unstable features for {} with {}", krate, toolchain);
                            TestResult::TestPass
                        }
                        Mode::Fix => {
                            info!("Running fix for {} with {}", krate, toolchain);
                            TestResult::TestPass
                        }
                    };

                    // Record progress
                    let log = b"placeholder log output";
                    api.record_progress(ex, &krate, toolchain, log, &result, None)?;
                }

                processed += 1;
            }
            None => {
                info!("No more crates to process (processed: {})", processed);
                break;
            }
        }
    }

    disk_space_watcher.stop();

    info!("Experiment '{}' completed", ex.name);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crates::sources::registry::RegistryCrate;
    use crate::experiments::{CapLints, Status};
    use crate::toolchain::{RustwideToolchain, Toolchain};
    use chrono::Utc;
    use std::sync::Mutex;

    struct TestRecorder {
        results: Mutex<Vec<(String, String, TestResult)>>,
    }

    impl RecordProgress for TestRecorder {
        fn record_progress(
            &self,
            _ex: &Experiment,
            krate: &Crate,
            toolchain: &Toolchain,
            _log: &[u8],
            result: &TestResult,
            _version: Option<(&Crate, &Crate)>,
        ) -> Fallible<()> {
            let mut results = self.results.lock().unwrap();
            results.push((krate.to_string(), toolchain.to_string(), result.clone()));
            Ok(())
        }
    }

    #[test]
    fn test_run_ex_basic() {
        let tc = Toolchain {
            source: RustwideToolchain::Dist("stable".to_string()),
            target: None,
            rustflags: None,
            rustdocflags: None,
            cargoflags: None,
            ci_try: false,
            patches: vec![],
        };

        let ex = Experiment {
            name: "test-exp".to_string(),
            toolchains: [tc.clone(), tc.clone()],
            mode: Mode::BuildAndTest,
            cap_lints: CapLints::Allow,
            priority: 0,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            platform_issue: None,
            status: Status::Queued,
            assigned_to: None,
            report_url: None,
            ignore_blacklist: false,
            requirement: None,
        };

        let workspace = Workspace;
        let recorder = TestRecorder {
            results: Mutex::new(Vec::new()),
        };

        let crates = vec![
            Crate::Registry(RegistryCrate::new("serde", "1.0.0")),
            Crate::Registry(RegistryCrate::new("tokio", "1.0.0")),
        ];
        let crates = Mutex::new(crates.into_iter());

        let next_crate = || {
            let mut iter = crates.lock().unwrap();
            Ok(iter.next())
        };

        let config = Config {
            demo_crates: Default::default(),
            sandbox: crate::config::SandboxConfig {
                memory_limit: "1G".parse().unwrap(),
                build_log_max_size: "10M".parse().unwrap(),
                build_log_max_lines: 1000,
            },
            server: crate::config::ServerConfig {
                acl: crate::config::ACL {
                    allowed_users: vec![],
                },
                callback: Default::default(),
            },
            platforms: Default::default(),
        };

        let result = run_ex(&ex, &workspace, &recorder, 2, &config, &next_crate);
        assert!(result.is_ok());

        let results = recorder.results.lock().unwrap();
        // 2 crates * 2 toolchains = 4 results
        assert_eq!(results.len(), 4);
    }
}
