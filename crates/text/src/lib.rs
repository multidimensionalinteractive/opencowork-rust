//! OpenCoWork text processing crate.
//!
//! Provides text chunking, truncation, and formatting utilities for
//! splitting long messages across platform character limits.

/// Maximum safe chunk size for most messaging platforms.
pub const DEFAULT_CHUNK_SIZE: usize = 4000;

/// Split text into chunks respecting word boundaries.
///
/// Never breaks a word in half. If a single word exceeds `max_bytes`,
/// it is placed in its own chunk unsplit.
pub fn chunk_text(text: &str, max_bytes: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }

    if text.len() <= max_bytes {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut current = String::with_capacity(max_bytes.min(text.len()));

    for word in text.split(' ') {
        // If current chunk is empty, we must accept this word even if it's long
        if current.is_empty() {
            current.push_str(word);
        // If adding a space + word would exceed limit, flush current chunk
        } else if current.len() + 1 + word.len() > max_bytes {
            chunks.push(current);
            current = String::with_capacity(max_bytes.min(text.len()));
            current.push_str(word);
        } else {
            current.push(' ');
            current.push_str(word);
        }
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

/// Truncate text to at most `max_bytes`, appending an ellipsis indicator
/// if truncation occurred.
pub fn truncate_text(text: &str, max_bytes: usize) -> String {
    if text.len() <= max_bytes {
        return text.to_string();
    }

    let suffix = "…";
    let limit = max_bytes.saturating_sub(suffix.len());

    // Find a valid char boundary at or before the limit
    let mut end = limit;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }

    format!("{}{}", &text[..end], suffix)
}

/// Create a short summary of an input message for logging / display.
///
/// Returns the first `max_chars` characters with a length indicator.
pub fn format_input_summary(text: &str, max_chars: usize) -> String {
    let truncated: String = text.chars().take(max_chars).collect();
    let total_len = text.chars().count();

    if total_len <= max_chars {
        format!("({} chars) {}", total_len, truncated)
    } else {
        format!("({} chars) {}…", total_len, truncated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_text_no_split_needed() {
        let chunks = chunk_text("hello world", 100);
        assert_eq!(chunks, vec!["hello world"]);
    }

    #[test]
    fn test_chunk_text_respects_word_boundaries() {
        let text = "one two three four five";
        let chunks = chunk_text(text, 10);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], "one two");
        assert_eq!(chunks[1], "three four");
        assert_eq!(chunks[2], "five");
    }

    #[test]
    fn test_chunk_text_long_word() {
        let text = "short superlongwordthatwontfit another";
        let chunks = chunk_text(text, 10);
        // "superlongwordthatwontfit" should be its own chunk
        assert!(chunks.len() >= 2);
        assert!(chunks.iter().any(|c| c.contains("superlongwordthatwontfit")));
    }

    #[test]
    fn test_truncate_text() {
        assert_eq!(truncate_text("hello", 10), "hello");
        let truncated = truncate_text("hello world this is long", 10);
        assert!(truncated.ends_with('…'));
        assert!(truncated.len() <= 10);
    }

    #[test]
    fn test_format_input_summary() {
        let summary = format_input_summary("hello", 20);
        assert_eq!(summary, "(5 chars) hello");

        let long = "a".repeat(100);
        let summary = format_input_summary(&long, 10);
        assert!(summary.starts_with("(100 chars)"));
    }
}
