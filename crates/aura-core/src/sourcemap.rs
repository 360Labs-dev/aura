//! # Source Map Generator (v3 format)
//!
//! Generates source maps that map generated code back to .aura source files.
//! Enables browser devtools debugging — set breakpoints in .aura, step through
//! generated JS, see .aura source in the Sources panel.
//!
//! ## Format: Source Map v3
//! https://sourcemaps.info/spec.html
//!
//! The mapping uses VLQ (Variable-Length Quantity) encoding for compact representation.

use serde::Serialize;

/// A source map in v3 format.
#[derive(Debug, Serialize)]
pub struct SourceMap {
    pub version: u8,
    pub file: String,
    pub sources: Vec<String>,
    #[serde(rename = "sourcesContent")]
    pub sources_content: Vec<Option<String>>,
    pub names: Vec<String>,
    pub mappings: String,
}

/// A single mapping entry: generated position → source position.
#[derive(Debug, Clone)]
pub struct Mapping {
    /// Generated line (0-based).
    pub gen_line: usize,
    /// Generated column (0-based).
    pub gen_col: usize,
    /// Source file index.
    pub source: usize,
    /// Source line (0-based).
    pub src_line: usize,
    /// Source column (0-based).
    pub src_col: usize,
}

/// Builder for constructing source maps incrementally.
pub struct SourceMapBuilder {
    file: String,
    sources: Vec<String>,
    sources_content: Vec<Option<String>>,
    names: Vec<String>,
    mappings: Vec<Mapping>,
}

impl SourceMapBuilder {
    pub fn new(output_file: &str) -> Self {
        Self {
            file: output_file.to_string(),
            sources: Vec::new(),
            sources_content: Vec::new(),
            names: Vec::new(),
            mappings: Vec::new(),
        }
    }

    /// Add a source file. Returns the source index.
    pub fn add_source(&mut self, path: &str, content: Option<&str>) -> usize {
        let idx = self.sources.len();
        self.sources.push(path.to_string());
        self.sources_content.push(content.map(|s| s.to_string()));
        idx
    }

    /// Add a mapping from generated position to source position.
    pub fn add_mapping(&mut self, mapping: Mapping) {
        self.mappings.push(mapping);
    }

    /// Add a simple line mapping: generated line N → source line M.
    pub fn map_line(&mut self, gen_line: usize, src_line: usize, source: usize) {
        self.mappings.push(Mapping {
            gen_line,
            gen_col: 0,
            source,
            src_line,
            src_col: 0,
        });
    }

    /// Build the final source map.
    pub fn build(mut self) -> SourceMap {
        // Sort mappings by generated position
        self.mappings
            .sort_by(|a, b| a.gen_line.cmp(&b.gen_line).then(a.gen_col.cmp(&b.gen_col)));

        let mappings = encode_mappings(&self.mappings);

        SourceMap {
            version: 3,
            file: self.file,
            sources: self.sources,
            sources_content: self.sources_content,
            names: self.names,
            mappings,
        }
    }
}

/// Encode mappings as a VLQ-encoded string.
fn encode_mappings(mappings: &[Mapping]) -> String {
    if mappings.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    let mut prev_gen_line = 0;
    let mut prev_gen_col: i64 = 0;
    let mut prev_source: i64 = 0;
    let mut prev_src_line: i64 = 0;
    let mut prev_src_col: i64 = 0;

    for mapping in mappings {
        // Add semicolons for empty lines
        while prev_gen_line < mapping.gen_line {
            result.push(';');
            prev_gen_line += 1;
            prev_gen_col = 0;
        }

        // Add comma separator within a line
        if !result.is_empty() && !result.ends_with(';') {
            result.push(',');
        }

        // Encode relative values as VLQ
        let gen_col_delta = mapping.gen_col as i64 - prev_gen_col;
        let source_delta = mapping.source as i64 - prev_source;
        let src_line_delta = mapping.src_line as i64 - prev_src_line;
        let src_col_delta = mapping.src_col as i64 - prev_src_col;

        result.push_str(&vlq_encode(gen_col_delta));
        result.push_str(&vlq_encode(source_delta));
        result.push_str(&vlq_encode(src_line_delta));
        result.push_str(&vlq_encode(src_col_delta));

        prev_gen_col = mapping.gen_col as i64;
        prev_source = mapping.source as i64;
        prev_src_line = mapping.src_line as i64;
        prev_src_col = mapping.src_col as i64;
    }

    result
}

/// Encode a single value as VLQ base64.
fn vlq_encode(value: i64) -> String {
    let mut vlq = if value < 0 {
        ((-value) << 1) | 1
    } else {
        value << 1
    };

    let mut result = String::new();
    let base64_chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    loop {
        let mut digit = (vlq & 0x1f) as u8;
        vlq >>= 5;
        if vlq > 0 {
            digit |= 0x20; // continuation bit
        }
        result.push(base64_chars[digit as usize] as char);
        if vlq == 0 {
            break;
        }
    }

    result
}

/// Generate a source map comment for appending to JS output.
pub fn source_map_comment(map_filename: &str) -> String {
    format!("//# sourceMappingURL={}\n", map_filename)
}

/// Generate an inline source map (base64-encoded, no separate file).
pub fn inline_source_map(map: &SourceMap) -> String {
    let json = serde_json::to_string(map).unwrap_or_default();
    let encoded = base64_encode(json.as_bytes());
    format!(
        "//# sourceMappingURL=data:application/json;base64,{}\n",
        encoded
    )
}

fn base64_encode(data: &[u8]) -> String {
    let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    let mut i = 0;
    while i < data.len() {
        let b0 = data[i] as u32;
        let b1 = if i + 1 < data.len() {
            data[i + 1] as u32
        } else {
            0
        };
        let b2 = if i + 2 < data.len() {
            data[i + 2] as u32
        } else {
            0
        };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(chars[((triple >> 18) & 0x3f) as usize] as char);
        result.push(chars[((triple >> 12) & 0x3f) as usize] as char);
        if i + 1 < data.len() {
            result.push(chars[((triple >> 6) & 0x3f) as usize] as char);
        } else {
            result.push('=');
        }
        if i + 2 < data.len() {
            result.push(chars[(triple & 0x3f) as usize] as char);
        } else {
            result.push('=');
        }
        i += 3;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vlq_encode() {
        assert_eq!(vlq_encode(0), "A");
        assert_eq!(vlq_encode(1), "C");
        assert_eq!(vlq_encode(-1), "D");
        assert_eq!(vlq_encode(5), "K");
    }

    #[test]
    fn test_source_map_builder() {
        let mut builder = SourceMapBuilder::new("app.js");
        let src = builder.add_source("main.aura", Some("app Hello"));
        builder.map_line(0, 0, src);
        builder.map_line(1, 1, src);

        let map = builder.build();
        assert_eq!(map.version, 3);
        assert_eq!(map.file, "app.js");
        assert_eq!(map.sources, vec!["main.aura"]);
        assert!(!map.mappings.is_empty());
    }

    #[test]
    fn test_source_map_json() {
        let mut builder = SourceMapBuilder::new("app.js");
        builder.add_source("main.aura", None);
        builder.map_line(0, 0, 0);
        let map = builder.build();
        let json = serde_json::to_string(&map).unwrap();
        assert!(json.contains("\"version\":3"));
        assert!(json.contains("\"sources\":[\"main.aura\"]"));
    }

    #[test]
    fn test_inline_source_map() {
        let mut builder = SourceMapBuilder::new("app.js");
        builder.add_source("main.aura", None);
        let map = builder.build();
        let inline = inline_source_map(&map);
        assert!(inline.starts_with("//# sourceMappingURL=data:application/json;base64,"));
    }
}
