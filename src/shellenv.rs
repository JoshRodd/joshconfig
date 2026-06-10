// Shared module used by both `joshconfig shellenv` and `joshconfig-env` binary.
// Emits `VARIABLE=value` / `export VARIABLE` shell commands for PATH, MANPATH, and INFOPATH
// based on system config (/etc/paths, /etc/paths.d, etc.) and user config (~/.paths.d, etc.).



/// Collect new paths from a `.d` directory, sorted by filename, deduplicated.
/// Reads all non-comment, non-empty lines from each file (some files like
/// /etc/paths.d/10-cryptex contain multiple paths).
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
        // Read all non-comment lines from the file (not just the first)
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            for line in content.lines() {
                if let Some(path) = read_path_line(line, home) {
                    if seen.insert(path.clone()) {
                        result.push(path);
                    }
                }
            }
        }
    }

    result
}

/// Parse a single line as a path entry (handles comments, quotes, ~ and $HOME).
fn read_path_line(line: &str, home: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }

    // Strip inline comment: first `#` preceded by whitespace
    let no_comment = if let Some(pos) = trimmed.find('#') {
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
        return None;
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
        return None;
    }

    // Expand ~ and $HOME
    let expanded = if unquoted.starts_with("~/") || unquoted == "~" {
        format!("{}{}", home, &unquoted[1..])
    } else if unquoted.contains("$HOME") {
        unquoted.replace("$HOME", home)
    } else {
        unquoted
    };

    Some(expanded)
}

/// Read paths from a plain file (one path per line, like /etc/paths).
/// Skips empty lines, comment lines, and inline comments.
fn collect_paths_from_file(file: &std::path::Path) -> Vec<String> {
    let content = match std::fs::read_to_string(file) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut result = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // Strip inline comment (only when preceded by whitespace)
        let path = if let Some(pos) = trimmed.find('#') {
            let before = &trimmed[..pos];
            if before.ends_with(char::is_whitespace) {
                before.trim_end().to_string()
            } else {
                trimmed.to_string()
            }
        } else {
            trimmed.to_string()
        };
        if !path.is_empty() {
            result.push(path);
        }
    }
    result
}

/// Emit shell commands for one variable, collecting from system and user sources.
/// Returns true if any output was emitted.
fn emit_one_var(
    sys_base: Option<&std::path::Path>,
    sys_d_dir: Option<&std::path::Path>,
    user_d_dir: &std::path::Path,
    var_name: &str,
    home: &str,
) -> bool {
    let user_paths = collect_new_paths(user_d_dir, home);
    let sys_d_paths = sys_d_dir
        .map(|d| collect_new_paths(d, home))
        .unwrap_or_default();
    let sys_base_paths = sys_base
        .map(|f| collect_paths_from_file(f))
        .unwrap_or_default();

    let existing = std::env::var(var_name).unwrap_or_default();

    // Build final list: user first (highest priority), then system .d, then system base,
    // then existing remainder. Within each .d group, reverse to match old shell loader
    // behavior (higher-numbered files end up first). System base file preserves file order.
    let mut seen = std::collections::HashSet::new();
    let mut parts: Vec<String> = Vec::new();

    for p in user_paths.iter().rev() {
        if seen.insert(p.clone()) {
            parts.push(p.clone());
        }
    }
    for p in sys_d_paths.iter().rev() {
        if seen.insert(p.clone()) {
            parts.push(p.clone());
        }
    }
    for p in &sys_base_paths {
        if seen.insert(p.clone()) {
            parts.push(p.clone());
        }
    }
    for p in existing.split(':') {
        if !p.is_empty() && seen.insert(p.to_string()) {
            parts.push(p.to_string());
        }
    }

    let combined = parts.join(":");

    let old_value = std::env::var(var_name).ok();
    if old_value.as_deref() == Some(&combined) {
        return false;
    }

    println!("{}={}", var_name, combined);
    println!("export {}", var_name);
    true
}

/// Emit `VARIABLE=value` / `export VARIABLE` for PATH, MANPATH, and INFOPATH.
/// Reads system paths from /etc/paths, /etc/paths.d, /etc/manpaths, /etc/manpaths.d
/// and user paths from ~/.paths.d, ~/.manpaths.d, ~/.infopaths.d.
/// Returns 0 on success, 1 on error.
pub fn emit_shell_env() -> i32 {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => {
            eprintln!("joshconfig: HOME not set");
            return 1;
        }
    };

    let vars: &[(
        Option<&str>, // system base file (e.g. /etc/paths)
        Option<&str>, // system .d directory (e.g. /etc/paths.d)
        &str,         // user .d directory (e.g. .paths.d)
        &str,         // env var name (e.g. PATH)
    )] = &[
        (
            Some("/etc/paths"),
            Some("/etc/paths.d"),
            ".paths.d",
            "PATH",
        ),
        (
            Some("/etc/manpaths"),
            Some("/etc/manpaths.d"),
            ".manpaths.d",
            "MANPATH",
        ),
        (
            None,                // no system base file for INFOPATH
            None,                // no system .d directory for INFOPATH
            ".infopaths.d",
            "INFOPATH",
        ),
    ];

    let mut any_output = false;

    for (sys_base, sys_d_dir, user_d_dir, var_name) in vars {
        let sys_base_path = sys_base.map(std::path::Path::new);
        let sys_d_path = sys_d_dir.map(std::path::Path::new);
        let user_d_path = std::path::PathBuf::from(&home).join(user_d_dir);

        if emit_one_var(sys_base_path, sys_d_path, &user_d_path, var_name, &home) {
            any_output = true;
        }
    }

    if any_output {
        0
    } else {
        0
    }
}
