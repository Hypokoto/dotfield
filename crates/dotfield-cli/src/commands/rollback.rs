use anyhow::Result;
use colored::Colorize;
use dotfield_core::GenerationManager;

pub fn execute() -> Result<()> {
    let mgr = GenerationManager::new(GenerationManager::default_dir());
    let rolled_gen = mgr.rollback()?;
    println!(
        "{}",
        format!("✔ Rolled back to Generation {rolled_gen}!")
            .green()
            .bold()
    );
    println!("Pointer ~/.dotfield/current updated.");

    Ok(())
}
