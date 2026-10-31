use anyhow::Result;
use colored::Colorize;
use dotfield_core::{Compiler, GenerationManager, Profile, Reloader, Validator};
use std::path::Path;

pub fn execute(profile_path: &Path) -> Result<()> {
    println!(
        "Loading profile: {}",
        profile_path.display().to_string().cyan()
    );
    let profile = Profile::from_file(profile_path)?;

    // 1. Validate profile
    let issues = Validator::validate(&profile)?;
    for issue in issues {
        println!("  - Warning: {}", issue.message.yellow());
    }

    // 2. Compile fragments
    println!("{}", "Compiling target fragments...".dimmed());
    let fragments = Compiler::compile(&profile)?;

    // 3. Deploy generation
    let mgr = GenerationManager::new(GenerationManager::default_dir());
    let new_gen = mgr.deploy(&profile, &fragments)?;
    println!(
        "{}",
        format!("✔ Deployed Generation {new_gen} successfully!")
            .green()
            .bold()
    );

    // 4. Selective reload
    println!("{}", "Executing selective reload...".dimmed());
    let reloaded = Reloader::reload_changed_targets(&fragments)?;
    if reloaded.is_empty() {
        println!("  No active processes required reload.");
    } else {
        for target in reloaded {
            println!("  ✔ Reloaded: {}", target.green());
        }
    }

    Ok(())
}
