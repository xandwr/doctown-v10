use super::{FileMetadata, ParseResult, Parser, SemanticKind, SemanticUnit};

/// Fallback parser for unknown/unsupported file types
pub struct UnknownParser;

impl Parser for UnknownParser {
    fn parse(&self, path: &str, bytes: &[u8]) -> ParseResult {
        let mut metadata = FileMetadata::from_path_and_bytes(path, bytes);

        // Try to extract printable text
        let normalized_text = if metadata.is_utf8 {
            // Valid UTF-8: use as-is
            String::from_utf8_lossy(bytes).into_owned()
        } else {
            // Binary or invalid UTF-8: extract printable ASCII
            Self::extract_printable(bytes)
        };

        // Count lines
        let line_count = normalized_text.lines().count();
        metadata.set_line_count(line_count);

        // Create semantic units (simple newline-based chunking)
        let semantic_units = Self::chunk_by_lines(&normalized_text, metadata.is_utf8);

        ParseResult {
            normalized_text,
            metadata,
            semantic_units,
        }
    }
}

impl UnknownParser {
    /// Extract printable ASCII from binary data
    fn extract_printable(bytes: &[u8]) -> String {
        bytes
            .iter()
            .filter(|&&b| b.is_ascii_graphic() || b.is_ascii_whitespace())
            .map(|&b| b as char)
            .collect()
    }

    /// Chunk text into semantic units by lines
    fn chunk_by_lines(text: &str, is_text_file: bool) -> Vec<SemanticUnit> {
        let kind = if is_text_file {
            SemanticKind::Unknown
        } else {
            SemanticKind::Blob
        };

        let mut units = Vec::new();
        let mut offset = 0;

        for line in text.lines() {
            let line_with_newline = format!("{}\n", line);
            let start = offset;
            let end = offset + line_with_newline.len();

            units.push(SemanticUnit {
                text: line_with_newline,
                start_offset: start,
                end_offset: end,
                kind,
            });

            offset = end;
        }

        // Handle case where there are no newlines
        if units.is_empty() && !text.is_empty() {
            units.push(SemanticUnit {
                text: text.to_string(),
                start_offset: 0,
                end_offset: text.len(),
                kind,
            });
        }

        units
    }
}
