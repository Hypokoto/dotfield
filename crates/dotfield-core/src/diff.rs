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
                        fs::read_to_string(file_path).unwrap_or_default()
                    } else {
                        String::new()
                    }
                }
                None => String::new(),
            };

            let diff = TextDiff::from_lines(&old_content, &frag.content);
            let mut diff_output = String::new();

            for change in diff.iter_all_changes() {
                let sign = match change.tag() {
                    ChangeTag::Delete => "-",
                    ChangeTag::Insert => "+",
                    ChangeTag::Equal => " ",
                };
                diff_output.push_str(&format!("{sign} {change}"));
            }

            diffs.push(FileDiff {
                path: frag.target_path.clone(),
                diff_text: diff_output,
            });
        }

        Ok(diffs)
    }
}
