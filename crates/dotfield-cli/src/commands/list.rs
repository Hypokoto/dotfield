use anyhow::Result;
use colored::Colorize;
use dotfield_core::Profile;
use std::fs;
use std::path::Path;

pub fn execute() -> Result<()> {
    let profiles_dir = Path::new("profiles");
    if !profiles_dir.exists() {
        println!("No profiles directory found at 'profiles/'");
        return Ok(());
    }

    println!("{}", "Available Profiles:".bold().underline());
    for entry in fs::read_dir(profiles_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("toml") {
            if let Ok(profile) = Profile::from_file(&path) {
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
        }
    }

    Ok(())
}
