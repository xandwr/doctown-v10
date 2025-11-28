use crate::parser::{SemanticKind, SemanticUnit};

/// A chunk of text ready for embedding/indexing
#[derive(Debug, Clone)]
pub struct Chunk {
    /// The text content of this chunk
    pub text: String,
    /// Metadata about the chunk
    pub metadata: ChunkMetadata,
}

/// Metadata for a chunk
#[derive(Debug, Clone)]
pub struct ChunkMetadata {
    /// Estimated token count
    pub token_count: usize,
    /// Byte offset in original file (start)
    pub start_offset: usize,
    /// Byte offset in original file (end)
    pub end_offset: usize,
    /// Semantic kinds included in this chunk
    pub kinds: Vec<SemanticKind>,
    /// Number of semantic units merged into this chunk
    pub unit_count: usize,
}

/// Chunk semantic units according to the rules:
/// - Merge small semantic units together
/// - Split huge units if they exceed max_tokens
/// - Aim for <2k tokens per chunk (configurable)
/// - Preserve unit boundaries if possible
/// - Fallback to newline splitting for oversized units
pub fn chunk_semantic_units(units: Vec<SemanticUnit>, max_tokens: usize) -> Vec<Chunk> {
    if units.is_empty() {
        return vec![];
    }

    let mut chunks = Vec::new();
    let mut current_batch: Vec<SemanticUnit> = Vec::new();
    let mut current_tokens = 0;

    for unit in units {
        let unit_tokens = estimate_tokens(&unit.text);

        // If this unit alone exceeds max_tokens, split it separately
        if unit_tokens > max_tokens {
            // Flush current batch first
            if !current_batch.is_empty() {
                chunks.push(create_chunk_from_units(current_batch, current_tokens));
                current_batch = Vec::new();
                current_tokens = 0;
            }

            // Split the huge unit
            chunks.extend(split_large_unit(unit, max_tokens));
            continue;
        }

        // Check if adding this unit would exceed the limit
        if current_tokens + unit_tokens > max_tokens && !current_batch.is_empty() {
            // Flush current batch
            chunks.push(create_chunk_from_units(current_batch, current_tokens));
            current_batch = Vec::new();
            current_tokens = 0;
        }

        // Add unit to current batch
        current_tokens += unit_tokens;
        current_batch.push(unit);
    }

    // Flush remaining batch
    if !current_batch.is_empty() {
        chunks.push(create_chunk_from_units(current_batch, current_tokens));
    }

    chunks
}

/// Create a chunk from a batch of semantic units
fn create_chunk_from_units(units: Vec<SemanticUnit>, token_count: usize) -> Chunk {
    let start_offset = units.first().map(|u| u.start_offset).unwrap_or(0);
    let end_offset = units.last().map(|u| u.end_offset).unwrap_or(0);

    let kinds: Vec<SemanticKind> = units
        .iter()
        .map(|u| u.kind)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let unit_count = units.len();
    let text = units
        .into_iter()
        .map(|u| u.text)
        .collect::<Vec<_>>()
        .join("\n\n");

    Chunk {
        text,
        metadata: ChunkMetadata {
            token_count,
            start_offset,
            end_offset,
            kinds,
            unit_count,
        },
    }
}

/// Split a large semantic unit that exceeds max_tokens
/// Falls back to newline-based splitting
fn split_large_unit(unit: SemanticUnit, max_tokens: usize) -> Vec<Chunk> {
    let lines: Vec<&str> = unit.text.lines().collect();
    let mut chunks = Vec::new();
    let mut current_lines = Vec::new();
    let mut current_tokens = 0;

    for line in lines {
        let line_tokens = estimate_tokens(line);

        // If a single line is too big, we have to include it anyway
        if line_tokens > max_tokens {
            // Flush current chunk if any
            if !current_lines.is_empty() {
                let text = current_lines.join("\n");
                chunks.push(create_single_chunk(
                    text,
                    current_tokens,
                    unit.kind,
                    unit.start_offset,
                ));
                current_lines.clear();
                current_tokens = 0;
            }

            // Add the huge line as its own chunk
            chunks.push(create_single_chunk(
                line.to_string(),
                line_tokens,
                unit.kind,
                unit.start_offset,
            ));
            continue;
        }

        // Check if adding this line would exceed the limit
        if current_tokens + line_tokens > max_tokens && !current_lines.is_empty() {
            let text = current_lines.join("\n");
            chunks.push(create_single_chunk(
                text,
                current_tokens,
                unit.kind,
                unit.start_offset,
            ));
            current_lines.clear();
            current_tokens = 0;
        }

        current_tokens += line_tokens;
        current_lines.push(line);
    }

    // Flush remaining lines
    if !current_lines.is_empty() {
        let text = current_lines.join("\n");
        chunks.push(create_single_chunk(
            text,
            current_tokens,
            unit.kind,
            unit.start_offset,
        ));
    }

    // If we somehow ended up with no chunks, create one from the whole unit
    if chunks.is_empty() {
        let token_count = estimate_tokens(&unit.text);
        chunks.push(create_single_chunk(
            unit.text,
            token_count,
            unit.kind,
            unit.start_offset,
        ));
    }

    chunks
}

/// Create a single chunk with the given properties
fn create_single_chunk(
    text: String,
    token_count: usize,
    kind: SemanticKind,
    start_offset: usize,
) -> Chunk {
    let end_offset = start_offset + text.len();

    Chunk {
        text,
        metadata: ChunkMetadata {
            token_count,
            start_offset,
            end_offset,
            kinds: vec![kind],
            unit_count: 1,
        },
    }
}

/// Estimate token count for a piece of text
/// Uses a simple heuristic: 1 token H 4 characters
/// This is a rough approximation suitable for most text
fn estimate_tokens(text: &str) -> usize {
    // Average token is ~4 characters for English text
    // Add 1 to avoid zero-token estimates for very short text
    (text.len() / 4).max(1)
}

#[cfg(test)]
mod chunk_tests {
    use super::*;

    fn make_unit(text: &str, kind: SemanticKind) -> SemanticUnit {
        SemanticUnit {
            text: text.to_string(),
            start_offset: 0,
            end_offset: text.len(),
            kind,
        }
    }

    #[test]
    fn test_empty_units() {
        let chunks = chunk_semantic_units(vec![], 2000);
        assert_eq!(chunks.len(), 0);
    }

    #[test]
    fn test_single_small_unit() {
        let units = vec![make_unit("fn main() {}", SemanticKind::Function)];
        let chunks = chunk_semantic_units(units, 2000);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].metadata.unit_count, 1);
    }

    #[test]
    fn test_merge_small_units() {
        let units = vec![
            make_unit("fn foo() {}", SemanticKind::Function),
            make_unit("fn bar() {}", SemanticKind::Function),
            make_unit("fn baz() {}", SemanticKind::Function),
        ];

        let chunks = chunk_semantic_units(units, 2000);

        // All should merge into one chunk since they're small
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].metadata.unit_count, 3);
    }

    #[test]
    fn test_split_at_boundary() {
        // Create units that will fill up the token limit
        let large_text = "x".repeat(8000); // ~2000 tokens
        let units = vec![
            make_unit(&large_text, SemanticKind::Function),
            make_unit("fn small() {}", SemanticKind::Function),
        ];

        let chunks = chunk_semantic_units(units, 2000);

        // Should split into 2 chunks at the unit boundary
        assert_eq!(chunks.len(), 2);
    }

    #[test]
    fn test_split_huge_unit() {
        // Create a unit with multiple lines that exceeds max tokens
        let lines = vec!["x".repeat(100); 200]; // 200 lines of 100 chars each = 20k chars ~5k tokens
        let huge_text = lines.join("\n");
        let units = vec![make_unit(&huge_text, SemanticKind::Blob)];

        let chunks = chunk_semantic_units(units, 2000);

        // Should split into multiple chunks
        assert!(chunks.len() > 1);
    }

    #[test]
    fn test_token_estimation() {
        assert_eq!(estimate_tokens(""), 1); // Minimum of 1
        assert_eq!(estimate_tokens("test"), 1); // 4 chars = 1 token
        assert_eq!(estimate_tokens("test test"), 2); // 9 chars H 2 tokens
        assert_eq!(estimate_tokens(&"x".repeat(8000)), 2000); // 8000 chars = 2000 tokens
    }
}
