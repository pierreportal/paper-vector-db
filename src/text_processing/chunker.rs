use crate::config::{CHUNKER_MAX_CHARS, CHUNKER_MIN_CHARS};
use regex::Regex;

#[derive(Debug)]
pub struct TextChunk {
    pub content: String,
    pub index: usize,
}

fn split_markdown(text: &str) -> Vec<TextChunk> {
    let re = Regex::new(r"(?m)^(#{1,6})\s+(.*)$").unwrap();

    let matches: Vec<_> = re.captures_iter(text).collect();

    let mut chunks = Vec::new();

    // Content before the first heading
    if let Some(first) = matches.first() {
        let content = text[..first.get(0).unwrap().start()].trim();

        if content.len() >= CHUNKER_MIN_CHARS && content.len() <= CHUNKER_MAX_CHARS {
            chunks.push(TextChunk {
                content: content.to_string(),
                index: chunks.len(),
            });
        }
    } else {
        // No headings at all
        if text.len() >= CHUNKER_MIN_CHARS && text.len() <= CHUNKER_MAX_CHARS {
            chunks.push(TextChunk {
                content: text.trim().to_string(),
                index: 0,
            });
        }

        return chunks;
    }

    // Content belonging to each heading
    for (i, captures) in matches.iter().enumerate() {
        let heading = captures.get(0).unwrap();

        // Start immediately after the heading
        let content_start = heading.end();

        // Stop at the next heading, or EOF
        let content_end = matches
            .get(i + 1)
            .map(|next| next.get(0).unwrap().start())
            .unwrap_or(text.len());

        let content = text[content_start..content_end].trim();

        if !content.is_empty() {
            chunks.push(TextChunk {
                content: content.to_string(),
                index: chunks.len(),
            });
        }
    }

    chunks
}

fn split_text(text: &str) -> Vec<TextChunk> {
    let paragraphs: Vec<&str> = text
        .split("\n\n")
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();

    let mut chunks = Vec::new();
    let mut current = String::new();

    for paragraph in paragraphs {
        if current.len() + paragraph.len() + 2 <= CHUNKER_MAX_CHARS {
            if !current.is_empty() {
                current.push_str("\n\n");
            }

            current.push_str(paragraph);
        } else {
            if current.len() >= CHUNKER_MIN_CHARS {
                chunks.push(TextChunk {
                    content: current,
                    index: chunks.len(),
                });
            }

            current = paragraph.to_string();
        }
    }

    if !current.is_empty() {
        chunks.push(TextChunk {
            content: current,
            index: chunks.len(),
        });
    }

    chunks
}

pub fn chunk_text(extension: &str, text: &str) -> Vec<TextChunk> {
    match extension {
        "md" => split_markdown(text),
        _ => split_text(text),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_splits_on_headings() {
        // "before" section is padded past CHUNKER_MIN_CHARS so it is kept.
        let text = format!(
            "{}\n\n# One\nfirst section\n\n## Two\nsecond section\n",
            "intro ".repeat(30)
        );

        let chunks = chunk_text("md", &text);

        assert_eq!(chunks.len(), 3);
        assert!(chunks[0].content.starts_with("intro"));
        assert_eq!(chunks[1].content, "first section");
        assert_eq!(chunks[2].content, "second section");
        // indices are contiguous
        assert_eq!(
            chunks.iter().map(|c| c.index).collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
    }

    #[test]
    fn markdown_without_headings_is_a_single_chunk() {
        let text = "a ".repeat(CHUNKER_MIN_CHARS);
        let chunks = chunk_text("md", text.trim());
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].index, 0);
    }

    #[test]
    fn short_markdown_without_headings_is_dropped() {
        let chunks = chunk_text("md", "too short");
        assert!(chunks.is_empty());
    }

    #[test]
    fn plain_text_groups_paragraphs() {
        let text = "first paragraph\n\nsecond paragraph";
        let chunks = chunk_text("txt", text);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, "first paragraph\n\nsecond paragraph");
    }

    #[test]
    fn plain_text_respects_max_chars() {
        // Two paragraphs that cannot fit inside CHUNKER_MAX_CHARS together.
        let big = "x".repeat(CHUNKER_MAX_CHARS - 10);
        let text = format!("{big}\n\nlorem ipsum dolor sit amet");

        let chunks = chunk_text("txt", &text);

        assert_eq!(chunks.len(), 2);
        assert!(chunks.iter().all(|c| c.content.len() <= CHUNKER_MAX_CHARS));
    }
}
