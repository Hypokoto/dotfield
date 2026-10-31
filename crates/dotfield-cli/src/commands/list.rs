use anyhow::Result;
use colored::Colorize;
use dotfield_core::Profile;
use std::fs;

pub fn execute() -> Result<()> {
    let profiles_dir = Profile::default_profiles_dir();
    if !profiles_dir.exists() {
        println!(
            "No profiles directory found at '{}'",
            profiles_dir.display()
        );
        return Ok(());
    }

    println!("{}", "Available Profiles:".bold().underline());
    for entry in fs::read_dir(profiles_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("toml") {
            match Profile::from_file(&path) {
                Ok(profile) => {
                    let desc = profile
                        .meta
                        .description
                        .unwrap_or_else(|| "No description".to_string());
                    println!(
                        "  • {} - {}",
                        profile.meta.name.cyan().bold(),
                        desc.dimmed()
                    );
                }
                Err(e) => {
                    eprintln!(
                        "  {} Failed to load profile at '{}': {e}",
                        "!".yellow().bold(),
                        path.display()
                    );
                }
            }
        }
    }

    Ok(())
}
