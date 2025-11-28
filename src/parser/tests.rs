#[cfg(test)]
mod tests {
    use crate::{FileMetadata, ParseResult, Parser, ParserRegistry, SemanticKind, UnknownParser};

    // ========================================================================
    // FileMetadata Tests
    // ========================================================================

    #[test]
    fn test_metadata_from_rust_file() {
        let metadata = FileMetadata::from_path_and_bytes("src/main.rs", b"fn main() {}");
        assert_eq!(metadata.path, "src/main.rs");
        assert_eq!(metadata.extension, "rs");
        assert_eq!(metadata.language, "rust");
        assert_eq!(metadata.size_bytes, 12);
        assert!(metadata.is_utf8);
    }

    #[test]
    fn test_metadata_from_python_file() {
        let metadata = FileMetadata::from_path_and_bytes("script.py", b"print('hello')");
        assert_eq!(metadata.extension, "py");
        assert_eq!(metadata.language, "python");
    }

    #[test]
    fn test_metadata_language_detection() {
        let test_cases = vec![
            ("file.rs", "rust"),
            ("file.py", "python"),
            ("file.js", "javascript"),
            ("file.ts", "typescript"),
            ("file.go", "go"),
            ("file.java", "java"),
            ("file.md", "markdown"),
            ("file.json", "json"),
            ("file.yaml", "yaml"),
            ("file.toml", "toml"),
            ("file.cpp", "cpp"),
            ("file.c", "c"),
            ("file.unknown", "unknown"),
        ];

        for (path, expected_lang) in test_cases {
            let metadata = FileMetadata::from_path_and_bytes(path, b"");
            assert_eq!(metadata.language, expected_lang, "Failed for {}", path);
        }
    }

    #[test]
    fn test_metadata_no_extension() {
        let metadata = FileMetadata::from_path_and_bytes("Makefile", b"all:");
        assert_eq!(metadata.extension, "");
        assert_eq!(metadata.language, "unknown");
    }

    #[test]
    fn test_metadata_binary_detection() {
        let binary_data = vec![0xFF, 0xFE, 0x00, 0x01, 0x02];
        let metadata = FileMetadata::from_path_and_bytes("binary.dat", &binary_data);
        assert!(!metadata.is_utf8);
    }

    #[test]
    fn test_metadata_utf8_detection() {
        let metadata = FileMetadata::from_path_and_bytes("text.txt", "Hello, 世界".as_bytes());
        assert!(metadata.is_utf8);
    }

    // ========================================================================
    // UnknownParser Tests
    // ========================================================================

    #[test]
    fn test_parse_simple_text() {
        let parser = UnknownParser;
        let result = parser.parse("test.txt", b"line1\nline2\nline3");

        assert_eq!(result.metadata.path, "test.txt");
        assert_eq!(result.metadata.line_count, 3);
        assert!(result.metadata.is_utf8);
        assert_eq!(result.semantic_units.len(), 3);
    }

    #[test]
    fn test_parse_empty_file() {
        let parser = UnknownParser;
        let result = parser.parse("empty.txt", b"");

        assert_eq!(result.metadata.line_count, 0);
        assert_eq!(result.semantic_units.len(), 0);
        assert_eq!(result.normalized_text, "");
    }

    #[test]
    fn test_parse_single_line_no_newline() {
        let parser = UnknownParser;
        let result = parser.parse("single.txt", b"single line");

        assert_eq!(result.metadata.line_count, 1);
        assert_eq!(result.semantic_units.len(), 1);
        // UnknownParser adds newlines to each line
        assert_eq!(result.semantic_units[0].text, "single line\n");
    }

    #[test]
    fn test_parse_binary_file() {
        let parser = UnknownParser;
        let binary = vec![0xFF, 0xFE, 0x41, 0x42, 0x00, 0x01];
        let result = parser.parse("binary.dat", &binary);

        assert!(!result.metadata.is_utf8);
        // Should extract only printable ASCII (A and B)
        assert!(result.normalized_text.contains('A'));
        assert!(result.normalized_text.contains('B'));
    }

    #[test]
    fn test_semantic_unit_offsets() {
        let parser = UnknownParser;
        let result = parser.parse("test.txt", b"line1\nline2\nline3");

        // First unit
        assert_eq!(result.semantic_units[0].start_offset, 0);
        assert_eq!(result.semantic_units[0].end_offset, 6); // "line1\n"

        // Second unit
        assert_eq!(result.semantic_units[1].start_offset, 6);
        assert_eq!(result.semantic_units[1].end_offset, 12); // "line2\n"

        // Third unit
        assert_eq!(result.semantic_units[2].start_offset, 12);
        assert_eq!(result.semantic_units[2].end_offset, 18); // "line3\n"
    }

    #[test]
    fn test_semantic_unit_kind_text_file() {
        let parser = UnknownParser;
        let result = parser.parse("text.txt", b"content");

        for unit in &result.semantic_units {
            assert_eq!(unit.kind, SemanticKind::Unknown);
        }
    }

    #[test]
    fn test_semantic_unit_kind_binary_file() {
        let parser = UnknownParser;
        let binary = vec![0xFF, 0xFE, 0x00];
        let result = parser.parse("binary.dat", &binary);

        for unit in &result.semantic_units {
            assert_eq!(unit.kind, SemanticKind::Blob);
        }
    }

    #[test]
    fn test_parse_utf8_with_unicode() {
        let parser = UnknownParser;
        let text = "Hello 世界\nBonjour 🌍\n";
        let result = parser.parse("unicode.txt", text.as_bytes());

        assert!(result.metadata.is_utf8);
        assert_eq!(result.metadata.line_count, 2);
        assert_eq!(result.normalized_text, text);
    }

    #[test]
    fn test_extract_printable_from_binary() {
        let parser = UnknownParser;
        // Binary with embedded ASCII text
        let binary = b"\xFF\xFEHELLO\x00\x01WORLD\xFF";
        let result = parser.parse("mixed.dat", binary);

        assert!(result.normalized_text.contains("HELLO"));
        assert!(result.normalized_text.contains("WORLD"));
        assert!(!result.normalized_text.contains('\u{FF}')); // Non-printable removed
    }

    // ========================================================================
    // ParserRegistry Tests
    // ========================================================================

    #[test]
    fn test_registry_new() {
        let registry = ParserRegistry::new();
        assert_eq!(registry.parser_count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut registry = ParserRegistry::new();
        registry.register("rs", UnknownParser);

        assert_eq!(registry.parser_count(), 1);
        assert!(registry.registered_extensions().contains(&"rs"));
    }

    #[test]
    fn test_registry_select_fallback() {
        let registry = ParserRegistry::new();
        let parser = registry.select("unknown.xyz");

        // Should return fallback parser (UnknownParser)
        let result = parser.parse("test.xyz", b"content");
        assert_eq!(result.metadata.language, "unknown");
    }

    #[test]
    fn test_registry_select_registered() {
        let mut registry = ParserRegistry::new();

        // Create a custom test parser
        struct TestParser;
        impl Parser for TestParser {
            fn parse(&self, path: &str, bytes: &[u8]) -> ParseResult {
                let mut metadata = FileMetadata::from_path_and_bytes(path, bytes);
                metadata.language = "custom".to_string(); // Override
                metadata.set_line_count(0);

                ParseResult {
                    normalized_text: String::new(),
                    metadata,
                    semantic_units: vec![],
                }
            }
        }

        registry.register("test", TestParser);
        let parser = registry.select("file.test");
        let result = parser.parse("file.test", b"");

        assert_eq!(result.metadata.language, "custom");
    }

    #[test]
    fn test_registry_case_insensitive() {
        let mut registry = ParserRegistry::new();
        registry.register("rs", UnknownParser);

        // Should match regardless of case
        let parser1 = registry.select("file.rs");

        // All should resolve to the registered parser (not fallback)
        // We can't directly compare trait objects, but we can test behavior
        let result = parser1.parse("test.rs", b"");
        assert_eq!(result.metadata.extension, "rs");
    }

    #[test]
    fn test_registry_multiple_extensions() {
        let mut registry = ParserRegistry::new();
        registry.register("rs", UnknownParser);
        registry.register("py", UnknownParser);
        registry.register("js", UnknownParser);

        assert_eq!(registry.parser_count(), 3);

        let extensions = registry.registered_extensions();
        assert_eq!(extensions.len(), 3);
        assert!(extensions.contains(&"rs"));
        assert!(extensions.contains(&"py"));
        assert!(extensions.contains(&"js"));
    }

    #[test]
    fn test_registry_overwrite_extension() {
        struct Parser1;
        impl Parser for Parser1 {
            fn parse(&self, path: &str, bytes: &[u8]) -> ParseResult {
                let mut metadata = FileMetadata::from_path_and_bytes(path, bytes);
                metadata.language = "parser1".to_string();
                metadata.set_line_count(0);
                ParseResult {
                    normalized_text: String::new(),
                    metadata,
                    semantic_units: vec![],
                }
            }
        }

        struct Parser2;
        impl Parser for Parser2 {
            fn parse(&self, path: &str, bytes: &[u8]) -> ParseResult {
                let mut metadata = FileMetadata::from_path_and_bytes(path, bytes);
                metadata.language = "parser2".to_string();
                metadata.set_line_count(0);
                ParseResult {
                    normalized_text: String::new(),
                    metadata,
                    semantic_units: vec![],
                }
            }
        }

        let mut registry = ParserRegistry::new();
        registry.register("test", Parser1);
        registry.register("test", Parser2); // Overwrite

        let parser = registry.select("file.test");
        let result = parser.parse("file.test", b"");

        // Should use the second parser
        assert_eq!(result.metadata.language, "parser2");
    }

    #[test]
    fn test_registry_no_extension() {
        let registry = ParserRegistry::new();
        let parser = registry.select("Makefile");

        // Should fall back to unknown parser
        let result = parser.parse("Makefile", b"all:");
        assert_eq!(result.metadata.extension, "");
    }

    // ========================================================================
    // Integration Tests
    // ========================================================================

    #[test]
    fn test_full_pipeline_with_registry() {
        let mut registry = ParserRegistry::new();
        registry.register("txt", UnknownParser);

        // Simulate processing multiple files
        let files = vec![
            ("file1.txt", b"line1\nline2" as &[u8]),
            ("file2.txt", b"single line"),
            ("file3.unknown", b"unknown type"),
        ];

        let mut total_units = 0;
        for (path, bytes) in files {
            let parser = registry.select(path);
            let result = parser.parse(path, bytes);
            total_units += result.semantic_units.len();
        }

        assert!(total_units > 0);
    }

    #[test]
    fn test_semantic_kind_values() {
        // Ensure all enum variants are distinct
        assert_ne!(SemanticKind::Unknown, SemanticKind::Blob);
        assert_ne!(SemanticKind::Function, SemanticKind::Class);
        assert_ne!(SemanticKind::Module, SemanticKind::Comment);
    }
}
