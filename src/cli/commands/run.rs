use crate::actions::experiments::ExperimentActions;
use crate::db::Database;
use crate::prelude::*;

pub fn run_graph(db: &Database, name: String, threads: usize) -> Fallible<()> {
    println!("Running experiment '{}' with {} threads...", name, threads);

    // Check if experiment exists
    let experiment = db.get(&name)?
        .ok_or_else(|| anyhow::anyhow!("Experiment '{}' not found", name))?;

    println!("  Experiment: {}", experiment.name);
    println!("  Mode: {:?}", experiment.mode);
    println!("  Status: {:?}", experiment.status);

    // Update status to running
    db.run(&name)?;

    println!("✓ Experiment '{}' started", name);
    println!("  Note: Actual execution is not yet implemented (Phase 3 integration pending)");

    Ok(())
}
