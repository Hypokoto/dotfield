use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use crate::compiler::CompiledFragment;
use crate::error::{DotfieldError, Result};
use crate::profile::Profile;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GenerationState {
    pub current_generation: u64,
    pub active_profile: String,
    pub timestamp: String,
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
        if !state_path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(state_path)?;
        let state: GenerationState =
            serde_json::from_str(&content).map_err(|e| DotfieldError::State(e.to_string()))?;
        Ok(Some(state))
    }

    pub fn save_state(&self, state: &GenerationState) -> Result<()> {
        let state_path = self.base_dir.join("state.json");
        let content =
            serde_json::to_string_pretty(state).map_err(|e| DotfieldError::State(e.to_string()))?;
        fs::write(state_path, content)?;
        Ok(())
    }

    pub fn deploy(&self, profile: &Profile, fragments: &[CompiledFragment]) -> Result<u64> {
        let state = self.get_state()?.unwrap_or(GenerationState {
            current_generation: 0,
            active_profile: String::new(),
            timestamp: String::new(),
        });

        let next_gen = state.current_generation + 1;
        let gen_dir = self.base_dir.join("generations").join(next_gen.to_string());
        fs::create_dir_all(&gen_dir)?;

        // Write all compiled fragments into the new generation directory
        for frag in fragments {
            let target_file = gen_dir.join(&frag.target_path);
            if let Some(parent) = target_file.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(target_file, &frag.content)?;
        }

        // Atomically update `current` symlink
        let current_symlink = self.base_dir.join("current");
        let tmp_symlink = self.base_dir.join(format!("current_tmp_{next_gen}"));

        if tmp_symlink.exists() {
            let _ = fs::remove_file(&tmp_symlink);
        }
        symlink(&gen_dir, &tmp_symlink)?;
        fs::rename(&tmp_symlink, &current_symlink)?;

        // Save new generation state
        let new_state = GenerationState {
            current_generation: next_gen,
            active_profile: profile.meta.name.clone(),
            timestamp: chrono_now(),
        };
        self.save_state(&new_state)?;

        Ok(next_gen)
    }

    pub fn rollback(&self) -> Result<u64> {
        let state = self
            .get_state()?
            .ok_or_else(|| DotfieldError::State("No current generation state found".to_string()))?;

        if state.current_generation <= 1 {
            return Err(DotfieldError::State(
                "Cannot rollback: already at oldest recorded generation".to_string(),
            ));
        }

        let prev_gen = state.current_generation - 1;
        let gen_dir = self.base_dir.join("generations").join(prev_gen.to_string());

        if !gen_dir.exists() {
            return Err(DotfieldError::State(format!(
                "Previous generation directory does not exist: {gen_dir:?}"
            )));
        }

        let current_symlink = self.base_dir.join("current");
        let tmp_symlink = self.base_dir.join(format!("current_tmp_{prev_gen}"));

        if tmp_symlink.exists() {
            let _ = fs::remove_file(&tmp_symlink);
        }
        symlink(&gen_dir, &tmp_symlink)?;
        fs::rename(&tmp_symlink, &current_symlink)?;

        let new_state = GenerationState {
            current_generation: prev_gen,
            active_profile: state.active_profile,
            timestamp: chrono_now(),
        };
        self.save_state(&new_state)?;

        Ok(prev_gen)
    }
}

fn chrono_now() -> String {
    // Generate a simple timestamp
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}_epoch", duration.as_secs())
}
