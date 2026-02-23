// Text Chunker — recursive character splitting with overlap
//
// Internal module, used by knowledge pipeline for document chunking.

use serde::{Deserialize, Serialize};

const DEFAULT_CHUNK_SIZE: usize = 500;
const DEFAULT_CHUNK_OVERLAP: usize = 100;
const SEPARATORS: &[&str] = &["\n\n", "\n", ". ", " "];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub line_start: usize,
    pub line_end: usize,
    pub page: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextChunk {
    pub content: String,
    pub metadata: ChunkMetadata,
}

pub fn chunk_text(text: &str, chunk_size: usize, chunk_overlap: usize) -> Vec<TextChunk> {
    let chunk_size = if chunk_size == 0 { DEFAULT_CHUNK_SIZE } else { chunk_size };
    let chunk_overlap = if chunk_overlap == 0 { DEFAULT_CHUNK_OVERLAP } else { chunk_overlap };

    if text.trim().is_empty() {
        return vec![];
    }

    let segments = recursive_split(text, SEPARATORS, chunk_size);
    let line_map = build_line_map(text);

    let mut chunks: Vec<TextChunk> = Vec::new();
    let mut current_text = String::new();
    let mut current_start: usize = 0;

    for segment in &segments {
        if current_text.len() + segment.len() <= chunk_size {
            current_text.push_str(segment);
        } else {
            if !current_text.trim().is_empty() {
                let offset = text[current_start..].find(&current_text[..])
                    .map(|o| current_start + o)
                    .unwrap_or(current_start);

                chunks.push(TextChunk {
                    content: current_text.trim().to_string(),
                    metadata: ChunkMetadata {
                        line_start: get_line_number(&line_map, offset),
                        line_end: get_line_number(&line_map, offset + current_text.len()),
                        page: None,
                    },
                });
                current_start = offset.saturating_add(current_text.len()).saturating_sub(chunk_overlap);
            }

            // Start new chunk with overlap
            if chunk_overlap > 0 && current_text.len() > chunk_overlap {
                let overlap_start = current_text.len() - chunk_overlap;
                current_text = format!("{}{}", &current_text[overlap_start..], segment);
            } else {
                current_text = segment.clone();
            }
        }
    }

    // Add the last chunk
    if !current_text.trim().is_empty() {
        let offset = text[current_start..].find(&current_text[..])
            .map(|o| current_start + o)
            .unwrap_or(current_start);

        chunks.push(TextChunk {
            content: current_text.trim().to_string(),
            metadata: ChunkMetadata {
                line_start: get_line_number(&line_map, offset),
                line_end: get_line_number(&line_map, offset + current_text.len()),
                page: None,
            },
        });
    }

    chunks
}

fn recursive_split(text: &str, separators: &[&str], chunk_size: usize) -> Vec<String> {
    if text.len() <= chunk_size {
        return vec![text.to_string()];
    }

    if separators.is_empty() {
        // Character-level split as last resort
        return text.as_bytes()
            .chunks(chunk_size)
            .map(|c| String::from_utf8_lossy(c).to_string())
            .collect();
    }

    let separator = separators[0];
    let parts: Vec<&str> = text.split(separator).collect();

    if parts.len() <= 1 {
        return recursive_split(text, &separators[1..], chunk_size);
    }

    let mut result: Vec<String> = Vec::new();
    let mut current = String::new();

    for (i, part) in parts.iter().enumerate() {
        let with_sep = if i < parts.len() - 1 {
            format!("{}{}", part, separator)
        } else {
            part.to_string()
        };

        if current.len() + with_sep.len() <= chunk_size {
            current.push_str(&with_sep);
        } else {
            if !current.is_empty() {
                result.push(current);
            }
            if with_sep.len() > chunk_size {
                result.extend(recursive_split(&with_sep, &separators[1..], chunk_size));
                current = String::new();
            } else {
                current = with_sep;
            }
        }
    }

    if !current.is_empty() {
        result.push(current);
    }

    result
}

/// Build a map of line start positions (byte offsets)
fn build_line_map(text: &str) -> Vec<usize> {
    let mut map = vec![0usize]; // position 0 = line 1
    for (i, ch) in text.char_indices() {
        if ch == '\n' {
            map.push(i + 1);
        }
    }
    map
}

/// Get 1-indexed line number for a byte offset
fn get_line_number(line_map: &[usize], offset: usize) -> usize {
    let offset = offset.min(line_map.last().copied().unwrap_or(0) + 1000);
    // Binary search
    let mut lo = 0usize;
    let mut hi = line_map.len().saturating_sub(1);
    while lo < hi {
        let mid = (lo + hi + 1) / 2;
        if line_map[mid] <= offset {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    lo + 1 // 1-indexed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_text() {
        let chunks = chunk_text("", DEFAULT_CHUNK_SIZE, DEFAULT_CHUNK_OVERLAP);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_whitespace_only() {
        let chunks = chunk_text("   \n\n  ", DEFAULT_CHUNK_SIZE, DEFAULT_CHUNK_OVERLAP);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_small_text() {
        let text = "Hello, world!";
        let chunks = chunk_text(text, DEFAULT_CHUNK_SIZE, DEFAULT_CHUNK_OVERLAP);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, "Hello, world!");
        assert_eq!(chunks[0].metadata.line_start, 1);
    }

    #[test]
    fn test_paragraph_splitting() {
        let text = "Paragraph one with enough text to be meaningful.\n\nParagraph two with different content.\n\nParagraph three is also here.";
        let chunks = chunk_text(text, 60, 10);
        assert!(chunks.len() >= 2);
        for chunk in &chunks {
            assert!(!chunk.content.is_empty());
        }
    }

    #[test]
    fn test_line_numbers() {
        let text = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5";
        let chunks = chunk_text(text, 1000, 0);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].metadata.line_start, 1);
        assert_eq!(chunks[0].metadata.line_end, 5);
    }

    #[test]
    fn test_build_line_map() {
        let text = "a\nb\nc";
        let map = build_line_map(text);
        assert_eq!(map, vec![0, 2, 4]);
    }

    #[test]
    fn test_get_line_number() {
        let map = vec![0, 5, 10, 15];
        assert_eq!(get_line_number(&map, 0), 1);
        assert_eq!(get_line_number(&map, 3), 1);
        assert_eq!(get_line_number(&map, 5), 2);
        assert_eq!(get_line_number(&map, 12), 3);
    }

    #[test]
    fn test_recursive_split_basic() {
        let text = "A\n\nB\n\nC";
        let parts = recursive_split(text, SEPARATORS, 5);
        assert!(parts.len() >= 2);
    }

    #[test]
    fn test_chunk_overlap() {
        // Create text that will definitely need multiple chunks
        let text = (0..20).map(|i| format!("Sentence number {}.", i)).collect::<Vec<_>>().join(" ");
        let chunks = chunk_text(&text, 80, 20);
        assert!(chunks.len() > 1);
    }
}
