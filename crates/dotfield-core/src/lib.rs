pub mod compiler;
pub mod diff;
pub mod error;
pub mod generation;
pub mod profile;
pub mod reloader;
pub mod validator;

pub use compiler::Compiler;
pub use diff::DiffEngine;
pub use error::{DotfieldError, Result};
pub use generation::GenerationManager;
pub use profile::Profile;
pub use reloader::Reloader;
pub use validator::Validator;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_parsing() {
        let toml_data = r#"
[meta]
name = "test"
description = "A test profile"
version = "1.0.0"

[compositor]
engine = "sway"

[apps]
terminal = "foot"
launcher = "fuzzel"

[theme]
palette = "controlled-chaos"

[bindings]
terminal = "Super+Return"
launcher = "Super+space"
"#;
        let profile: Profile = toml_data.parse().expect("Must parse test profile");
        assert_eq!(profile.meta.name, "test");
        assert_eq!(profile.apps.terminal, "foot");
        assert_eq!(profile.bindings.get("terminal").unwrap(), "Super+Return");
    }

    #[test]
    fn test_compiler_output() {
        let toml_data = r#"
[meta]
name = "test"

[compositor]
engine = "sway"

[apps]
terminal = "foot"
launcher = "fuzzel"

[theme]
palette = "controlled-chaos"

[bindings]
terminal = "Super+Return"
"#;
        let profile: Profile = toml_data.parse().expect("Must parse");
        let fragments = Compiler::compile(&profile).expect("Must compile fragments");
        assert!(!fragments.is_empty());
        let sway_frag = fragments
            .iter()
            .find(|f| f.target_path.contains("sway"))
            .expect("Must have sway fragment");
        assert!(sway_frag.content.contains("bindsym Mod4+Return exec foot"));
    }
}
