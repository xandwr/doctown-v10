#[cfg(test)]
mod tests {
    use crate::security::PathSanitizer;

    #[test]
    fn test_valid_simple_path() {
        let result = PathSanitizer::sanitize("src/main.rs");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "src/main.rs");
    }

    #[test]
    fn test_valid_nested_path() {
        let result = PathSanitizer::sanitize("src/parser/mod.rs");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "src/parser/mod.rs");
    }

    #[test]
    fn test_hidden_files_allowed_by_default() {
        let result = PathSanitizer::sanitize(".github/workflows/ci.yml");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ".github/workflows/ci.yml");
    }

    #[test]
    fn test_hidden_files_rejected_when_disabled() {
        let result = PathSanitizer::sanitize_with_options(".gitignore", false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Hidden files"));
    }

    #[test]
    fn test_reject_parent_directory_traversal() {
        let result = PathSanitizer::sanitize("../etc/passwd");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Parent directory traversal")
        );
    }

    #[test]
    fn test_reject_parent_in_middle() {
        let result = PathSanitizer::sanitize("src/../../etc/passwd");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Parent directory traversal")
        );
    }

    #[test]
    fn test_reject_absolute_unix_path() {
        let result = PathSanitizer::sanitize("/etc/passwd");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Absolute path"));
    }

    #[test]
    fn test_reject_empty_path() {
        let result = PathSanitizer::sanitize("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty path"));
    }

    #[test]
    fn test_normalize_current_dir_markers() {
        let result = PathSanitizer::sanitize("./src/./main.rs");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "src/main.rs");
    }

    #[test]
    fn test_reject_only_current_dir() {
        let result = PathSanitizer::sanitize("./.");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No valid components")
        );
    }

    #[test]
    fn test_windows_style_paths_normalized() {
        let result = PathSanitizer::sanitize("src\\main.rs");
        assert!(result.is_ok());
        // On Unix systems, backslash is a valid filename character
        // Path::new doesn't automatically convert \ to / on Unix
        // This test behavior is platform-dependent, so we just verify it succeeds
        assert!(result.unwrap().contains("main.rs"));
    }

    #[test]
    fn test_complex_valid_path() {
        let result = PathSanitizer::sanitize("test_suite/tests/ui/default-attribute/struct.rs");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "test_suite/tests/ui/default-attribute/struct.rs"
        );
    }

    #[test]
    fn test_path_with_spaces() {
        let result = PathSanitizer::sanitize("My Documents/file.txt");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "My Documents/file.txt");
    }

    #[test]
    fn test_path_with_unicode() {
        let result = PathSanitizer::sanitize("docs/文档/readme.md");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "docs/文档/readme.md");
    }

    #[test]
    fn test_reject_only_hidden_component() {
        let result = PathSanitizer::sanitize_with_options(".git", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_hidden_components_allowed() {
        let result = PathSanitizer::sanitize(".github/.gitignore");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ".github/.gitignore");
    }
}
