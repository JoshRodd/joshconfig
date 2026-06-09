use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Which environment variable is being modified
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PathVar {
    Path,
    Manpath,
}


/// A single path entry extracted from shell config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathEntry {
    /// The path value (may contain $HOME, ~, etc.)
    pub path: String,
    /// Which variable (PATH or MANPATH)
    pub variable: PathVar,
    /// Source file where this was defined
    pub source_file: PathBuf,
    /// Line number in source file (if known)
    pub line_number: Option<u32>,
    /// Order in which this appears (for numbering)
    pub order: usize,
    /// Human-readable comment describing the path
    pub comment: String,
}

impl PathEntry {
    /// Normalize path: convert ~ to $HOME
    pub fn normalized_path(&self) -> String {
        self.path.replace('~', "$HOME")
    }

    /// Generate filename for .paths.d entry
    pub fn filename(&self) -> String {
        // Create a slug from the path
        let slug = self
            .path
            .replace("$HOME", "home")
            .replace('~', "home")
            .replace('/', "-")
            .trim_matches('-')
            .to_string();

        let slug = if slug.is_empty() {
            "entry".to_string()
        } else {
            slug
        };

        format!("{:02}-{}", self.order, slug)
    }

    /// Generate file content for .paths.d entry
    pub fn file_content(&self) -> String {
        format!("{} # {}\n", self.normalized_path(), self.comment)
    }
}


/// Shell type for instrumentation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellType {
    Bash,
    Zsh,
}

impl ShellType {
    pub fn binary(&self) -> &'static str {
        match self {
            ShellType::Bash => "bash",
            ShellType::Zsh => "zsh",
        }
    }

    pub fn config_files(&self) -> Vec<PathBuf> {
        let home = dirs::home_dir().expect("Could not determine home directory");

        match self {
            ShellType::Bash => vec![
                PathBuf::from("/etc/profile"),
                PathBuf::from("/etc/bash_profile"),
                PathBuf::from("/etc/bashrc"),
                home.join(".bash_profile"),
                home.join(".bashrc"),
                home.join(".profile"),
            ],
            ShellType::Zsh => vec![
                PathBuf::from("/etc/zprofile"),
                PathBuf::from("/etc/zshrc"),
                home.join(".zprofile"),
                home.join(".zshrc"),
            ],
        }
    }
}
