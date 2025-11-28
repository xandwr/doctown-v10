#[cfg(test)]
mod tests {
    use crate::SandboxBuilder;

    #[test]
    fn test_sandbox_builder_new() {
        let builder = SandboxBuilder::new();
        assert_eq!(builder.arena.len(), 0);
        assert_eq!(builder.index.len(), 0);
    }

    #[test]
    fn test_add_single_file() {
        let mut builder = SandboxBuilder::new();
        let result = builder.add_file("test.txt", b"hello world");
        assert!(result.is_ok());

        let sandbox = builder.build();
        assert_eq!(sandbox.file_count(), 1);
        assert_eq!(sandbox.total_size(), 11);
    }

    #[test]
    fn test_add_multiple_files() {
        let mut builder = SandboxBuilder::new();
        builder.add_file("file1.txt", b"content1").unwrap();
        builder.add_file("file2.txt", b"content2").unwrap();
        builder.add_file("file3.txt", b"content3").unwrap();

        let sandbox = builder.build();
        assert_eq!(sandbox.file_count(), 3);
        assert_eq!(sandbox.total_size(), 24); // 8 + 8 + 8
    }

    #[test]
    fn test_get_file_zero_copy() {
        let mut builder = SandboxBuilder::new();
        builder.add_file("test.txt", b"hello world").unwrap();
        let sandbox = builder.build();

        let content = sandbox.get("test.txt");
        assert!(content.is_some());
        assert_eq!(content.unwrap(), b"hello world");
    }

    #[test]
    fn test_get_nonexistent_file() {
        let sandbox = SandboxBuilder::new().build();
        let content = sandbox.get("nonexistent.txt");
        assert!(content.is_none());
    }

    #[test]
    fn test_arena_concatenation() {
        let mut builder = SandboxBuilder::new();
        builder.add_file("a.txt", b"AAA").unwrap();
        builder.add_file("b.txt", b"BBB").unwrap();
        builder.add_file("c.txt", b"CCC").unwrap();

        let sandbox = builder.build();

        // Files should be concatenated in arena
        assert_eq!(sandbox.get("a.txt").unwrap(), b"AAA");
        assert_eq!(sandbox.get("b.txt").unwrap(), b"BBB");
        assert_eq!(sandbox.get("c.txt").unwrap(), b"CCC");

        // Arena should contain all data
        assert_eq!(sandbox.total_size(), 9);
    }

    #[test]
    fn test_file_entry_metadata() {
        let mut builder = SandboxBuilder::new();
        builder.add_file("test.txt", b"hello").unwrap();
        let sandbox = builder.build();

        let entry = sandbox.get_entry("test.txt");
        assert!(entry.is_some());

        let entry = entry.unwrap();
        assert_eq!(entry.virtual_path, "test.txt");
        assert_eq!(entry.length, 5);
        assert_eq!(entry.offset, 0);
    }

    #[test]
    fn test_file_size_limit() {
        let mut builder = SandboxBuilder::new().max_file_size(10);

        let result = builder.add_file("small.txt", b"tiny");
        assert!(result.is_ok());

        let result = builder.add_file("large.txt", b"this is way too large");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("File too large"));
    }

    #[test]
    fn test_total_size_limit() {
        let mut builder = SandboxBuilder::new().max_file_size(100).max_total_size(20);

        builder.add_file("file1.txt", b"12345").unwrap(); // 5 bytes
        builder.add_file("file2.txt", b"67890").unwrap(); // 10 total

        let result = builder.add_file("file3.txt", b"this will exceed"); // Would be 26 total
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("File too large"));
    }

    #[test]
    fn test_list_all_files() {
        let mut builder = SandboxBuilder::new();
        builder.add_file("a.txt", b"a").unwrap();
        builder.add_file("b.txt", b"b").unwrap();
        builder.add_file("c.txt", b"c").unwrap();

        let sandbox = builder.build();
        let files: Vec<_> = sandbox.list().collect();

        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_walk_prefix_basic() {
        let mut builder = SandboxBuilder::new();
        builder.add_file("src/main.rs", b"").unwrap();
        builder.add_file("src/lib.rs", b"").unwrap();
        builder.add_file("tests/test.rs", b"").unwrap();
        builder.add_file("README.md", b"").unwrap();

        let sandbox = builder.build();

        let src_files = sandbox.walk_prefix("src");
        assert_eq!(src_files.len(), 2);

        let test_files = sandbox.walk_prefix("tests");
        assert_eq!(test_files.len(), 1);

        let all_files = sandbox.walk_prefix("");
        assert_eq!(all_files.len(), 4);
    }

    #[test]
    fn test_walk_prefix_nested() {
        let mut builder = SandboxBuilder::new();
        builder.add_file("src/parser/mod.rs", b"").unwrap();
        builder.add_file("src/parser/registry.rs", b"").unwrap();
        builder.add_file("src/sandbox/mod.rs", b"").unwrap();

        let sandbox = builder.build();

        let parser_files = sandbox.walk_prefix("src/parser");
        assert_eq!(parser_files.len(), 2);

        let sandbox_files = sandbox.walk_prefix("src/sandbox");
        assert_eq!(sandbox_files.len(), 1);

        let all_src = sandbox.walk_prefix("src");
        assert_eq!(all_src.len(), 3);
    }

    #[test]
    fn test_walk_prefix_trailing_slash() {
        let mut builder = SandboxBuilder::new();
        builder.add_file("dir/file.txt", b"").unwrap();

        let sandbox = builder.build();

        // Should work with or without trailing slash
        assert_eq!(sandbox.walk_prefix("dir").len(), 1);
        assert_eq!(sandbox.walk_prefix("dir/").len(), 1);
    }

    #[test]
    fn test_path_sanitization_in_add_file() {
        let mut builder = SandboxBuilder::new();

        // Should reject dangerous paths
        let result = builder.add_file("../etc/passwd", b"bad");
        assert!(result.is_err());

        let result = builder.add_file("/etc/passwd", b"bad");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_sandbox() {
        let sandbox = SandboxBuilder::new().build();
        assert_eq!(sandbox.file_count(), 0);
        assert_eq!(sandbox.total_size(), 0);
        assert_eq!(sandbox.list().count(), 0);
    }

    #[test]
    fn test_binary_data() {
        let mut builder = SandboxBuilder::new();
        let binary_data: Vec<u8> = (0..=255).collect();
        builder.add_file("binary.dat", &binary_data).unwrap();

        let sandbox = builder.build();
        let retrieved = sandbox.get("binary.dat").unwrap();

        assert_eq!(retrieved.len(), 256);
        assert_eq!(retrieved, binary_data.as_slice());
    }

    #[test]
    fn test_large_file_offsets() {
        let mut builder = SandboxBuilder::new();

        // Add files to test offset calculation
        builder.add_file("file1.txt", b"12345").unwrap(); // offset 0, len 5
        builder.add_file("file2.txt", b"67890").unwrap(); // offset 5, len 5
        builder.add_file("file3.txt", b"ABCDE").unwrap(); // offset 10, len 5

        let sandbox = builder.build();

        let entry1 = sandbox.get_entry("file1.txt").unwrap();
        assert_eq!(entry1.offset, 0);
        assert_eq!(entry1.length, 5);

        let entry2 = sandbox.get_entry("file2.txt").unwrap();
        assert_eq!(entry2.offset, 5);
        assert_eq!(entry2.length, 5);

        let entry3 = sandbox.get_entry("file3.txt").unwrap();
        assert_eq!(entry3.offset, 10);
        assert_eq!(entry3.length, 5);
    }

    #[test]
    fn test_duplicate_path_overwrites() {
        let mut builder = SandboxBuilder::new();
        builder.add_file("test.txt", b"first").unwrap();
        builder.add_file("test.txt", b"second").unwrap();

        let sandbox = builder.build();

        // Should only have one entry (overwritten)
        assert_eq!(sandbox.file_count(), 1);

        // But arena contains both (not ideal, but documents current behavior)
        assert_eq!(sandbox.total_size(), 11); // 5 + 6
    }
}
