use crate::actions::experiments::{CreateExperiment, ExperimentActions};
use crate::db::Database;
use crate::experiments::{CrateSelect, Mode};
use crate::prelude::*;
use crate::toolchain::Toolchain;

pub fn define_ex(
    db: &Database,
    name: String,
    toolchain1: String,
    toolchain2: String,
    crate_select: String,
    mode: String,
    priority: i32,
) -> Fallible<()> {
    println!("Defining experiment '{}'...", name);

    // Parse toolchains
    let tc1: Toolchain = toolchain1.parse()?;
    let tc2: Toolchain = toolchain2.parse()?;

    // Parse mode
    let mode: Mode = mode.parse()?;

    // Parse crate selection
    let crate_select: CrateSelect = crate_select.parse()?;

    let req = CreateExperiment {
        name: name.clone(),
        toolchains: [tc1, tc2],
        mode,
        crate_select,
        platform_issue: None,
        callback_url: None,
        priority,
    };

    let experiment = db.create(req)?;
    
    println!("✓ Experiment '{}' created successfully", experiment.name);
    println!("  Status: {:?}", experiment.status);
    println!("  Mode: {:?}", experiment.mode);
    println!("  Toolchains: {} vs {}", 
        experiment.toolchains[0], 
        experiment.toolchains[1]);

    Ok(())
}
