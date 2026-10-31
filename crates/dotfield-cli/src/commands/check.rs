use anyhow::Result;
use colored::Colorize;
use dotfield_core::{Profile, Validator};
use std::path::Path;

pub fn execute(profile_path: &Path) -> Result<()> {
    println!(
        "Checking profile: {}",
        profile_path.display().to_string().cyan()
    );
    let profile = Profile::from_file(profile_path)?;

    let issues = Validator::validate(&profile)?;
    if issues.is_empty() {
        println!(
            "{}",
            "✔ Profile is valid and all prerequisites are met!"
                .green()
                .bold()
        );
    } else {
        println!("{}", "Warnings detected:".yellow().bold());
        for issue in issues {
            println!("  - {}", issue.message.yellow());
        }
    }

    Ok(())
}
