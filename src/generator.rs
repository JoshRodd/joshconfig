use crate::types::{PathEntry, PathVar};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Generate .paths.d and .manpaths.d directories with numbered files
pub fn generate_path_files(entries: &[PathEntry], home: &Path) -> Result<()> {
    let paths_dir = home.join(".paths.d");
    let manpaths_dir = home.join(".manpaths.d");

    // Create directories if they don't exist
    fs::create_dir_all(&paths_dir)
        .with_context(|| format!("Failed to create {}", paths_dir.display()))?;
    fs::create_dir_all(&manpaths_dir)
        .with_context(|| format!("Failed to create {}", manpaths_dir.display()))?;

    // Filter out entries from /etc/paths.d and /etc/manpaths.d
    let filtered_entries: Vec<&PathEntry> = entries
        .iter()
        .filter(|e| !is_path_helper_entry(e))
        .collect();

    // Separate PATH and MANPATH entries
    let path_entries: Vec<&PathEntry> = filtered_entries
        .iter()
        .filter(|e| e.variable == PathVar::Path)
        .copied()
        .collect();

    let manpath_entries: Vec<&PathEntry> = filtered_entries
        .iter()
        .filter(|e| e.variable == PathVar::Manpath)
        .copied()
        .collect();

    // Generate PATH files
    for entry in path_entries {
        let filename = entry.filename();
        let file_path = paths_dir.join(&filename);
        let content = entry.file_content();

        fs::write(&file_path, content)
            .with_context(|| format!("Failed to write {}", file_path.display()))?;
    }

    // Generate MANPATH files
    for entry in manpath_entries {
        let filename = entry.filename();
        let file_path = manpaths_dir.join(&filename);
        let content = entry.file_content();

        fs::write(&file_path, content)
            .with_context(|| format!("Failed to write {}", file_path.display()))?;
    }

    Ok(())
}

/// Check if an entry comes from /etc/paths.d or /etc/manpaths.d
/// These are handled by path_helper and should not be duplicated
fn is_path_helper_entry(entry: &PathEntry) -> bool {
    let source_str = entry.source_file.to_string_lossy();
    source_str.contains("/etc/paths.d") || source_str.contains("/etc/manpaths.d")
}

/// Clean up old files in .paths.d/.manpaths.d that are no longer needed
pub fn cleanup_old_entries(home: &Path, current_entries: &[PathEntry]) -> Result<()> {
    let paths_dir = home.join(".paths.d");
    let manpaths_dir = home.join(".manpaths.d");

    cleanup_dir(&paths_dir, current_entries, PathVar::Path)?;
    cleanup_dir(&manpaths_dir, current_entries, PathVar::Manpath)?;

    Ok(())
}

/// Clean up old files in a single directory
fn cleanup_dir(dir: &Path, current_entries: &[PathEntry], variable: PathVar) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    // Get set of current filenames
    let current_filenames: Vec<String> = current_entries
        .iter()
        .filter(|e| e.variable == variable)
        .map(|e| e.filename())
        .collect();

    // Read directory and remove files not in current set
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if !current_filenames.contains(&filename.to_string()) {
                    fs::remove_file(&path).with_context(|| {
                        format!("Failed to remove old entry {}", path.display())
                    })?;
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PathVar;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_generate_path_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let home = temp_dir.path();

        let entries = vec![
            PathEntry {
                path: "/usr/local/bin".to_string(),
                variable: PathVar::Path,
                source_file: PathBuf::from("/home/user/.zshrc"),
                line_number: Some(10),
                order: 1,
                comment: "Local binaries".to_string(),
            },
            PathEntry {
                path: "/usr/share/man".to_string(),
                variable: PathVar::Manpath,
                source_file: PathBuf::from("/home/user/.zshrc"),
                line_number: Some(15),
                order: 2,
                comment: "System man pages".to_string(),
            },
        ];

        generate_path_files(&entries, home)?;

        let paths_dir = home.join(".paths.d");
        let manpaths_dir = home.join(".manpaths.d");

        assert!(paths_dir.exists());
        assert!(manpaths_dir.exists());

        let path_files: Vec<_> = fs::read_dir(&paths_dir)?.collect();
        assert_eq!(path_files.len(), 1);

        let manpath_files: Vec<_> = fs::read_dir(&manpaths_dir)?.collect();
        assert_eq!(manpath_files.len(), 1);

        Ok(())
    }

    #[test]
    fn test_is_path_helper_entry() {
        let entry1 = PathEntry {
            path: "/usr/local/bin".to_string(),
            variable: PathVar::Path,
            source_file: PathBuf::from("/etc/paths.d/homebrew"),
            line_number: None,
            order: 1,
            comment: "Homebrew".to_string(),
        };

        let entry2 = PathEntry {
            path: "/usr/local/bin".to_string(),
            variable: PathVar::Path,
            source_file: PathBuf::from("/home/user/.zshrc"),
            line_number: Some(10),
            order: 1,
            comment: "Local".to_string(),
        };

        assert!(is_path_helper_entry(&entry1));
        assert!(!is_path_helper_entry(&entry2));
    }

    #[test]
    fn test_tilde_normalization() {
        let entry = PathEntry {
            path: "~/bin".to_string(),
            variable: PathVar::Path,
            source_file: PathBuf::from("/home/user/.zshrc"),
            line_number: Some(10),
            order: 1,
            comment: "User bin".to_string(),
        };

        assert_eq!(entry.normalized_path(), "$HOME/bin");
    }
}
