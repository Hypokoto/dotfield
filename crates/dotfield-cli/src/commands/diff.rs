use anyhow::Result;
use colored::Colorize;
use dotfield_core::{Compiler, DiffEngine, GenerationManager, Profile};
use std::path::Path;

pub fn execute(profile_path: &Path) -> Result<()> {
    let profile = Profile::from_file(profile_path)?;
    let fragments = Compiler::compile(&profile)?;

    let current_dir = GenerationManager::default_dir().join("current");
    let current_path_opt = if current_dir.exists() {
        Some(current_dir.as_path())
    } else {
        None
    };

    let diffs = DiffEngine::diff_fragments(current_path_opt, &fragments)?;

    println!(
        "{}",
        format!("Diff for profile '{}':", profile.meta.name)
            .bold()
            .underline()
    );
    for diff in diffs {
        println!("\n--- Fragment: {} ---", diff.path.cyan().bold());
        for line in diff.diff_text.lines() {
            if line.starts_with('+') {
                println!("{}", line.green());
            } else if line.starts_with('-') {
                println!("{}", line.red());
            } else {
                println!("{}", line.dimmed());
            }
        }
    }

    Ok(())
}
