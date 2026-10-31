mod power;

use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use dotfield_core::{Compiler, GenerationManager, Profile, Reloader};
use power::{detect_power_source, PowerSource};

#[derive(Parser, Debug)]
#[command(
    name = "dotfieldd",
    about = "Background hardware event listener & automated profile switching daemon",
    version = "0.1.0"
)]
struct Cli {
    /// Run continuously as a background daemon
    #[arg(long)]
    daemon: bool,

    /// Polling interval in seconds (default: 5)
    #[arg(long, default_value = "5")]
    interval_secs: u64,

    /// Perform a single evaluation of hardware state and apply profile
    #[arg(long)]
    once: bool,

    /// Dry run: evaluate state transitions without deploying generations
    #[arg(long)]
    dry_run: bool,

    /// Profile TOML to activate when on AC / Mains power
    #[arg(long, default_value = "profiles/default.toml")]
    ac_profile: PathBuf,

    /// Profile TOML to activate when on Battery power
    #[arg(long, default_value = "profiles/minimal.toml")]
    battery_profile: PathBuf,

    /// Sysfs power supply path
    #[arg(long, default_value = "/sys/class/power_supply")]
    sysfs_path: PathBuf,
}

fn apply_profile_if_needed(
    target_profile_path: &Path,
    dry_run: bool,
    source_label: &str,
) -> Result<()> {
    if !target_profile_path.exists() {
        println!(
            "{}",
            format!(
                "[dotfieldd] Target profile path does not exist: {}",
                target_profile_path.display()
            )
            .yellow()
        );
        return Ok(());
    }

    let profile = Profile::from_file(target_profile_path).with_context(|| {
        format!(
            "Failed to read profile at {}",
            target_profile_path.display()
        )
    })?;

    let mgr = GenerationManager::new(GenerationManager::default_dir());
    let current_active = mgr
        .get_state()?
        .map(|s| s.active_profile)
        .unwrap_or_default();

    if current_active == profile.meta.name {
        println!(
            "{}",
            format!(
                "[dotfieldd] Profile '{}' already active for {source_label}.",
                profile.meta.name
            )
            .dimmed()
        );
        return Ok(());
    }

    println!(
        "{}",
        format!(
            "[dotfieldd] Power state changed to {source_label}! Switching to profile '{}'...",
            profile.meta.name
        )
        .cyan()
        .bold()
    );

    if dry_run {
        println!(
            "{}",
            "[dotfieldd] Dry-run enabled: skipping deployment.".yellow()
        );
        return Ok(());
    }

    let fragments = Compiler::compile(&profile)?;
    let new_gen = mgr.deploy(&profile, &fragments)?;
    println!(
        "{}",
        format!(
            "[dotfieldd] ✔ Successfully deployed Generation {new_gen} ('{}')",
            profile.meta.name
        )
        .green()
        .bold()
    );

    let reloaded = Reloader::reload_changed_targets(&fragments)?;
    for target in reloaded {
        println!("{}", format!("[dotfieldd]   ✔ Reloaded {target}").green());
    }

    Ok(())
}

fn run_evaluation(cli: &Cli, last_source: &mut Option<PowerSource>) -> Result<()> {
    let current_source = detect_power_source(&cli.sysfs_path);

    if Some(current_source) != *last_source {
        *last_source = Some(current_source);

        match current_source {
            PowerSource::Ac => {
                apply_profile_if_needed(&cli.ac_profile, cli.dry_run, "AC Power")?;
            }
            PowerSource::Battery => {
                apply_profile_if_needed(&cli.battery_profile, cli.dry_run, "Battery Power")?;
            }
            PowerSource::Unknown => {
                println!(
                    "{}",
                    "[dotfieldd] Hardware power state unknown; maintaining current generation."
                        .dimmed()
                );
            }
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!(
        "{}",
        "dotfieldd: Hardware Event Daemon initialized".cyan().bold()
    );

    let mut last_source = None;

    if cli.once || !cli.daemon {
        run_evaluation(&cli, &mut last_source)?;
        return Ok(());
    }

    println!(
        "{}",
        format!(
            "[dotfieldd] Monitoring {} every {}s. Press Ctrl+C to terminate.",
            cli.sysfs_path.display(),
            cli.interval_secs
        )
        .dimmed()
    );

    loop {
        if let Err(e) = run_evaluation(&cli, &mut last_source) {
            eprintln!("{}", format!("[dotfieldd] Evaluation error: {e}").red());
        }
        sleep(Duration::from_secs(cli.interval_secs));
    }
}
