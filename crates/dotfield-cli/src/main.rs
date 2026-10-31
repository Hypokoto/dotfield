use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;

#[derive(Parser)]
#[command(name = "dotfield")]
#[command(about = "Declarative Linux ricing compiler and dotfile manager in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show current deployment generation and active profile state
    Status,

    /// Check a profile TOML for syntax and system requirements
    Check {
        /// Path to profile TOML
        #[arg(default_value = "profiles/default.toml")]
        profile: PathBuf,
    },

    /// Preview configuration fragment diffs before applying
    Diff {
        /// Path to profile TOML
        #[arg(default_value = "profiles/default.toml")]
        profile: PathBuf,
    },

    /// Compile and atomically deploy a profile generation with selective reload
    Apply {
        /// Path to profile TOML
        #[arg(default_value = "profiles/default.toml")]
        profile: PathBuf,
    },

    /// Roll back to the previous deployed generation
    Rollback,

    /// List available profiles in profiles/ directory
    List,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Status => commands::status::execute()?,
        Commands::Check { profile } => commands::check::execute(&profile)?,
        Commands::Diff { profile } => commands::diff::execute(&profile)?,
        Commands::Apply { profile } => commands::apply::execute(&profile)?,
        Commands::Rollback => commands::rollback::execute()?,
        Commands::List => commands::list::execute()?,
    }

    Ok(())
}
