use crate::actions::experiments::ExperimentActions;
use crate::db::Database;
use crate::prelude::*;

pub fn list_ex(db: &Database) -> Fallible<()> {
    println!("Listing all experiments...\n");

    let experiments = db.list()?;

    if experiments.is_empty() {
        println!("No experiments found.");
        return Ok(());
    }

    println!("{:<30} {:<15} {:<20} {:<10}", "Name", "Status", "Mode", "Priority");
    println!("{}", "-".repeat(80));

    for exp in experiments {
        println!(
            "{:<30} {:<15} {:<20} {:<10}",
            exp.name,
            format!("{:?}", exp.status),
            format!("{:?}", exp.mode),
            exp.priority
        );
    }

    println!("\nTotal: {} experiment(s)", db.list()?.len());

    Ok(())
}

pub fn delete_ex(db: &Database, name: String) -> Fallible<()> {
    println!("Deleting experiment '{}'...", name);

    db.delete(&name)?;

    println!("✓ Experiment '{}' deleted successfully", name);

    Ok(())
}

pub fn abort_ex(db: &Database, name: String) -> Fallible<()> {
    println!("Aborting experiment '{}'...", name);

    db.abort(&name, "Aborted by user")?;

    println!("✓ Experiment '{}' aborted successfully", name);

    Ok(())
}
