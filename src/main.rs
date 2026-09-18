mod cli;
mod db;
mod text_processing;
mod types;

use cli::{Commands, parse_command};
use db::collection::Collection;
use std::collections::HashMap;
use std::io::{self, Write};
use text_processing::embeddings::{BatchSize, TextProcessing};
use types::document::DocumentInsert;

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

    loop {
        print!("\npaper-db ~> ");
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
            Err(e) => println!("Error: {}", e),
            Ok(cmd) => match cmd {
                Commands::Insert { doc } => match text_processing.embed(&doc, BatchSize::None) {
                    Ok(embedding) => {
                        collection.insert(DocumentInsert {
                            content: doc.to_owned(),
                            embedding,
                        });
                    }
                    Err(e) => eprintln!("Error: {}", e),
                },
                Commands::Search { query } => {
                    match text_processing.embed(&query, BatchSize::None) {
                        Ok(embedding) => {
                            let results = collection.search(&embedding, 5);
                            println!("Best matches: {:#?}", results);
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
                Commands::Quit => {}
            },
        }
        ()
    }
}
