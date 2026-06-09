use crate::types::{PathEntry, PathVar};
use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::Path;

/// Parse a shell config file and extract PATH/MANPATH modifications
pub fn parse_shell_file(path: &Path) -> Result<Vec<PathEntry>> {
    let content = fs::read_to_string(path)?;
    let mut entries = Vec::new();

    // Regex patterns for PATH/MANPATH assignments
    // Matches: PATH=..., export PATH=..., PATH="...", etc.
    // We'll match the assignment and handle quotes in post-processing
    let path_assign = Regex::new(
        r"(?m)^(?:export\s+)?(PATH|MANPATH)=(.+)$"
    )?;

    // Matches zsh array syntax: path+=(...) or path=(...)
    let zsh_array_append = Regex::new(
        r"(?m)^(?:path|manpath)\+=\(([^)]+)\)"
    )?;

    let zsh_array_assign = Regex::new(
        r"(?m)^(?:path|manpath)=\(([^)]+)\)"
    )?;

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();

        // Skip comments
        if line.starts_with('#') {
            continue;
        }

        // Check for PATH/MANPATH assignments
        if let Some(caps) = path_assign.captures(line) {
            let var_name = caps.get(1).unwrap().as_str();
            let mut value = caps.get(2).unwrap().as_str();
            
            // Strip surrounding quotes if present
            if (value.starts_with('"') && value.ends_with('"')) ||
               (value.starts_with('\'') && value.ends_with('\'')) {
                value = &value[1..value.len()-1];
            }

            let variable = if var_name == "PATH" {
                PathVar::Path
            } else {
                PathVar::Manpath
            };

            // Extract new path components (those not in $PATH/$MANPATH)
            let new_paths = extract_new_paths(value, var_name);

            for new_path in new_paths {
                entries.push(PathEntry {
                    path: new_path,
                    variable,
                    source_file: path.to_path_buf(),
                    line_number: Some((line_num + 1) as u32),
                    order: 0, // Will be set later
                    comment: generate_comment(path, line_num + 1),
                });
            }
        }

        // Check for zsh array append: path+=(...)
        if let Some(caps) = zsh_array_append.captures(line) {
            let value = caps.get(1).unwrap().as_str();
            let variable = if line.starts_with("path") {
                PathVar::Path
            } else {
                PathVar::Manpath
            };

            // Parse array elements
            let new_paths = parse_array_elements(value);

            for new_path in new_paths {
                entries.push(PathEntry {
                    path: new_path,
                    variable,
                    source_file: path.to_path_buf(),
                    line_number: Some((line_num + 1) as u32),
                    order: 0,
                    comment: generate_comment(path, line_num + 1),
                });
            }
        }

        // Check for zsh array assign: path=(...)
        if let Some(caps) = zsh_array_assign.captures(line) {
            let value = caps.get(1).unwrap().as_str();
            let variable = if line.starts_with("path") {
                PathVar::Path
            } else {
                PathVar::Manpath
            };

            // Parse array elements, filtering out $path/$manpath references
            let elements = parse_array_elements(value);
            let new_paths: Vec<String> = elements
                .into_iter()
                .filter(|e| !e.contains("$path") && !e.contains("$manpath"))
                .collect();

            for new_path in new_paths {
                entries.push(PathEntry {
                    path: new_path,
                    variable,
                    source_file: path.to_path_buf(),
                    line_number: Some((line_num + 1) as u32),
                    order: 0,
                    comment: generate_comment(path, line_num + 1),
                });
            }
        }
    }

    Ok(entries)
}

/// Extract new path components from an assignment value
/// e.g., "/new/path:$PATH" -> ["/new/path"]
/// e.g., "$PATH:/new/path" -> ["/new/path"]
fn extract_new_paths(value: &str, var_name: &str) -> Vec<String> {
    let var_ref = format!("${}", var_name);
    let var_ref_braced = format!("${{{}}}", var_name);

    // Split by ':' and filter out references to the variable itself
    value
        .split(':')
        .filter(|component| {
            let c = component.trim();
            !c.is_empty()
                && c != var_ref
                && c != var_ref_braced
                && !c.contains(&var_ref)
                && !c.contains(&var_ref_braced)
        })
        .map(|s| s.trim().to_string())
        .collect()
}

/// Parse zsh array elements: "elem1 elem2 elem3" -> ["elem1", "elem2", "elem3"]
fn parse_array_elements(value: &str) -> Vec<String> {
    value
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Generate a comment describing the path's origin
fn generate_comment(source: &Path, line_num: usize) -> String {
    let filename = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    format!("{}:{}", filename, line_num)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_extract_new_paths() {
        assert_eq!(
            extract_new_paths("/usr/local/bin:$PATH", "PATH"),
            vec!["/usr/local/bin"]
        );

        assert_eq!(
            extract_new_paths("$PATH:/usr/local/bin", "PATH"),
            vec!["/usr/local/bin"]
        );

        assert_eq!(
            extract_new_paths("/opt/bin:$PATH:/usr/local/bin", "PATH"),
            vec!["/opt/bin", "/usr/local/bin"]
        );

        assert_eq!(
            extract_new_paths("${PATH}:/home/user/bin", "PATH"),
            vec!["/home/user/bin"]
        );
    }

    #[test]
    fn test_parse_simple_path() -> Result<()> {
        let mut file = NamedTempFile::new()?;
        writeln!(file, "export PATH=/usr/local/bin:$PATH")?;

        let entries = parse_shell_file(file.path())?;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "/usr/local/bin");
        assert_eq!(entries[0].variable, PathVar::Path);

        Ok(())
    }

    #[test]
    fn test_parse_multiple_paths() -> Result<()> {
        let mut file = NamedTempFile::new()?;
        writeln!(file, "PATH=/opt/bin:/usr/local/bin:$PATH")?;

        let entries = parse_shell_file(file.path())?;
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].path, "/opt/bin");
        assert_eq!(entries[1].path, "/usr/local/bin");

        Ok(())
    }

    #[test]
    fn test_parse_manpath() -> Result<()> {
        let mut file = NamedTempFile::new()?;
        writeln!(file, "export MANPATH=/usr/local/man:$MANPATH")?;

        let entries = parse_shell_file(file.path())?;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "/usr/local/man");
        assert_eq!(entries[0].variable, PathVar::Manpath);

        Ok(())
    }
}
