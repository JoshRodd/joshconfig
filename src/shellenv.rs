// Shared module used by both `joshconfig shellenv` and `joshconfig-env` binary.
// Emits `VARIABLE=value` / `export VARIABLE` shell commands for PATH and MANPATH
// based on entries in ~/.paths.d and ~/.manpaths.d.


/// Read first non-empty, non-comment line from a path-entry file.
/// Returns `None` if the file is empty, all-comment, or unreadable.
fn read_path_entry(path: &std::path::Path, home: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Strip inline comment: first `#` preceded by whitespace
        let no_comment = if let Some(pos) = trimmed.find('#') {
            // Only strip if there's a whitespace before `#`
            let before = &trimmed[..pos];
            if before.ends_with(char::is_whitespace) {
                before.trim_end().to_string()
            } else {
                trimmed.to_string()
            }
        } else {
            trimmed.to_string()
        };

        if no_comment.is_empty() {
            continue;
        }

        // Strip surrounding quotes (both ends must match)
        let unquoted = if (no_comment.starts_with('"') && no_comment.ends_with('"'))
            || (no_comment.starts_with('\'') && no_comment.ends_with('\''))
        {
            if no_comment.len() >= 2 {
                no_comment[1..no_comment.len() - 1].to_string()
            } else {
                no_comment
            }
        } else {
            no_comment
        };

        if unquoted.is_empty() {
            continue;
        }

        // Expand ~ and $HOME
        let expanded = if unquoted.starts_with("~/") || unquoted == "~" {
            format!("{}{}", home, &unquoted[1..])
        } else if unquoted.contains("$HOME") {
            unquoted.replace("$HOME", home)
        } else {
            unquoted
        };

        return Some(expanded);
    }

    None
}

/// Collect new paths from a `.d` directory, sorted by filename, deduplicated.
fn collect_new_paths(dir: &std::path::Path, home: &str) -> Vec<String> {
    if !dir.is_dir() {
        return Vec::new();
    }

    let mut entries: Vec<_> = match std::fs::read_dir(dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
            .collect(),
        Err(_) => return Vec::new(),
    };

    entries.sort_by_key(|e| e.file_name());

    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();

    for entry in entries {
        if let Some(path) = read_path_entry(&entry.path(), home) {
            if seen.insert(path.clone()) {
                result.push(path);
            }
        }
    }

    result
}

/// Build combined value: new entries first, then existing, deduplicated across the union.
fn combine_paths(new_entries: &[String], existing: &str, _name: &str) -> Option<String> {
    let mut seen = std::collections::HashSet::new();
    let mut parts: Vec<&str> = Vec::new();

    // Process in reverse order so higher-numbered (last-sorted) files end
    // up first, matching the old shell loader's prepend behavior.
    for p in new_entries.iter().rev() {
        if seen.insert(p.as_str()) {
            parts.push(p);
        }
    }

    // Then existing entries, split on `:`
    if !existing.is_empty() {
        for p in existing.split(':') {
            if !p.is_empty() && seen.insert(p) {
                parts.push(p);
            }
        }
    }

    if parts.is_empty() {
        return None;
    }

    Some(parts.join(":"))
}

/// Emit `VARIABLE=value` / `export VARIABLE` for each variable whose value changed.
/// Returns 0 on success, 1 on error.
pub fn emit_shell_env() -> i32 {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => {
            eprintln!("joshconfig: HOME not set");
            return 1;
        }
    };

    let pairs = [
        (
            std::path::PathBuf::from(&home).join(".paths.d"),
            "PATH",
        ),
        (
            std::path::PathBuf::from(&home).join(".manpaths.d"),
            "MANPATH",
        ),
    ];

    let mut any_output = false;

    for (dir, var_name) in &pairs {
        let new_entries = collect_new_paths(dir, &home);

        if new_entries.is_empty() {
            continue;
        }

        let existing = std::env::var(var_name).unwrap_or_default();

        // If existing is empty and no new entries, skip
        // (collect_new_paths already returned empty if no entries, so we have entries here)

        let combined = combine_paths(&new_entries, &existing, var_name);

        if let Some(value) = combined {
            let old_value = std::env::var(var_name).ok();
            if old_value.as_deref() != Some(&value) {
                println!("{}={}", var_name, value);
                println!("export {}", var_name);
                any_output = true;
            }
        }
    }

    if any_output {
        0
    } else {
        // Not an error — just nothing to emit
        0
    }
}
