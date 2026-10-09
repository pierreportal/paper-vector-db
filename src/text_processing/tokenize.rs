pub fn tokenize(input: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for c in input.chars() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
            }

            c if c.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }

            _ => {
                current.push(c);
            }
        }
    }

    if in_quotes {
        return Err("Unclosed quote".into());
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_whitespace() {
        assert_eq!(tokenize("search rust").unwrap(), vec!["search", "rust"]);
    }

    #[test]
    fn keeps_quoted_argument_together() {
        assert_eq!(
            tokenize(r#"search "hello world""#).unwrap(),
            vec!["search", "hello world"]
        );
    }

    #[test]
    fn collapses_extra_whitespace() {
        assert_eq!(tokenize("  all   files  ").unwrap(), vec!["all", "files"]);
    }

    #[test]
    fn empty_input_yields_no_tokens() {
        assert!(tokenize("   ").unwrap().is_empty());
    }

    #[test]
    fn unclosed_quote_is_an_error() {
        assert!(tokenize(r#"search "unterminated"#).is_err());
    }
}
