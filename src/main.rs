mod cli;
mod db;
mod text_processing;
mod types;
use crate::{
    db::collection,
    text_processing::embeddings::{BatchSize, TextProcessing},
    types::document::DocumentInsert,
};
use clap::Parser;
use cli::{Cli, Commands};
use db::collection::Collection;
use std::collections::HashMap;
// use std::io;
use std::io::{self, Write};

pub struct Database {
    pub collections: HashMap<String, Collection>,
}

fn main() {
    let mut text_processing = match TextProcessing::new() {
        Ok(model) => model,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };

    let mut collection = Collection::new("my_collection");

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

        parse_command(&user_input);
    }
}

fn parse_command(user_input: &str) -> Result<Commands, String> {
    if let Ok(tokens) = tokenize(user_input) {
        let tokens: Vec<&str> = tokens.iter().map(String::as_str).collect();

        match tokens.as_slice() {
            ["quit"] | ["exit"] => Ok(Commands::Quit),
            ["insert", collection, item] => Ok(Commands::Insert {
                doc: item.to_string(),
            }),
            ["search", collection, item] => Ok(Commands::Search {
                query: item.to_string(),
            }),
            _ => Err("Unknown command".into()),
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
/*
let cli = Cli::parse();

match &cli.command {
    Commands::Insert { doc } => match text_processing.embed(doc, BatchSize::None) {
        Ok(embedding) => {
            collection.insert(DocumentInsert {
                content: doc.to_owned(),
                embedding,
            });
        }
        Err(e) => eprintln!("Error: {}", e),
    },
    Commands::Search { query } => match text_processing.embed(query, BatchSize::None) {
        Ok(embedding) => {
            let results = collection.search(&embedding, 5);
            println!("Best matches: {:?}", results);
        }
        Err(e) => eprintln!("Error: {}", e),
    },
}
*/
