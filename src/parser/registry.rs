use super::{Parser, UnknownParser};
use std::collections::HashMap;
use std::path::Path;

/// Dynamic dispatch table for file type parsers
pub struct ParserRegistry {
    /// Fallback parser for unknown extensions
    fallback: Box<dyn Parser>,
    /// Extension -> Parser mapping
    map: HashMap<String, Box<dyn Parser>>,
}

impl ParserRegistry {
    /// Create a new registry with UnknownParser as fallback
    pub fn new() -> Self {
        Self {
            fallback: Box::new(UnknownParser),
            map: HashMap::new(),
        }
    }

    /// Register a parser for a specific file extension
    ///
    /// # Arguments
    /// * `extension` - File extension without dot (e.g., "rs", "py")
    /// * `parser` - Parser implementation
    ///
    /// # Example
    /// ```ignore
    /// registry.register("rs", RustParser::new());
    /// registry.register("py", PythonParser::new());
    /// ```
    pub fn register(&mut self, extension: impl Into<String>, parser: impl Parser + 'static) {
        self.map.insert(extension.into(), Box::new(parser));
    }

    /// Select the appropriate parser for a given file path
    ///
    /// Falls back to UnknownParser if no extension-specific parser exists
    pub fn select(&self, path: &str) -> &dyn Parser {
        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        self.map.get(&ext).map(|p| &**p).unwrap_or(&*self.fallback)
    }

    /// Get the number of registered parsers (excluding fallback)
    pub fn parser_count(&self) -> usize {
        self.map.len()
    }

    /// List all registered extensions
    pub fn registered_extensions(&self) -> Vec<&str> {
        self.map.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_to_unknown() {
        let registry = ParserRegistry::new();
        let parser = registry.select("foo.xyz");

        // Should return UnknownParser for unknown extensions
        let result = parser.parse("foo.xyz", b"test");
        assert_eq!(result.metadata.language, "unknown");
    }

    #[test]
    fn test_extension_selection() {
        let mut registry = ParserRegistry::new();
        registry.register("test", UnknownParser);

        assert_eq!(registry.parser_count(), 1);
        assert!(registry.registered_extensions().contains(&"test"));
    }
}
