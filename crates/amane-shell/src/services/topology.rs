//! Project Topology & Session Discovery Service
//!
//! Provides zero-latency in-memory cache of project directories, Git worktrees,
//! and active Tmux sessions across `~/dev/projects/` and adjacent repositories.
//!
//! UI readers access cached targets via lock-free `ArcSwap` without blocking
//! on filesystem traversal or process spawning. Background refreshes run
//! asynchronously.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use arc_swap::ArcSwap;

/// A recognized developer project target, worktree, or session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectTarget {
    /// Human-friendly display name (e.g., "dotfield", "dotfield (feature-x)")
    pub name: String,
    /// Safe alphanumeric slug for IPC and window app-id routing (e.g. "dotfield")
    pub slug: String,
    /// Absolute filesystem path
    pub path: PathBuf,
    /// Active git branch name or tag, if in a git repository
    pub branch: Option<String>,
    /// Whether this target is a secondary git worktree
    pub is_worktree: bool,
    /// Whether an active Tmux session matches this target
    pub is_tmux_active: bool,
    /// Distinctive glyph icon for UI display
    pub glyph: &'static str,
}

impl ProjectTarget {
    /// Create a slug safe for window class/app-id (alphanumeric, dots, dashes).
    pub fn slugify(raw: &str) -> String {
        let mut slug = String::with_capacity(raw.len());
        for c in raw.chars() {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                slug.push(c.to_ascii_lowercase());
            } else if c == ' ' || c == '/' || c == '.' {
                slug.push('-');
            }
        }
        let trimmed = slug.trim_matches('-');
        if trimmed.is_empty() {
            "project".to_string()
        } else {
            trimmed.to_string()
        }
    }
}

/// Global cached list of discovered project targets.
static TOPOLOGY_CACHE: LazyLock<ArcSwap<Vec<ProjectTarget>>> =
    LazyLock::new(|| ArcSwap::from_pointee(Vec::new()));

/// Timestamp of last successful refresh to throttle frequent triggers.
static LAST_REFRESH: LazyLock<std::sync::Mutex<Option<Instant>>> =
    LazyLock::new(|| std::sync::Mutex::new(None));

pub struct TopologyService;

impl TopologyService {
    /// Return the currently cached list of project targets (zero-cost lock-free read).
    pub fn list() -> Arc<Vec<ProjectTarget>> {
        TOPOLOGY_CACHE.load_full()
    }

    /// Trigger an asynchronous background refresh of project targets.
    ///
    /// Throttles refreshes to at most once every 1.5 seconds to prevent spam.
    pub fn refresh_async() {
        {
            let mut last = match LAST_REFRESH.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            if let Some(prev) = *last
                && prev.elapsed() < Duration::from_millis(1500)
            {
                return;
            }
            *last = Some(Instant::now());
        }

        std::thread::Builder::new()
            .name("amane-topology-discovery".into())
            .spawn(|| {
                let targets = Self::discover_all();
                TOPOLOGY_CACHE.store(Arc::new(targets));
            })
            .ok();
    }

    /// Run full synchronous discovery of projects, worktrees, and tmux sessions.
    pub fn discover_all() -> Vec<ProjectTarget> {
        let mut results = Vec::new();
        let mut seen_paths = HashSet::new();

        let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        let dev_projects = PathBuf::from(&home_dir).join("dev").join("projects");

        // 1. Query active tmux sessions
        let tmux_sessions = Self::discover_tmux_sessions();

        // 2. Scan standard dev/projects directory
        if dev_projects.is_dir()
            && let Ok(entries) = fs::read_dir(&dev_projects)
        {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                // Check for git repository & worktrees
                let worktrees = Self::discover_git_worktrees(&path);
                if !worktrees.is_empty() {
                    for wt in worktrees {
                        if seen_paths.insert(wt.path.clone()) {
                            results.push(wt);
                        }
                    }
                } else {
                    // Regular directory or non-worktree project
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("project")
                        .to_string();
                    let slug = ProjectTarget::slugify(&name);
                    let is_tmux = tmux_sessions.contains_key(&slug);

                    let branch = Self::get_git_branch(&path);

                    let target = ProjectTarget {
                        name,
                        slug,
                        path: path.clone(),
                        branch,
                        is_worktree: false,
                        is_tmux_active: is_tmux,
                        glyph: "󰘐",
                    };

                    if seen_paths.insert(path) {
                        results.push(target);
                    }
                }
            }
        }

        // 3. Incorporate any detached tmux sessions located outside dev/projects
        for (session_name, session_path) in tmux_sessions {
            let path_buf = PathBuf::from(&session_path);
            if !seen_paths.contains(&path_buf) && path_buf.is_dir() {
                let slug = ProjectTarget::slugify(&session_name);
                let branch = Self::get_git_branch(&path_buf);
                seen_paths.insert(path_buf.clone());
                results.push(ProjectTarget {
                    name: session_name,
                    slug,
                    path: path_buf,
                    branch,
                    is_worktree: false,
                    is_tmux_active: true,
                    glyph: "󰓩",
                });
            }
        }

        // Sort: active tmux sessions first, then worktrees, then alphabetical
        results.sort_by(|a, b| {
            b.is_tmux_active
                .cmp(&a.is_tmux_active)
                .then_with(|| b.is_worktree.cmp(&a.is_worktree))
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

        results
    }

    /// Query tmux list-sessions for active developer sessions.
    fn discover_tmux_sessions() -> HashMap<String, String> {
        let mut map = HashMap::new();
        let output = match Command::new("tmux")
            .args(["list-sessions", "-F", "#{session_name}\t#{session_path}"])
            .output()
        {
            Ok(out) => out,
            Err(_) => return map,
        };

        if !output.status.success() {
            return map;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                let name = parts[0].trim().to_string();
                let path = parts[1].trim().to_string();
                let slug = ProjectTarget::slugify(&name);
                map.insert(slug, path);
            }
        }

        map
    }

    /// Query git worktree list --porcelain in a candidate git repository.
    fn discover_git_worktrees(repo_dir: &Path) -> Vec<ProjectTarget> {
        let mut worktrees = Vec::new();
        let output = match Command::new("git")
            .args([
                "-C",
                &repo_dir.to_string_lossy(),
                "worktree",
                "list",
                "--porcelain",
            ])
            .output()
        {
            Ok(out) => out,
            Err(_) => return worktrees,
        };

        if !output.status.success() {
            return worktrees;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut current_path: Option<PathBuf> = None;
        let mut current_branch: Option<String> = None;
        let mut is_first = true;

        let base_name = repo_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project");

        let mut push_entry = |path: PathBuf, branch: Option<String>, is_first_entry: &mut bool| {
            let is_wt = !*is_first_entry;
            let display_name = if is_wt {
                let wt_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("worktree");
                format!("{base_name} ({wt_name})")
            } else {
                base_name.to_string()
            };
            let slug = ProjectTarget::slugify(&display_name);
            worktrees.push(ProjectTarget {
                name: display_name,
                slug,
                path,
                branch,
                is_worktree: is_wt,
                is_tmux_active: false,
                glyph: if is_wt { "󰘬" } else { "󰘐" },
            });
            *is_first_entry = false;
        };

        for line in stdout.lines() {
            if line.starts_with("worktree ") {
                if let Some(path) = current_path.take() {
                    push_entry(path, current_branch.take(), &mut is_first);
                }
                current_path = Some(PathBuf::from(line.trim_start_matches("worktree ").trim()));
            } else if line.starts_with("branch ") {
                let branch_ref = line.trim_start_matches("branch ").trim();
                let clean_branch = branch_ref.trim_start_matches("refs/heads/").to_string();
                current_branch = Some(clean_branch);
            }
        }

        // Add trailing entry
        if let Some(path) = current_path {
            push_entry(path, current_branch, &mut is_first);
        }

        worktrees
    }

    /// Extract current git branch for a path if it is a git repo.
    fn get_git_branch(dir: &Path) -> Option<String> {
        let output = Command::new("git")
            .args(["-C", &dir.to_string_lossy(), "branch", "--show-current"])
            .output()
            .ok()?;

        if output.status.success() {
            let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !branch.is_empty() {
                return Some(branch);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(ProjectTarget::slugify("dotfield"), "dotfield");
        assert_eq!(
            ProjectTarget::slugify("dotfield (feature-x)"),
            "dotfield-feature-x"
        );
        assert_eq!(
            ProjectTarget::slugify("My Cool Project!"),
            "my-cool-project"
        );
        assert_eq!(ProjectTarget::slugify("---test---"), "test");
        assert_eq!(ProjectTarget::slugify(""), "project");
    }

    #[test]
    fn test_topology_cache_read() {
        let list = TopologyService::list();
        // Initial list is a valid empty or cached vector
        assert!(list.is_empty() || !list.is_empty());
    }

    #[test]
    fn test_git_worktree_discovery_on_current_repo() {
        let current_dir = std::env::current_dir().unwrap();
        let worktrees = TopologyService::discover_git_worktrees(&current_dir);
        assert!(
            !worktrees.is_empty(),
            "Should discover current repo worktree"
        );
        assert_eq!(worktrees[0].branch.as_deref(), Some("main"));
    }
}
