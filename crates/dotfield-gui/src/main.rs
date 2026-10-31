use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use anyhow::{Context, Result};
use clap::Parser;
use dotfield_core::{Compiler, GenerationManager, Profile, Reloader};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

slint::include_modules!();

#[derive(Parser, Debug)]
#[command(
    name = "dotfield-gui",
    about = "Wayland-native GUI utilities for Dotfield",
    version = "0.1.0"
)]
struct Cli {
    /// Show the Keybind Cheat-Sheet HUD
    #[arg(long)]
    hud: bool,

    /// Show the Profile Switcher card interface
    #[arg(long)]
    switcher: bool,

    /// Path to specific profile TOML (for HUD display)
    #[arg(short, long)]
    profile: Option<PathBuf>,

    /// Directory containing profile TOML files
    #[arg(long, default_value = "profiles")]
    profiles_dir: PathBuf,
}

fn resolve_active_profile_path(explicit_path: Option<&PathBuf>) -> PathBuf {
    if let Some(p) = explicit_path {
        if p.exists() {
            return p.clone();
        }
    }

    let mgr = GenerationManager::new(GenerationManager::default_dir());
    if let Ok(Some(state)) = mgr.get_state() {
        let candidate = PathBuf::from("profiles").join(format!("{}.toml", state.active_profile));
        if candidate.exists() {
            return candidate;
        }
    }

    let default_path = PathBuf::from("profiles/default.toml");
    if default_path.exists() {
        default_path
    } else {
        PathBuf::from("default.toml")
    }
}

fn run_hud(profile_path: &Path) -> Result<()> {
    let profile = Profile::from_file(profile_path).context("Failed to load profile for HUD")?;
    let hud = KeybindHud::new().map_err(|e| anyhow::anyhow!("Failed to create HUD: {e}"))?;

    hud.set_profile_name(SharedString::from(&profile.meta.name));

    // Extract keybind entries
    let mut all_items: Vec<KeybindItem> = Vec::new();

    for (action, keybind) in &profile.bindings {
        let category = if action == "terminal"
            || action == "launcher"
            || action == "editor"
            || action == "multiplexer"
        {
            "apps"
        } else if action.starts_with("focus_") || action.starts_with("move_") || action == "close" {
            "window"
        } else {
            "system"
        };

        all_items.push(KeybindItem {
            action: SharedString::from(action),
            keybind: SharedString::from(keybind),
            category: SharedString::from(category),
        });
    }

    all_items.sort_by(|a, b| a.action.cmp(&b.action));

    let items_model = Rc::new(VecModel::from(all_items.clone()));
    hud.set_items(ModelRc::from(items_model));

    let hud_weak = hud.as_weak();
    let all_items_cached = all_items;

    hud.on_search_changed(move |query| {
        if let Some(hud) = hud_weak.upgrade() {
            let q = query.to_lowercase();
            let filtered: Vec<KeybindItem> = if q.is_empty() {
                all_items_cached.clone()
            } else {
                all_items_cached
                    .iter()
                    .filter(|item| {
                        item.action.to_lowercase().contains(&q)
                            || item.keybind.to_lowercase().contains(&q)
                            || item.category.to_lowercase().contains(&q)
                    })
                    .cloned()
                    .collect()
            };
            hud.set_items(ModelRc::from(Rc::new(VecModel::from(filtered))));
        }
    });

    hud.on_close_clicked(|| {
        slint::quit_event_loop().ok();
    });

    hud.run()
        .map_err(|e| anyhow::anyhow!("Error running HUD window: {e}"))?;
    Ok(())
}

fn load_available_profiles(
    profiles_dir: &Path,
    active_name: &str,
) -> Result<Vec<(PathBuf, ProfileCardData)>> {
    let mut profiles = Vec::new();

    if profiles_dir.exists() && profiles_dir.is_dir() {
        let mut entries: Vec<_> = std::fs::read_dir(profiles_dir)?
            .filter_map(std::result::Result::ok)
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "toml"))
            .collect();
        entries.sort_by_key(|e| e.path());

        for entry in entries {
            let path = entry.path();
            if let Ok(profile) = Profile::from_file(&path) {
                let is_active = profile.meta.name == active_name;
                let security = if profile.meta.name.contains("hardening") {
                    "Hardened"
                } else {
                    "Standard"
                };

                let card = ProfileCardData {
                    name: SharedString::from(&profile.meta.name),
                    description: SharedString::from(
                        profile.meta.description.as_deref().unwrap_or(""),
                    ),
                    compositor: SharedString::from(&profile.compositor.engine),
                    apps: SharedString::from(format!(
                        "{}, {}",
                        profile.apps.terminal, profile.apps.launcher
                    )),
                    security: SharedString::from(security),
                    active: is_active,
                };
                profiles.push((path, card));
            }
        }
    }

    Ok(profiles)
}

fn run_switcher(profiles_dir: &Path) -> Result<()> {
    let mgr = GenerationManager::new(GenerationManager::default_dir());
    let active_name = mgr
        .get_state()
        .ok()
        .flatten()
        .map(|s| s.active_profile)
        .unwrap_or_else(|| "default".to_string());

    let loaded = load_available_profiles(profiles_dir, &active_name)?;
    let switcher =
        ProfileSwitcher::new().map_err(|e| anyhow::anyhow!("Failed to create Switcher: {e}"))?;

    let cards: Vec<ProfileCardData> = loaded.iter().map(|(_, c)| c.clone()).collect();
    let model = Rc::new(VecModel::from(cards));
    switcher.set_profiles(ModelRc::from(model));

    let switcher_weak = switcher.as_weak();
    let loaded_ref = Rc::new(RefCell::new(loaded));
    let profiles_dir_owned = profiles_dir.to_path_buf();

    switcher.on_apply_profile(move |target_name| {
        if let Some(switcher) = switcher_weak.upgrade() {
            let target_str = target_name.as_str();
            switcher.set_status_message(SharedString::from(format!(
                "Applying profile '{target_str}'..."
            )));

            let found_path = loaded_ref
                .borrow()
                .iter()
                .find(|(_, c)| c.name.as_str() == target_str)
                .map(|(p, _)| p.clone());

            if let Some(path) = found_path {
                match Profile::from_file(&path) {
                    Ok(profile) => match Compiler::compile(&profile) {
                        Ok(fragments) => {
                            let mgr = GenerationManager::new(GenerationManager::default_dir());
                            match mgr.deploy(&profile, &fragments) {
                                Ok(new_gen) => {
                                    let _ = Reloader::reload_changed_targets(&fragments);
                                    switcher.set_status_message(SharedString::from(format!(
                                        "✔ Generation {new_gen} deployed ('{target_str}')"
                                    )));

                                    // Refresh active state in cards
                                    if let Ok(updated) = load_available_profiles(
                                        &profiles_dir_owned,
                                        &profile.meta.name,
                                    ) {
                                        *loaded_ref.borrow_mut() = updated;
                                        let new_cards: Vec<ProfileCardData> = loaded_ref
                                            .borrow()
                                            .iter()
                                            .map(|(_, c)| c.clone())
                                            .collect();
                                        switcher.set_profiles(ModelRc::from(Rc::new(
                                            VecModel::from(new_cards),
                                        )));
                                    }
                                }
                                Err(e) => {
                                    switcher.set_status_message(SharedString::from(format!(
                                        "Deploy failed: {e}"
                                    )));
                                }
                            }
                        }
                        Err(e) => {
                            switcher.set_status_message(SharedString::from(format!(
                                "Compile failed: {e}"
                            )));
                        }
                    },
                    Err(e) => {
                        switcher.set_status_message(SharedString::from(format!(
                            "Failed to read profile: {e}"
                        )));
                    }
                }
            }
        }
    });

    switcher.on_close_clicked(|| {
        slint::quit_event_loop().ok();
    });

    switcher
        .run()
        .map_err(|e| anyhow::anyhow!("Error running Switcher window: {e}"))?;
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.switcher {
        run_switcher(&cli.profiles_dir)?;
    } else {
        // Default to HUD if --hud or neither specified
        let profile_path = resolve_active_profile_path(cli.profile.as_ref());
        run_hud(&profile_path)?;
    }

    Ok(())
}
