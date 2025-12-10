use crate::prelude::*;
use crate::results::{BrokenReason, FailureReason, TestResult};
use crate::runner::tasks::TaskCtx;
use crate::runner::OverrideResult;

// Placeholder for rustwide types
pub struct LogStorage;
pub struct PrepareError;

/// Extract failure reason from an error
pub(crate) fn failure_reason(err: &anyhow::Error) -> FailureReason {
    let err_string = format!("{:#}", err);

    if err_string.contains("out of memory") || err_string.contains("OOM") {
        FailureReason::OOM
    } else if err_string.contains("no space left") || err_string.contains("disk full") {
        FailureReason::NoSpace
    } else if err_string.contains("timeout") || err_string.contains("timed out") {
        FailureReason::Timeout
    } else if err_string.contains("internal compiler error")
        || err_string.contains("ICE")
        || err_string.contains("thread 'rustc' panicked")
    {
        FailureReason::ICE
    } else if err_string.contains("network") || err_string.contains("connection") {
        FailureReason::NetworkAccess
    } else if err_string.contains("docker") || err_string.contains("container") {
        FailureReason::Docker
    } else {
        FailureReason::Unknown
    }
}

/// Detect if a crate is broken and convert error accordingly
pub(super) fn detect_broken<T>(res: Result<T, anyhow::Error>) -> Result<T, anyhow::Error> {
    if let Err(ref err) = res {
        let err_string = format!("{:#}", err);

        if err_string.contains("Cargo.toml") && err_string.contains("parse") {
            return Err(OverrideResult(TestResult::BrokenCrate(
                BrokenReason::CargoToml,
            ))
            .into());
        } else if err_string.contains("yanked") {
            return Err(
                OverrideResult(TestResult::BrokenCrate(BrokenReason::Yanked)).into(),
            );
        } else if err_string.contains("missing") && err_string.contains("dependencies") {
            return Err(OverrideResult(TestResult::BrokenCrate(
                BrokenReason::MissingDependencies,
            ))
            .into());
        } else if err_string.contains("git") && err_string.contains("not found") {
            return Err(OverrideResult(TestResult::BrokenCrate(
                BrokenReason::MissingGitRepository,
            ))
            .into());
        }
    }
    res
}

/// Placeholder for actual test execution - would integrate with rustwide
pub(super) fn run_test(
    action: &str,
    ctx: &TaskCtx,
    test_fn: fn(&TaskCtx) -> Fallible<TestResult>,
) -> Fallible<TestResult> {
    info!(
        "Running {} for crate {} with toolchain {}",
        action, ctx.krate, ctx.toolchain
    );

    // In real implementation, this would set up logging, build environment, etc.
    let result = detect_broken(test_fn(ctx))?;
    Ok(result)
}

/// Build and test a crate
pub(super) fn test_build_and_test(ctx: &TaskCtx) -> Fallible<TestResult> {
    info!("Building and testing crate {}", ctx.krate);

    // Placeholder implementation
    // In real implementation, this would:
    // 1. Prepare the build environment
    // 2. Run cargo build
    // 3. Run cargo test
    // 4. Parse output and return appropriate TestResult

    Ok(TestResult::TestPass)
}

/// Build a crate without running tests
pub(super) fn test_build_only(ctx: &TaskCtx) -> Fallible<TestResult> {
    info!("Building crate {} (no tests)", ctx.krate);

    // Placeholder implementation
    Ok(TestResult::TestPass)
}

/// Check a crate without building
pub(super) fn test_check_only(ctx: &TaskCtx) -> Fallible<TestResult> {
    info!("Checking crate {}", ctx.krate);

    // Placeholder implementation
    Ok(TestResult::TestPass)
}

/// Run clippy on a crate
pub(super) fn test_clippy_only(ctx: &TaskCtx) -> Fallible<TestResult> {
    info!("Running clippy on crate {}", ctx.krate);

    // Placeholder implementation
    Ok(TestResult::TestPass)
}

/// Generate rustdoc for a crate
pub(super) fn test_rustdoc(ctx: &TaskCtx) -> Fallible<TestResult> {
    info!("Generating rustdoc for crate {}", ctx.krate);

    // Placeholder implementation
    Ok(TestResult::TestPass)
}

/// Run cargo fix on a crate
pub(super) fn fix(ctx: &TaskCtx) -> Fallible<TestResult> {
    info!("Running cargo fix on crate {}", ctx.krate);

    // Placeholder implementation
    Ok(TestResult::TestPass)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_failure_reason_oom() {
        let err = anyhow::anyhow!("process killed: out of memory");
        assert_eq!(failure_reason(&err), FailureReason::OOM);
    }

    #[test]
    fn test_failure_reason_no_space() {
        let err = anyhow::anyhow!("no space left on device");
        assert_eq!(failure_reason(&err), FailureReason::NoSpace);
    }

    #[test]
    fn test_failure_reason_timeout() {
        let err = anyhow::anyhow!("operation timed out");
        assert_eq!(failure_reason(&err), FailureReason::Timeout);
    }

    #[test]
    fn test_failure_reason_ice() {
        let err = anyhow::anyhow!("internal compiler error: unexpected panic");
        assert_eq!(failure_reason(&err), FailureReason::ICE);
    }

    #[test]
    fn test_failure_reason_unknown() {
        let err = anyhow::anyhow!("some random error");
        assert_eq!(failure_reason(&err), FailureReason::Unknown);
    }

    #[test]
    fn test_detect_broken_cargo_toml() {
        let err = anyhow::anyhow!("failed to parse Cargo.toml");
        let result: Result<(), _> = detect_broken(Err(err));

        assert!(result.is_err());
        let err = result.unwrap_err();
        if let Some(override_result) = err.downcast_ref::<OverrideResult>() {
            assert_eq!(
                override_result.0,
                TestResult::BrokenCrate(BrokenReason::CargoToml)
            );
        } else {
            panic!("Expected OverrideResult");
        }
    }

    #[test]
    fn test_detect_broken_yanked() {
        let err = anyhow::anyhow!("dependency has been yanked");
        let result: Result<(), _> = detect_broken(Err(err));

        assert!(result.is_err());
        let err = result.unwrap_err();
        if let Some(override_result) = err.downcast_ref::<OverrideResult>() {
            assert_eq!(
                override_result.0,
                TestResult::BrokenCrate(BrokenReason::Yanked)
            );
        } else {
            panic!("Expected OverrideResult");
        }
    }
}
