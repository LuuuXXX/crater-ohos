use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "crater-ohos")]
#[command(about = "A tool for testing third-party libraries in OHOS environments")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Prepare local environment
    PrepareLocal,

    /// Define an experiment
    DefineEx {
        /// Experiment name
        #[arg(long = "ex")]
        name: String,

        /// First toolchain
        toolchain1: String,

        /// Second toolchain
        toolchain2: String,

        /// Crate selection strategy
        #[arg(long = "crate-select")]
        crate_select: String,

        /// Experiment mode (default: build-and-test)
        #[arg(long = "mode", default_value = "build-and-test")]
        mode: String,

        /// Priority (default: 0)
        #[arg(long = "priority", default_value = "0")]
        priority: i32,
    },

    /// Run experiment
    RunGraph {
        /// Experiment name
        #[arg(long = "ex")]
        name: String,

        /// Number of threads
        #[arg(short = 't', long = "threads", default_value = "1")]
        threads: usize,
    },

    /// Generate report
    GenReport {
        /// Experiment name
        #[arg(long = "ex")]
        name: String,

        /// Output directory
        output_dir: String,
    },

    /// Start API server
    Server {
        /// Server port
        #[arg(long = "port", default_value = "3000")]
        port: u16,

        /// Config file path
        #[arg(long = "config", default_value = "config.toml")]
        config: String,
    },

    /// List all experiments
    ListEx,

    /// Delete an experiment
    DeleteEx {
        /// Experiment name
        #[arg(long = "ex")]
        name: String,
    },

    /// Abort an experiment
    AbortEx {
        /// Experiment name
        #[arg(long = "ex")]
        name: String,
    },
}
