mod cli;
mod db;
mod text_processing;
mod types;
mod utils;
mod visualization;

// use anyhow::bail;
use cli::{Commands, parse_command};
use db::collection::Collection;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::io::{self, Write};
// use std::path::PathBuf;
use text_processing::embeddings::{BatchSize, TextProcessing};
use types::document::DocumentInsert;
use uuid::Uuid;

use crate::utils::chunk_text::chunk_text;
use crate::utils::file_validation::validate_file;
use crate::visualization::pca::pca_2d;

pub struct Database {
    pub collections: HashMap<String, Collection>,
}

// struct Embedder;

// impl Embedder {
//     fn embed_file(
//         path: PathBuf,
//         mut text_processing: TextProcessing,
//         mut collection: &Collection,
//     ) -> anyhow::Result<()> {
//         let absolute_path = path.canonicalize().unwrap_or(path.clone());

//         if validate_file(&absolute_path).is_err() {
//             bail!("Error: invalid path");
//         }

//         let extension = path.extension().and_then(|ext| ext.to_str()).unwrap();

//         let doc = match extension {
//             "pdf" => pdf_extract::extract_text(&absolute_path).unwrap(),
//             _ => read_to_string(&absolute_path).unwrap(),
//         };

//         let chunks = chunk_text(extension, &doc);

//         for c in chunks {
//             match text_processing.embed(&c.content, BatchSize::None) {
//                 Ok(embedding) => {
//                     collection.insert(DocumentInsert {
//                         chunk_index: c.index,
//                         path: format!("{}", absolute_path.display()),
//                         embedding,
//                     });
//                     match collection.save() {
//                         Err(e) => println!("{}", e),
//                         Ok(_) => println!("New file embedded."),
//                     }
//                 }
//                 Err(e) => eprintln!("Error: {}", e),
//             }
//         }

//         Ok(())
//     }
// }

fn main() {
    let mut text_processing = match TextProcessing::new() {
        Ok(model) => model,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ();
        }
    };

    let db_path = "my_collection.db";

    let mut collection: Collection = match Collection::load(db_path) {
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
                Commands::Map => {
                    let embeddings: Vec<Vec<f32>> = collection
                        .documents
                        .iter()
                        .map(|doc| doc.embedding.clone())
                        .collect();

                    let points = pca_2d(&embeddings);

                    for (document, point) in collection.documents.iter().zip(points.iter()) {
                        println!("{} → ({:.3}, {:.3})", document.path, point.x, point.y);
                    }
                }

                Commands::EmbedFile { path } => {
                    let absolute_path = path.canonicalize().unwrap_or(path.clone());

                    if collection
                        .documents
                        .iter()
                        .find(|d| d.path == absolute_path)
                        .is_some()
                    {
                        eprintln!("Error: file already embeded");
                        continue;
                    }

                    if validate_file(&absolute_path).is_err() {
                        eprintln!("Error: invalid path");
                        continue;
                    }

                    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap();

                    let doc = match extension {
                        "pdf" => pdf_extract::extract_text(&absolute_path).unwrap(),
                        _ => read_to_string(&absolute_path).unwrap(),
                    };

                    let chunks = chunk_text(extension, &doc);

                    for c in chunks {
                        match text_processing.embed(&c.content, BatchSize::None) {
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
                Commands::Search { query } => {
                    match text_processing.embed(&query, BatchSize::None) {
                        Ok(embedding) => {
                            let results = collection.search(&embedding, 5);
                            println!("Best matches: {:#?}", results);
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
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
