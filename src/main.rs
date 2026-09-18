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
use uuid::Uuid;

pub struct Database {
    pub collections: HashMap<String, Collection>,
}

fn main() {
    let mut text_processing = match TextProcessing::new() {
        Ok(model) => model,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ();
        }
    };

    let db_path = "my_collection.db";

    let mut collection = match Collection::load(db_path) {
        Ok(c) => c,
        Err(_) => Collection::new("my_collection", db_path),
    };

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
                        match collection.save() {
                            Err(e) => println!("{}", e),
                            Ok(_) => println!("New item inserted."),
                        }
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
                Commands::Delete { id } => {
                    let uuid = Uuid::parse_str(&id).unwrap();
                    collection.delete(uuid);
                    match collection.save() {
                        Err(e) => println!("{}", e),
                        Ok(_) => println!("Item deleted."),
                    }
                }
                Commands::Quit => break,
            },
        }
    }
}
