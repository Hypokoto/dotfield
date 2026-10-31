use anyhow::Result;
use colored::Colorize;
use dotfield_core::{GenerationManager, Reloader};

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

    let fragments = match mgr.get_generation_fragments(rolled_gen) {
        Ok(f) => f,
        Err(e) => {
            eprintln!(
                "  {} Could not read generation fragments: {e}",
                "!".yellow().bold()
            );
            anyhow::bail!(
                "Rollback succeeded, but could not read generation fragments for reload: {e}"
            );
        }
    };

    match Reloader::reload_changed_targets(&fragments) {
        Ok(reloaded) => {
            if !reloaded.is_empty() {
                for target in reloaded {
                    println!("  ✔ Reloaded: {}", target.green());
                }
            }
        }
        Err(e) => {
            eprintln!("  {} Reload incomplete: {e}", "!".yellow().bold());
            anyhow::bail!("Rollback succeeded, but reload of changed targets failed: {e}");
        }
    }

    Ok(())
}
