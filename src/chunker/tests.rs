use super::*;
use crate::parser::{SemanticKind, SemanticUnit};

fn make_test_unit(text: &str, kind: SemanticKind, start: usize) -> SemanticUnit {
    SemanticUnit {
        text: text.to_string(),
        start_offset: start,
        end_offset: start + text.len(),
        kind,
    }
}

#[test]
fn test_chunker_preserves_boundaries() {
    let units = vec![
        make_test_unit("// Function A\nfn a() {}", SemanticKind::Function, 0),
        make_test_unit("// Function B\nfn b() {}", SemanticKind::Function, 100),
        make_test_unit("// Function C\nfn c() {}", SemanticKind::Function, 200),
    ];

    let chunks = chunk_semantic_units(units, DEFAULT_MAX_TOKENS);

    // Should merge all small functions into one chunk
    assert_eq!(chunks.len(), 1);
    assert!(chunks[0].text.contains("Function A"));
    assert!(chunks[0].text.contains("Function B"));
    assert!(chunks[0].text.contains("Function C"));
}

#[test]
fn test_chunker_splits_large_content() {
    // Create a huge blob with newlines that exceeds max tokens
    let lines = vec!["x".repeat(100); 200]; // 200 lines of 100 chars each = 20k chars ~5k tokens
    let large_content = lines.join("\n");
    let units = vec![make_test_unit(&large_content, SemanticKind::Blob, 0)];

    let chunks = chunk_semantic_units(units, DEFAULT_MAX_TOKENS);

    // Should split into multiple chunks
    assert!(chunks.len() >= 2, "Large content should be split");

    // Verify all chunks are within limits (allowing some overage for indivisible content)
    for chunk in &chunks {
        assert!(
            chunk.metadata.token_count <= DEFAULT_MAX_TOKENS * 2,
            "Chunk should not be excessively large"
        );
    }
}

#[test]
fn test_chunker_respects_max_tokens() {
    // Create units that individually fit but together exceed limit
    let medium_text = "x".repeat(6000); // ~1500 tokens each
    let units = vec![
        make_test_unit(&medium_text, SemanticKind::Function, 0),
        make_test_unit(&medium_text, SemanticKind::Function, 6000),
    ];

    let chunks = chunk_semantic_units(units, DEFAULT_MAX_TOKENS);

    // Should create 2 separate chunks since combining would exceed limit
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].metadata.unit_count, 1);
    assert_eq!(chunks[1].metadata.unit_count, 1);
}

#[test]
fn test_chunker_tracks_metadata() {
    let units = vec![
        make_test_unit("fn foo() {}", SemanticKind::Function, 0),
        make_test_unit("struct Bar {}", SemanticKind::Class, 50),
    ];

    let chunks = chunk_semantic_units(units.clone(), DEFAULT_MAX_TOKENS);

    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].metadata.unit_count, 2);
    assert_eq!(chunks[0].metadata.start_offset, 0);
    assert!(chunks[0].metadata.kinds.contains(&SemanticKind::Function));
    assert!(chunks[0].metadata.kinds.contains(&SemanticKind::Class));
}

#[test]
fn test_chunker_newline_fallback() {
    // Create a unit with multiple lines where the whole unit exceeds limit
    let lines = vec!["line 1".to_string(); 1000]; // Many small lines
    let large_text = lines.join("\n");

    let units = vec![make_test_unit(&large_text, SemanticKind::Comment, 0)];

    let chunks = chunk_semantic_units(units, 500); // Lower limit to force splitting

    // Should split by newlines
    assert!(chunks.len() > 1, "Should split large unit by newlines");

    // All chunks should be the same kind
    for chunk in &chunks {
        assert!(chunk.metadata.kinds.contains(&SemanticKind::Comment));
    }
}
