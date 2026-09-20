mod cli;
mod config;
mod db;
mod text_processing;
mod types;
mod utils;
mod visualization;

use crate::config::{CLI_PROMPT, DEFAULT_COLLECTION_NAME, DEFAULT_K_SEARCH_RESULTS};
use crate::utils::extract_text_chuck_from_file::extract_text_chunks_from_file;
use crate::utils::file_validation::validate_file;
use crate::visualization::pca::pca_2d;

use cli::{Commands, parse_command};
use db::collection::Collection;
use std::io::{self, Write};
use std::path::PathBuf;
use text_processing::embedder::{BatchSize, Embedder};
use types::document::DocumentInsert;
use uuid::Uuid;
use walkdir::WalkDir;

fn embed_file(absolute_path: PathBuf, embedder: &mut Embedder, collection: &mut Collection) {
    let chunks = extract_text_chunks_from_file(&absolute_path);

    for c in chunks {
        // Should I parallelise ?
        match embedder.embed(&c.content, BatchSize::None) {
            Ok(embedding) => {
                collection.insert(DocumentInsert {
                    chunk_index: c.index,
                    path: format!("{}", absolute_path.display()),
                    embedding,
                });
                match collection.save() {
                    Err(e) => println!("{}", e),
                    Ok(_) => println!("New file embedded."),
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}

fn run(mut collection: Collection, mut embedder: Embedder) {
    loop {
        print!("{}", CLI_PROMPT);
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
                Commands::All => {
                    for document in collection.documents.iter() {
                        println!("{} - Chunk #{}", document.path, document.chunk_index);
                    }
                }

                Commands::Map => {
                    let embeddings = collection.embeddings();
                    let points = pca_2d(&embeddings);
                    for (document, point) in collection.documents.iter().zip(points.iter()) {
                        println!("{} → ({:.3}, {:.3})", document.path, point.x, point.y);
                    }
                }

                Commands::IndexDirectory { path } => {
                    for entry in WalkDir::new(path) {
                        let entry = entry.unwrap();
                        let absolute_path = entry.path().canonicalize().unwrap();

                        if collection.find(|d| d.path == absolute_path).is_some() {
                            eprintln!("Error: file already embeded");
                            continue;
                        }

                        if validate_file(&absolute_path).is_err() {
                            continue;
                        }
                        embed_file(absolute_path, &mut embedder, &mut collection);
                    }
                }

                Commands::EmbedFile { path } => {
                    let absolute_path = path.canonicalize().unwrap_or(path.clone());

                    if collection.find(|d| d.path == absolute_path).is_some() {
                        eprintln!("Error: file already embeded");
                        continue;
                    }

                    if validate_file(&absolute_path).is_err() {
                        eprintln!("Error: invalid path");
                        continue;
                    }
                    embed_file(absolute_path, &mut embedder, &mut collection);
                }

                Commands::Lucky { query } => match embedder.embed(&query, BatchSize::None) {
                    Ok(embedding) => {
                        let result = collection.search(&embedding, 1);
                        println!("Best match: {:#?}", result);
                    }
                    Err(e) => eprintln!("Error: {}", e),
                },

                Commands::Search { query } => match embedder.embed(&query, BatchSize::None) {
                    Ok(embedding) => {
                        let results = collection.search(&embedding, DEFAULT_K_SEARCH_RESULTS);
                        println!("Best matches: {:#?}", results);
                    }
                    Err(e) => eprintln!("Error: {}", e),
                },

                Commands::DeleteById { id } => {
                    let uuid = Uuid::parse_str(&id).unwrap();
                    collection.delete(uuid);

                    match collection.save() {
                        Err(e) => println!("{}", e),
                        Ok(_) => println!("Item deleted."),
                    }
                }

                Commands::Delete { path_str } => {
                    collection.delete_at_path(path_str);

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

fn main() {
    let embedder = match Embedder::new() {
        Ok(model) => model,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ();
        }
    };
    let db_path = format!("{}.db", DEFAULT_COLLECTION_NAME);

    let collection: Collection = match Collection::load(&db_path) {
        Ok(c) => c,
        Err(_) => Collection::new(DEFAULT_COLLECTION_NAME, &db_path),
    };

    run(collection, embedder);
}
