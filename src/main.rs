mod cli;
mod db;
mod text_processing;
mod types;
use crate::{
    text_processing::embeddings::{BatchSize, TextProcessing},
    types::document::DocumentInsert,
};
use cli::Commands;
use db::collection::Collection;
use std::collections::HashMap;
use std::io::{self, Write};

pub struct Database {
    pub collections: HashMap<String, Collection>,
}

fn main() -> anyhow::Result<()> {
    let mut text_processing = match TextProcessing::new() {
        Ok(model) => model,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Ok(());
        }
    };

    let mut collection = Collection::new("my_collection");

    println!("{:?}", collection);

    loop {
        print!("paper-db> ");
        io::stdout().flush().unwrap();
        let mut user_input = String::new();

        let stdin = io::stdin();

        stdin
            .read_line(&mut user_input)
            .expect("Failed to read input");

        let line = user_input.trim();

        if line.is_empty() {
            continue;
        }

        match parse_command(&user_input) {
            Err(e) => println!("{}", e),
            Ok(cmd) => {
                println!("{:?}", cmd);
                match cmd {
                    Commands::Insert { doc } => {
                        match text_processing.embed(&doc, BatchSize::None) {
                            Ok(embedding) => {
                                collection.insert(DocumentInsert {
                                    content: doc.to_owned(),
                                    embedding,
                                });
                            }
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    }
                    Commands::Search { query } => {
                        match text_processing.embed(&query, BatchSize::None) {
                            Ok(embedding) => {
                                let results = collection.search(&embedding, 5);
                                println!("Best matches: {:?}", results);
                            }
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    }
                    Commands::Quit => {}
                }
            }
        }

        ()
    }
}

fn parse_command(user_input: &str) -> Result<Commands, String> {
    println!("{}", user_input);
    if let Ok(tokens) = tokenize(user_input) {
        let tokens: Vec<&str> = tokens.iter().map(String::as_str).collect();

        match tokens.as_slice() {
            ["quit"] | ["exit"] => Ok(Commands::Quit),
            ["insert", item] => Ok(Commands::Insert {
                doc: item.to_string(),
            }),
            ["search", item] => Ok(Commands::Search {
                query: item.to_string(),
            }),
            _ => Err(format!("Unknown command: {}", tokens.as_slice().join(" ")).into()),
        }
    } else {
        Err("".into())
    }
}

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
