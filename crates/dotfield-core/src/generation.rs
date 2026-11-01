use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::symlink;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};

use crate::compiler::CompiledFragment;
use crate::error::{DotfieldError, Result};
use crate::profile::Profile;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GenerationState {
    pub current_generation: u64,
    pub active_profile: String,
    pub timestamp: String,
    #[serde(default)]
    pub history: Vec<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GenerationMeta {
    pub generation: u64,
    pub profile_name: String,
    pub timestamp: String,
}

struct GenDirGuard<'a> {
    path: &'a Path,
    armed: bool,
}

impl Drop for GenDirGuard<'_> {
    fn drop(&mut self) {
        if self.armed {
            let _ = fs::remove_dir_all(self.path);
        }
    }
}

pub struct GenerationManager {
    base_dir: PathBuf,
}

impl GenerationManager {
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    pub fn default_dir() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".dotfield")
    }

    pub fn get_state(&self) -> Result<Option<GenerationState>> {
        let state_path = self.base_dir.join("state.json");
        let current_symlink = self.base_dir.join("current");

        let mut loaded_state: Option<GenerationState> = if state_path.exists() {
            let content = fs::read_to_string(&state_path)?;
            let state: GenerationState =
                serde_json::from_str(&content).map_err(|e| DotfieldError::State(e.to_string()))?;
            Some(state)
        } else {
            None
        };

        // Reconcile and self-heal state if current symlink points to a valid generation with meta.json
        if let Ok(meta_content) = fs::read_to_string(current_symlink.join("meta.json")) {
            if let Ok(meta) = serde_json::from_str::<GenerationMeta>(&meta_content) {
                if let Some(ref mut st) = loaded_state {
                    if st.current_generation != meta.generation {
                        st.current_generation = meta.generation;
                        st.active_profile = meta.profile_name;
                    }
                } else {
                    loaded_state = Some(GenerationState {
                        current_generation: meta.generation,
                        active_profile: meta.profile_name,
                        timestamp: meta.timestamp,
                        history: Vec::new(),
                    });
                }
            }
        }

        Ok(loaded_state)
    }

    fn acquire_lock(&self) -> Result<std::fs::File> {
        fs::create_dir_all(&self.base_dir)?;
        let lock_path = self.base_dir.join(".lock");
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)?;
        let res = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) };
        if res != 0 {
            return Err(DotfieldError::State(format!(
                "Failed to acquire cross-process lock on {lock_path:?}: {}",
                std::io::Error::last_os_error()
            )));
        }
        Ok(file)
    }

    pub fn save_state(&self, state: &GenerationState) -> Result<()> {
        let state_path = self.base_dir.join("state.json");
        let tmp_path = self
            .base_dir
            .join(format!("state.json.tmp.{}", std::process::id()));
        let content =
            serde_json::to_string_pretty(state).map_err(|e| DotfieldError::State(e.to_string()))?;
        fs::write(&tmp_path, content)?;
        fs::rename(&tmp_path, &state_path)?;
        Ok(())
    }

    pub fn deploy(&self, profile: &Profile, fragments: &[CompiledFragment]) -> Result<u64> {
        let _lock = self.acquire_lock()?;
        let state = self.get_state()?.unwrap_or(GenerationState {
            current_generation: 0,
            active_profile: String::new(),
            timestamp: String::new(),
            history: Vec::new(),
        });

        // Compute strictly monotonic generation number to prevent collision
        let generations_dir = self.base_dir.join("generations");
        fs::create_dir_all(&generations_dir)?;
        let highest_existing = if let Ok(entries) = fs::read_dir(&generations_dir) {
            entries
                .filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().to_string_lossy().parse::<u64>().ok())
                .max()
                .unwrap_or(0)
        } else {
            0
        };

        let mut next_gen = std::cmp::max(state.current_generation, highest_existing) + 1;
        let gen_dir = loop {
            let candidate = generations_dir.join(next_gen.to_string());
            match fs::create_dir(&candidate) {
                Ok(()) => break candidate,
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => next_gen += 1,
                Err(e) => return Err(e.into()),
            }
        };
        let mut gen_guard = GenDirGuard {
            path: &gen_dir,
            armed: true,
        };

        // Write all compiled fragments into the new generation directory
        for frag in fragments {
            let target_file = gen_dir.join(&frag.target_path);
            if let Some(parent) = target_file.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(target_file, &frag.content)?;
        }

        // Save per-generation metadata
        let now_str = chrono_now();
        let gen_meta = GenerationMeta {
            generation: next_gen,
            profile_name: profile.meta.name.clone(),
            timestamp: now_str.clone(),
        };
        let meta_content = serde_json::to_string_pretty(&gen_meta)
            .map_err(|e| DotfieldError::State(e.to_string()))?;
        fs::write(gen_dir.join("meta.json"), meta_content)?;

        // Atomically update `current` symlink with compensation on binding failure
        let current_symlink = self.base_dir.join("current");
        let prior_target = if current_symlink.exists() {
            fs::read_link(&current_symlink).ok()
        } else {
            None
        };
        let tmp_symlink = self.base_dir.join(format!("current_tmp_{next_gen}"));

        if fs::symlink_metadata(&tmp_symlink).is_ok() {
            let _ = fs::remove_file(&tmp_symlink);
        }
        symlink(&gen_dir, &tmp_symlink)?;
        fs::rename(&tmp_symlink, &current_symlink)?;

        // Bind live targets in ~/.config so Sway/Niri/Mako consume the generated files
        if let Err(e) = self.bind_live_targets(fragments) {
            let mut compensation_errors = Vec::new();
            if let Some(target) = prior_target {
                if let Err(err) = symlink(&target, &tmp_symlink) {
                    compensation_errors
                        .push(format!("failed to symlink prior target {target:?}: {err}"));
                } else if let Err(err) = fs::rename(&tmp_symlink, &current_symlink) {
                    compensation_errors.push(format!(
                        "failed to restore current symlink to prior target: {err}"
                    ));
                }
                if state.current_generation > 0 {
                    match self.get_generation_fragments(state.current_generation) {
                        Ok(prior_frags) => {
                            if let Err(err) = self.bind_live_targets(&prior_frags) {
                                compensation_errors
                                    .push(format!("failed to restore prior live targets: {err}"));
                            }
                        }
                        Err(err) => {
                            compensation_errors
                                .push(format!("failed to load prior fragments: {err}"));
                        }
                    }
                }
            } else {
                if let Err(err) = self.bind_live_targets(&[]) {
                    compensation_errors.push(format!(
                        "failed to clean up partially bound live targets: {err}"
                    ));
                }
                if let Err(err) = fs::remove_file(&current_symlink) {
                    compensation_errors.push(format!("failed to remove current symlink: {err}"));
                }
            }

            if !compensation_errors.is_empty() {
                gen_guard.armed = false;
                return Err(DotfieldError::State(format!(
                    "Live target binding failed: {e}. Additionally, recovery compensation failed: {}",
                    compensation_errors.join("; ")
                )));
            }

            return Err(e);
        }

        // Save new generation state with updated history only after binding succeeds
        let mut history = state.history.clone();
        if state.current_generation > 0 {
            history.push(state.current_generation);
        }

        let new_state = GenerationState {
            current_generation: next_gen,
            active_profile: profile.meta.name.clone(),
            timestamp: now_str,
            history,
        };
        if let Err(e) = self.save_state(&new_state) {
            let mut compensation_errors = Vec::new();
            if let Some(target) = prior_target {
                if let Err(err) = symlink(&target, &tmp_symlink) {
                    compensation_errors
                        .push(format!("failed to symlink prior target {target:?}: {err}"));
                } else if let Err(err) = fs::rename(&tmp_symlink, &current_symlink) {
                    compensation_errors.push(format!(
                        "failed to restore current symlink to prior target: {err}"
                    ));
                }
                if state.current_generation > 0 {
                    match self.get_generation_fragments(state.current_generation) {
                        Ok(prior_frags) => {
                            if let Err(err) = self.bind_live_targets(&prior_frags) {
                                compensation_errors
                                    .push(format!("failed to restore prior live targets: {err}"));
                            }
                        }
                        Err(err) => {
                            compensation_errors
                                .push(format!("failed to load prior fragments: {err}"));
                        }
                    }
                }
            } else {
                if let Err(err) = self.bind_live_targets(&[]) {
                    compensation_errors.push(format!(
                        "failed to clean up partially bound live targets: {err}"
                    ));
                }
                if let Err(err) = fs::remove_file(&current_symlink) {
                    compensation_errors.push(format!("failed to remove current symlink: {err}"));
                }
            }

            if !compensation_errors.is_empty() {
                gen_guard.armed = false;
                return Err(DotfieldError::State(format!(
                    "State persistence failed: {e}. Additionally, recovery compensation failed: {}",
                    compensation_errors.join("; ")
                )));
            }

            return Err(e);
        }

        gen_guard.armed = false;
        Ok(next_gen)
    }

    pub fn rollback(&self) -> Result<u64> {
        let _lock = self.acquire_lock()?;
        let state = self
            .get_state()?
            .ok_or_else(|| DotfieldError::State("No current generation state found".to_string()))?;

        let mut history = state.history.clone();
        let prev_gen = if let Some(gen) = history.pop() {
            gen
        } else if state.current_generation > 1 {
            state.current_generation - 1
        } else {
            return Err(DotfieldError::State(
                "Cannot rollback: already at oldest recorded generation".to_string(),
            ));
        };

        let gen_dir = self.base_dir.join("generations").join(prev_gen.to_string());

        if !gen_dir.exists() {
            return Err(DotfieldError::State(format!(
                "Previous generation directory does not exist: {gen_dir:?}"
            )));
        }

        let current_symlink = self.base_dir.join("current");
        let prior_target = if current_symlink.exists() {
            fs::read_link(&current_symlink).ok()
        } else {
            None
        };
        let tmp_symlink = self.base_dir.join(format!("current_tmp_{prev_gen}"));

        if fs::symlink_metadata(&tmp_symlink).is_ok() {
            let _ = fs::remove_file(&tmp_symlink);
        }
        symlink(&gen_dir, &tmp_symlink)?;
        fs::rename(&tmp_symlink, &current_symlink)?;

        // Restore actual profile name from the previous generation's metadata
        let meta_file = gen_dir.join("meta.json");
        let profile_name = if meta_file.exists() {
            fs::read_to_string(&meta_file)
                .ok()
                .and_then(|c| serde_json::from_str::<GenerationMeta>(&c).ok())
                .map(|m| m.profile_name)
                .unwrap_or(state.active_profile)
        } else {
            state.active_profile
        };

        // Reconcile live symlinks to match the restored generation's fragments
        let restored_fragments = match self.get_generation_fragments(prev_gen) {
            Ok(frags) => frags,
            Err(e) => {
                let mut compensation_errors = Vec::new();
                if let Some(target) = prior_target {
                    if let Err(err) = symlink(&target, &tmp_symlink) {
                        compensation_errors
                            .push(format!("failed to symlink prior target {target:?}: {err}"));
                    } else if let Err(err) = fs::rename(&tmp_symlink, &current_symlink) {
                        compensation_errors
                            .push(format!("failed to restore current symlink: {err}"));
                    }
                }
                if !compensation_errors.is_empty() {
                    return Err(DotfieldError::State(format!(
                        "Failed to load fragments for rollback: {e}. Additionally, compensation failed: {}",
                        compensation_errors.join("; ")
                    )));
                }
                return Err(e);
            }
        };

        if let Err(e) = self.bind_live_targets(&restored_fragments) {
            let mut compensation_errors = Vec::new();
            if let Some(target) = prior_target {
                if let Err(err) = symlink(&target, &tmp_symlink) {
                    compensation_errors
                        .push(format!("failed to symlink prior target {target:?}: {err}"));
                } else if let Err(err) = fs::rename(&tmp_symlink, &current_symlink) {
                    compensation_errors.push(format!("failed to restore current symlink: {err}"));
                }
                if let Ok(prior_frags) = self.get_generation_fragments(state.current_generation) {
                    if let Err(err) = self.bind_live_targets(&prior_frags) {
                        compensation_errors
                            .push(format!("failed to restore prior live targets: {err}"));
                    }
                }
            } else {
                if let Err(err) = self.bind_live_targets(&[]) {
                    compensation_errors.push(format!(
                        "failed to clean up partially bound live targets: {err}"
                    ));
                }
                if let Err(err) = fs::remove_file(&current_symlink) {
                    compensation_errors.push(format!("failed to remove current symlink: {err}"));
                }
            }

            if !compensation_errors.is_empty() {
                return Err(DotfieldError::State(format!(
                    "Rollback live target binding failed: {e}. Additionally, recovery compensation failed: {}",
                    compensation_errors.join("; ")
                )));
            }

            return Err(e);
        }

        // Save state only after live targets are successfully reconciled
        let new_state = GenerationState {
            current_generation: prev_gen,
            active_profile: profile_name,
            timestamp: chrono_now(),
            history,
        };
        if let Err(e) = self.save_state(&new_state) {
            let mut compensation_errors = Vec::new();
            if let Some(target) = prior_target {
                if let Err(err) = symlink(&target, &tmp_symlink) {
                    compensation_errors
                        .push(format!("failed to symlink prior target {target:?}: {err}"));
                } else if let Err(err) = fs::rename(&tmp_symlink, &current_symlink) {
                    compensation_errors.push(format!(
                        "failed to restore current symlink to prior target: {err}"
                    ));
                }
                if let Ok(prior_frags) = self.get_generation_fragments(state.current_generation) {
                    if let Err(err) = self.bind_live_targets(&prior_frags) {
                        compensation_errors
                            .push(format!("failed to restore prior live targets: {err}"));
                    }
                }
            }
            if !compensation_errors.is_empty() {
                return Err(DotfieldError::State(format!(
                    "Rollback state persistence failed: {e}. Additionally, recovery compensation failed: {}",
                    compensation_errors.join("; ")
                )));
            }
            return Err(e);
        }

        Ok(prev_gen)
    }

    pub fn bind_live_targets(&self, fragments: &[CompiledFragment]) -> Result<()> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let config_dir = PathBuf::from(home).join(".config");
        let current_dir = self.base_dir.join("current");

        const KNOWN_TARGETS: &[(&str, &str, &str)] = &[
            (
                "sway/00-generated-bindings.conf",
                "sway/config.d",
                "00-generated-bindings.conf",
            ),
            (
                "niri/00-generated-bindings.kdl",
                "niri/config.kdl.d",
                "00-generated-bindings.kdl",
            ),
            ("fuzzel/colors.ini", "fuzzel", "colors.ini"),
            ("foot/colors.ini", "foot", "colors.ini"),
            ("kitty/colors.conf", "kitty", "colors.conf"),
            ("alacritty/colors.toml", "alacritty", "colors.toml"),
            (
                "ghostty/themes/dotfield-theme",
                "ghostty/themes",
                "dotfield-theme",
            ),
            ("mako/config", "mako", "config"),
            ("tmux/colors.conf", "tmux", "colors.conf"),
            (
                "zellij/themes/dotfield.kdl",
                "zellij/themes",
                "dotfield.kdl",
            ),
            (
                "nvim/lua/palette_generated.lua",
                "nvim/lua",
                "palette_generated.lua",
            ),
            (
                "nvim/lua/themes/dotfield.lua",
                "nvim/lua/themes",
                "dotfield.lua",
            ),
        ];

        let mut renamed_backups: Vec<(PathBuf, PathBuf)> = Vec::new();

        let bind_result: Result<()> = (|| {
            for frag in fragments {
                let Some(&(_, target_dir_rel, file_name)) = KNOWN_TARGETS
                    .iter()
                    .find(|(t, _, _)| *t == frag.target_path)
                else {
                    continue;
                };

                let live_dir = config_dir.join(target_dir_rel);

                // Self-heal any broken dangling symlinks in the path hierarchy
                let mut check_ancestor = live_dir.clone();
                while let Some(parent) = check_ancestor.parent() {
                    if let Ok(meta) = fs::symlink_metadata(&check_ancestor) {
                        if meta.file_type().is_symlink() && !check_ancestor.exists() {
                            let _ = fs::remove_file(&check_ancestor);
                        }
                    }
                    if parent == config_dir {
                        break;
                    }
                    check_ancestor = parent.to_path_buf();
                }

                fs::create_dir_all(&live_dir)?;
                let live_symlink = live_dir.join(file_name);
                let current_source = current_dir.join(&frag.target_path);

                if let Ok(meta) = fs::symlink_metadata(&live_symlink) {
                    if meta.file_type().is_symlink() && !live_symlink.exists() {
                        let _ = fs::remove_file(&live_symlink);
                    } else {
                        let is_our_symlink = meta.file_type().is_symlink()
                            && fs::read_link(&live_symlink)
                                .map(|dest| dest.starts_with(&current_dir))
                                .unwrap_or(false);

                        if is_our_symlink {
                            fs::remove_file(&live_symlink)?;
                        } else {
                            let mut backup = live_symlink.with_extension("dotfield-bak");
                            if backup.exists() {
                                let ts = chrono_now();
                                let mut candidate =
                                    live_symlink.with_extension(format!("dotfield-bak.{ts}"));
                                let mut counter = 1;
                                while candidate.exists() {
                                    candidate = live_symlink
                                        .with_extension(format!("dotfield-bak.{ts}.{counter}"));
                                    counter += 1;
                                }
                                backup = candidate;
                            }
                            fs::rename(&live_symlink, &backup)?;
                            renamed_backups.push((backup, live_symlink.clone()));
                        }
                    }
                }
                symlink(&current_source, &live_symlink)?;
            }

            // Remove stale live symlinks for targets dropped in this generation (only if owned by Dotfield)
            for (target_path, dir_rel, file_name) in KNOWN_TARGETS {
                if fragments.iter().any(|f| f.target_path == *target_path) {
                    continue;
                }
                let live = config_dir.join(dir_rel).join(file_name);
                let is_our_symlink = fs::symlink_metadata(&live)
                    .map(|m| m.file_type().is_symlink())
                    .unwrap_or(false)
                    && fs::read_link(&live)
                        .map(|dest| dest.starts_with(&current_dir))
                        .unwrap_or(false);

                if is_our_symlink {
                    fs::remove_file(&live)?;
                    // If removing an owned Niri fragment, restore backup or write a valid placeholder
                    // so Niri's config parser does not fail on its required include.
                    if *target_path == "niri/00-generated-bindings.kdl" {
                        let backup = live.with_extension("dotfield-bak");
                        if backup.exists() {
                            let _ = fs::rename(&backup, &live);
                        } else {
                            let _ = fs::write(&live, "// Dotfield fallback placeholder\n");
                        }
                    }
                }
            }
            Ok(())
        })();

        if let Err(e) = bind_result {
            // Restore any backed-up files if binding failed
            for (backup, original) in renamed_backups.into_iter().rev() {
                if fs::symlink_metadata(&original).is_ok() {
                    let _ = fs::remove_file(&original);
                }
                let _ = fs::rename(&backup, &original);
            }
            return Err(e);
        }

        Ok(())
    }

    pub fn get_generation_fragments(&self, generation: u64) -> Result<Vec<CompiledFragment>> {
        let gen_dir = self
            .base_dir
            .join("generations")
            .join(generation.to_string());
        if !gen_dir.exists() {
            return Err(DotfieldError::State(format!(
                "Generation directory does not exist: {gen_dir:?}"
            )));
        }

        let mut fragments = Vec::new();
        self.collect_fragments(&gen_dir, &gen_dir, &mut fragments)?;
        Ok(fragments)
    }

    fn collect_fragments(
        &self,
        root: &Path,
        current: &Path,
        fragments: &mut Vec<CompiledFragment>,
    ) -> Result<()> {
        if current.is_dir() {
            for entry in fs::read_dir(current)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    self.collect_fragments(root, &path, fragments)?;
                } else if path.file_name().and_then(|n| n.to_str()) != Some("meta.json") {
                    if let Ok(rel_path) = path.strip_prefix(root) {
                        let content = fs::read_to_string(&path).unwrap_or_default();
                        fragments.push(CompiledFragment {
                            target_path: rel_path.to_string_lossy().to_string(),
                            content,
                        });
                    }
                }
            }
        }
        Ok(())
    }
}

fn chrono_now() -> String {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}_epoch", duration.as_secs())
}
