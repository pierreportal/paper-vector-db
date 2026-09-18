use regex::Regex;

#[derive(Debug)]
pub struct TextChunk {
    pub content: String,
    pub index: usize,
}

const MAX_CHARS: usize = 4_000;
const MIN_CHARS: usize = 100;

fn split_markdown(text: &str) -> Vec<TextChunk> {
    let re = Regex::new(r"(?m)^(#{1,6})\s+(.*)$").unwrap();

    let matches: Vec<_> = re.captures_iter(text).collect();

    let mut chunks = Vec::new();

    // Content before the first heading
    if let Some(first) = matches.first() {
        let content = text[..first.get(0).unwrap().start()].trim();

        if content.len() >= MIN_CHARS && content.len() <= MAX_CHARS {
            chunks.push(TextChunk {
                content: content.to_string(),
                index: chunks.len(),
            });
        }
    } else {
        // No headings at all
        if text.len() >= MIN_CHARS && text.len() <= MAX_CHARS {
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

    let b = split_markdown(text);
    println!("{:#?}", b);

    let mut chunks = Vec::new();
    let mut current = String::new();

    for paragraph in paragraphs {
        if current.len() + paragraph.len() + 2 <= MAX_CHARS {
            if !current.is_empty() {
                current.push_str("\n\n");
            }

            current.push_str(paragraph);
        } else {
            if current.len() >= MIN_CHARS {
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
