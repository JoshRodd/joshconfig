use crate::types::{PathEntry, PathVar, ShellType};
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Instrument a shell to capture PATH/MANPATH changes from config files
pub fn analyze_shell(shell: ShellType) -> Result<Vec<PathEntry>> {
    let config_files = shell.config_files();
    let mut all_entries = Vec::new();
    let mut global_order = 0;

    for config_file in config_files {
        if !config_file.exists() {
            continue;
        }

        let entries = analyze_single_file(shell, &config_file, &mut global_order)?;
        all_entries.extend(entries);
    }

    Ok(all_entries)
}

/// Analyze a single config file by sourcing it in a shell
fn analyze_single_file(
    shell: ShellType,
    config_file: &Path,
    global_order: &mut usize,
) -> Result<Vec<PathEntry>> {
    // Create a wrapper script that:
    // 1. Captures initial PATH/MANPATH
    // 2. Sources the config file
    // 3. Captures final PATH/MANPATH
    let wrapper_script = format!(
        r#"
echo "===PATH_BEFORE==="
echo "$PATH"
echo "===MANPATH_BEFORE==="
echo "$MANPATH"
echo "===SOURCE_START==="
. "{}" 2>/dev/null || true
echo "===SOURCE_END==="
echo "===PATH_AFTER==="
echo "$PATH"
echo "===MANPATH_AFTER==="
echo "$MANPATH"
"#,
        config_file.display()
    );

    let output = Command::new(shell.binary())
        .arg("--norc")
        .arg("--noprofile")
        .arg("-c")
        .arg(&wrapper_script)
        .output()
        .with_context(|| format!("Failed to run {}", shell.binary()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let entries = parse_instrumentation_output(&stdout, config_file, global_order)?;

    Ok(entries)
}

/// Parse the output from shell instrumentation
fn parse_instrumentation_output(
    output: &str,
    source_file: &Path,
    global_order: &mut usize,
) -> Result<Vec<PathEntry>> {
    let mut entries = Vec::new();

    let path_before = extract_section(output, "===PATH_BEFORE===", "===MANPATH_BEFORE===");
    let manpath_before = extract_section(output, "===MANPATH_BEFORE===", "===SOURCE_START===");
    let path_after = extract_section(output, "===PATH_AFTER===", "===MANPATH_AFTER===");
    let manpath_after = extract_section(output, "===MANPATH_AFTER===", "");

    // Find new PATH entries
    if let (Some(before), Some(after)) = (path_before, path_after) {
        let new_paths = diff_paths(&before, &after);
        for path in new_paths {
            *global_order += 1;
            entries.push(PathEntry {
                path,
                variable: PathVar::Path,
                source_file: source_file.to_path_buf(),
                line_number: None,
                order: *global_order,
                comment: format!(
                    "Added by {}",
                    source_file.file_name().and_then(|n| n.to_str()).unwrap_or("config")
                ),
            });
        }
    }

    // Find new MANPATH entries
    if let (Some(before), Some(after)) = (manpath_before, manpath_after) {
        let new_paths = diff_paths(&before, &after);
        for path in new_paths {
            *global_order += 1;
            entries.push(PathEntry {
                path,
                variable: PathVar::Manpath,
                source_file: source_file.to_path_buf(),
                line_number: None,
                order: *global_order,
                comment: format!(
                    "Added by {}",
                    source_file.file_name().and_then(|n| n.to_str()).unwrap_or("config")
                ),
            });
        }
    }

    Ok(entries)
}

/// Extract a section from the instrumentation output
fn extract_section<'a>(output: &'a str, start_marker: &str, end_marker: &str) -> Option<String> {
    let start_idx = output.find(start_marker)?;
    let start = start_idx + start_marker.len();

    let end = if end_marker.is_empty() {
        output.len()
    } else {
        output[start..].find(end_marker)? + start
    };

    Some(output[start..end].trim().to_string())
}

/// Compute the difference between two PATH strings
/// Returns paths that are in `after` but not in `before`
fn diff_paths(before: &str, after: &str) -> Vec<String> {
    let before_paths: Vec<&str> = before.split(':').filter(|s| !s.is_empty()).collect();
    let after_paths: Vec<&str> = after.split(':').filter(|s| !s.is_empty()).collect();

    after_paths
        .into_iter()
        .filter(|p| !before_paths.contains(p))
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_paths() {
        let before = "/usr/bin:/bin";
        let after = "/usr/local/bin:/usr/bin:/bin:/opt/bin";

        let new = diff_paths(before, after);
        assert_eq!(new, vec!["/usr/local/bin", "/opt/bin"]);
    }

    #[test]
    fn test_diff_paths_no_change() {
        let before = "/usr/bin:/bin";
        let after = "/usr/bin:/bin";

        let new = diff_paths(before, after);
        assert!(new.is_empty());
    }

    #[test]
    fn test_extract_section() {
        let output = r#"
===PATH_BEFORE===
/usr/bin:/bin
===MANPATH_BEFORE===
/usr/share/man
===SOURCE_START===
===SOURCE_END===
===PATH_AFTER===
/usr/local/bin:/usr/bin:/bin
===MANPATH_AFTER===
/usr/share/man
"#;

        let path_before = extract_section(output, "===PATH_BEFORE===", "===MANPATH_BEFORE===");
        assert_eq!(path_before, Some("/usr/bin:/bin".to_string()));

        let path_after = extract_section(output, "===PATH_AFTER===", "===MANPATH_AFTER===");
        assert_eq!(path_after, Some("/usr/local/bin:/usr/bin:/bin".to_string()));
    }
}
