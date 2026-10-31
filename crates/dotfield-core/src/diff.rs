use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::Path;

use crate::compiler::CompiledFragment;
use crate::error::Result;

pub struct FileDiff {
    pub path: String,
    pub diff_text: String,
}

pub struct DiffEngine;

impl DiffEngine {
    pub fn diff_fragments<P: AsRef<Path>>(
        current_dir: Option<P>,
        fragments: &[CompiledFragment],
    ) -> Result<Vec<FileDiff>> {
        let mut diffs = Vec::new();

        for frag in fragments {
            let old_content = match &current_dir {
                Some(dir) => {
                    let file_path = dir.as_ref().join(&frag.target_path);
                    if file_path.exists() {
                        fs::read_to_string(file_path)?
                    } else {
                        String::new()
                    }
                }
                None => String::new(),
            };

            if old_content == frag.content {
                continue;
            }

            let diff = TextDiff::from_lines(&old_content, &frag.content);
            let mut diff_output = String::new();
            let mut has_changes = false;

            for change in diff.iter_all_changes() {
                let sign = match change.tag() {
                    ChangeTag::Delete => {
                        has_changes = true;
                        "-"
                    }
                    ChangeTag::Insert => {
                        has_changes = true;
                        "+"
                    }
                    ChangeTag::Equal => " ",
                };
                diff_output.push_str(&format!("{sign} {change}"));
                if change.missing_newline() {
                    diff_output.push_str("\n\\ No newline at end of file\n");
                }
            }

            if has_changes {
                diffs.push(FileDiff {
                    path: frag.target_path.clone(),
                    diff_text: diff_output,
                });
            }
        }

        // Show fragments present in current generation that the candidate profile removes
        if let Some(dir) = &current_dir {
            let root = dir.as_ref();
            if root.exists() {
                for entry in walkdir::WalkDir::new(root).into_iter().flatten() {
                    if !entry.file_type().is_file() || entry.file_name() == "meta.json" {
                        continue;
                    }
                    let Ok(rel) = entry.path().strip_prefix(root) else {
                        continue;
                    };
                    let rel = rel.to_string_lossy().to_string();
                    if fragments.iter().any(|f| f.target_path == rel) {
                        continue;
                    }
                    let old = fs::read_to_string(entry.path()).unwrap_or_default();
                    let diff_text: String = old.lines().map(|l| format!("- {l}\n")).collect();
                    diffs.push(FileDiff {
                        path: rel,
                        diff_text,
                    });
                }
            }
        }

        Ok(diffs)
    }
}
