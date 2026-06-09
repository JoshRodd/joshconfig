use joshconfig::parser;
use joshconfig::types::{PathEntry, PathVar};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_parse_simple_zshrc() {
    let fixture = PathBuf::from("tests/fixtures/simple.zshrc");
    let entries = parser::parse_shell_file(&fixture).unwrap();

    // Should find 2 PATH entries and 1 MANPATH entry
    let path_entries: Vec<_> = entries.iter().filter(|e| e.variable == PathVar::Path).collect();
    let manpath_entries: Vec<_> = entries.iter().filter(|e| e.variable == PathVar::Manpath).collect();

    assert_eq!(path_entries.len(), 2);
    assert_eq!(manpath_entries.len(), 1);

    // Check specific paths
    assert!(path_entries.iter().any(|e| e.path == "/usr/local/bin"));
    assert!(path_entries.iter().any(|e| e.path == "/opt/homebrew/bin"));
    assert!(manpath_entries.iter().any(|e| e.path == "/usr/local/share/man"));
}

#[test]
fn test_parse_complex_bashrc() {
    let fixture = PathBuf::from("tests/fixtures/complex.bashrc");
    let entries = parser::parse_shell_file(&fixture).unwrap();

    // Should find multiple PATH entries
    assert!(entries.len() >= 5);

    // Check for specific paths
    assert!(entries.iter().any(|e| e.path == "/usr/local/bin"));
    assert!(entries.iter().any(|e| e.path == "/usr/local/sbin"));
    assert!(entries.iter().any(|e| e.path == "/opt/bin"));
}

#[test]
fn test_parse_tilde_expansion() {
    let fixture = PathBuf::from("tests/fixtures/tilde.zshrc");
    let entries = parser::parse_shell_file(&fixture).unwrap();

    // Should find paths with tilde
    assert!(entries.iter().any(|e| e.path == "~/bin"));
    assert!(entries.iter().any(|e| e.path == "$HOME/.cargo/bin"));
    assert!(entries.iter().any(|e| e.path == "~/man"));

    // Check normalization
    let tilde_entry = entries.iter().find(|e| e.path == "~/bin").unwrap();
    assert_eq!(tilde_entry.normalized_path(), "$HOME/bin");
}

#[test]
fn test_parse_zsh_arrays() {
    let fixture = PathBuf::from("tests/fixtures/zsh_arrays.zshrc");
    let entries = parser::parse_shell_file(&fixture).unwrap();

    // Should find array-based path additions
    assert!(entries.iter().any(|e| e.path == "/usr/local/bin"));
    assert!(entries.iter().any(|e| e.path == "/opt/bin"));
    assert!(entries.iter().any(|e| e.path == "/usr/local/share/man"));
}

#[test]
fn test_path_entry_filename() {
    let entry = PathEntry {
        path: "/usr/local/bin".to_string(),
        variable: PathVar::Path,
        source_file: PathBuf::from("/home/user/.zshrc"),
        line_number: Some(10),
        order: 1,
        comment: "Local binaries".to_string(),
    };

    assert_eq!(entry.filename(), "01-usr-local-bin");
}

#[test]
fn test_path_entry_filename_with_home() {
    let entry = PathEntry {
        path: "$HOME/.local/bin".to_string(),
        variable: PathVar::Path,
        source_file: PathBuf::from("/home/user/.zshrc"),
        line_number: Some(10),
        order: 2,
        comment: "User local".to_string(),
    };

    assert_eq!(entry.filename(), "02-home-.local-bin");
}

#[test]
fn test_path_entry_file_content() {
    let entry = PathEntry {
        path: "/usr/local/bin".to_string(),
        variable: PathVar::Path,
        source_file: PathBuf::from("/home/user/.zshrc"),
        line_number: Some(10),
        order: 1,
        comment: "Local binaries".to_string(),
    };

    assert_eq!(entry.file_content(), "/usr/local/bin # Local binaries\n");
}

#[test]
fn test_path_entry_file_content_with_tilde() {
    let entry = PathEntry {
        path: "~/bin".to_string(),
        variable: PathVar::Path,
        source_file: PathBuf::from("/home/user/.zshrc"),
        line_number: Some(10),
        order: 1,
        comment: "User bin".to_string(),
    };

    assert_eq!(entry.file_content(), "$HOME/bin # User bin\n");
}

#[test]
fn test_generator_creates_files() {
    let temp_dir = TempDir::new().unwrap();
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

    joshconfig::generator::generate_path_files(&entries, home).unwrap();

    let paths_dir = home.join(".paths.d");
    let manpaths_dir = home.join(".manpaths.d");

    assert!(paths_dir.exists());
    assert!(manpaths_dir.exists());

    // Check PATH file
    let path_file = paths_dir.join("01-usr-local-bin");
    assert!(path_file.exists());
    let content = std::fs::read_to_string(&path_file).unwrap();
    assert_eq!(content, "/usr/local/bin # Local binaries\n");

    // Check MANPATH file
    let manpath_file = manpaths_dir.join("02-usr-share-man");
    assert!(manpath_file.exists());
    let content = std::fs::read_to_string(&manpath_file).unwrap();
    assert_eq!(content, "/usr/share/man # System man pages\n");
}

#[test]
fn test_generator_skips_path_helper_entries() {
    let temp_dir = TempDir::new().unwrap();
    let home = temp_dir.path();

    let entries = vec![
        PathEntry {
            path: "/usr/local/bin".to_string(),
            variable: PathVar::Path,
            source_file: PathBuf::from("/etc/paths.d/homebrew"),
            line_number: None,
            order: 1,
            comment: "Homebrew".to_string(),
        },
        PathEntry {
            path: "/opt/bin".to_string(),
            variable: PathVar::Path,
            source_file: PathBuf::from("/home/user/.zshrc"),
            line_number: Some(10),
            order: 2,
            comment: "Opt".to_string(),
        },
    ];

    joshconfig::generator::generate_path_files(&entries, home).unwrap();

    let paths_dir = home.join(".paths.d");

    // Should only have the .zshrc entry, not the /etc/paths.d entry
    let files: Vec<_> = std::fs::read_dir(&paths_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();

    assert_eq!(files.len(), 1);
    assert!(files[0].file_name().to_string_lossy().contains("opt-bin"));
}
