use anyhow::Result;
use colored::Colorize;
use dotfield_core::GenerationManager;

pub fn execute() -> Result<()> {
    let mgr = GenerationManager::new(GenerationManager::default_dir());
    let state = mgr.get_state()?;

    println!("{}", "Dotfield State Status".bold().underline());
    match state {
        Some(s) => {
            println!("  Active Profile:    {}", s.active_profile.green().bold());
            println!(
                "  Active Generation: {}",
                s.current_generation.to_string().cyan().bold()
            );
            println!("  Last Deployed:     {}", s.timestamp.dimmed());
            println!("  Live Link:         ~/.dotfield/current");
        }
        None => {
            println!(
                "  Status: {}",
                "No active generation deployed yet. Run `dotfield apply` to start.".yellow()
            );
        }
    }

    Ok(())
}
