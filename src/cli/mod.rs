pub mod args;
pub mod commands;

use crate::db::Database;
use crate::prelude::*;
use args::{Cli, Commands};
use clap::Parser;

pub async fn run() -> Fallible<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::PrepareLocal => commands::prepare::prepare_local(),
        
        Commands::DefineEx {
            name,
            toolchain1,
            toolchain2,
            crate_select,
            mode,
            priority,
        } => {
            let db = Database::open()?;
            commands::define::define_ex(
                &db,
                name,
                toolchain1,
                toolchain2,
                crate_select,
                mode,
                priority,
            )
        }
        
        Commands::RunGraph { name, threads } => {
            let db = Database::open()?;
            commands::run::run_graph(&db, name, threads)
        }
        
        Commands::GenReport { name, output_dir } => {
            let db = Database::open()?;
            commands::report::gen_report(&db, name, output_dir)
        }
        
        Commands::Server { port, config } => {
            commands::server::server(port, config).await
        }
        
        Commands::ListEx => {
            let db = Database::open()?;
            commands::manage::list_ex(&db)
        }
        
        Commands::DeleteEx { name } => {
            let db = Database::open()?;
            commands::manage::delete_ex(&db, name)
        }
        
        Commands::AbortEx { name } => {
            let db = Database::open()?;
            commands::manage::abort_ex(&db, name)
        }
    }
}
