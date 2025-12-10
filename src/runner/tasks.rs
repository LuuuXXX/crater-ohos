use crate::config::Config;
use crate::crates::Crate;
use crate::experiments::Experiment;
use crate::toolchain::Toolchain;
use std::fmt;

// Placeholder for rustwide types - in real implementation, these would come from rustwide crate
pub struct BuildDirectory;
pub struct Build;

pub(super) struct TaskCtx<'ctx> {
    pub(super) config: &'ctx Config,
    pub(super) experiment: &'ctx Experiment,
    pub(super) toolchain: &'ctx Toolchain,
    pub(super) krate: &'ctx Crate,
    pub(super) quiet: bool,
}

pub(super) enum TaskStep {
    BuildAndTest { tc: Toolchain, quiet: bool },
    BuildOnly { tc: Toolchain, quiet: bool },
    CheckOnly { tc: Toolchain, quiet: bool },
    Clippy { tc: Toolchain, quiet: bool },
    Rustdoc { tc: Toolchain, quiet: bool },
    UnstableFeatures { tc: Toolchain },
    Fix { tc: Toolchain, quiet: bool },
}

impl TaskStep {
    pub(super) fn toolchain(&self) -> &Toolchain {
        match self {
            TaskStep::BuildAndTest { tc, .. } => tc,
            TaskStep::BuildOnly { tc, .. } => tc,
            TaskStep::CheckOnly { tc, .. } => tc,
            TaskStep::Clippy { tc, .. } => tc,
            TaskStep::Rustdoc { tc, .. } => tc,
            TaskStep::UnstableFeatures { tc } => tc,
            TaskStep::Fix { tc, .. } => tc,
        }
    }

    pub(super) fn is_quiet(&self) -> bool {
        match self {
            TaskStep::BuildAndTest { quiet, .. } => *quiet,
            TaskStep::BuildOnly { quiet, .. } => *quiet,
            TaskStep::CheckOnly { quiet, .. } => *quiet,
            TaskStep::Clippy { quiet, .. } => *quiet,
            TaskStep::Rustdoc { quiet, .. } => *quiet,
            TaskStep::UnstableFeatures { .. } => false,
            TaskStep::Fix { quiet, .. } => *quiet,
        }
    }
}

impl fmt::Display for TaskStep {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TaskStep::BuildAndTest { tc, .. } => write!(f, "build and test with {}", tc),
            TaskStep::BuildOnly { tc, .. } => write!(f, "build with {}", tc),
            TaskStep::CheckOnly { tc, .. } => write!(f, "check with {}", tc),
            TaskStep::Clippy { tc, .. } => write!(f, "clippy with {}", tc),
            TaskStep::Rustdoc { tc, .. } => write!(f, "rustdoc with {}", tc),
            TaskStep::UnstableFeatures { tc } => write!(f, "unstable features with {}", tc),
            TaskStep::Fix { tc, .. } => write!(f, "fix with {}", tc),
        }
    }
}

pub(super) struct Task {
    pub(super) krate: Crate,
    pub(super) step: TaskStep,
}

impl Task {
    pub(super) fn new(krate: Crate, step: TaskStep) -> Self {
        Task { krate, step }
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} for crate {}", self.step, self.krate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crates::sources::registry::RegistryCrate;
    use crate::toolchain::RustwideToolchain;

    #[test]
    fn test_task_step_toolchain() {
        let tc = Toolchain {
            source: RustwideToolchain::Dist("stable".to_string()),
            target: None,
            rustflags: None,
            rustdocflags: None,
            cargoflags: None,
            ci_try: false,
            patches: vec![],
        };

        let step = TaskStep::BuildAndTest {
            tc: tc.clone(),
            quiet: false,
        };
        assert_eq!(step.toolchain().source, tc.source);
        assert!(!step.is_quiet());
    }

    #[test]
    fn test_task_display() {
        let tc = Toolchain {
            source: RustwideToolchain::Dist("stable".to_string()),
            target: None,
            rustflags: None,
            rustdocflags: None,
            cargoflags: None,
            ci_try: false,
            patches: vec![],
        };

        let krate = Crate::Registry(RegistryCrate::new("serde", "1.0.0"));
        let step = TaskStep::BuildAndTest {
            tc: tc.clone(),
            quiet: false,
        };
        let task = Task::new(krate, step);

        let display = format!("{}", task);
        assert!(display.contains("build and test"));
        assert!(display.contains("serde"));
    }
}
