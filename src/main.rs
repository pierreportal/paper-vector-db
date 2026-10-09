mod cli;
mod config;
mod db;
mod text_processing;
mod types;
mod utils;
mod visualization;

use crate::config::{CLI_PROMPT, DEFAULT_COLLECTION_NAME, DEFAULT_K_SEARCH_RESULTS};
use crate::db::collection::SearchResult;
use crate::utils::extract_text_chuck_from_file::extract_text_chunks_from_file;
use crate::utils::file_validation::validate_file;
use crate::visualization::pca::pca_2d;

use anyhow::bail;
use cli::{Commands, parse_command};
use db::collection::Collection;
use std::io::{self, Write};
use std::path::PathBuf;
use text_processing::embedder::{BatchSize, Embedder};
use types::document::DocumentInsert;
use uuid::Uuid;
use walkdir::WalkDir;

struct Program {
    collection: Collection,
    embedder: Embedder,
}

impl Program {
    fn new(collection: Collection, embedder: Embedder) -> Self {
        Self {
            collection,
            embedder,
        }
    }

    fn embed_file_chunks(&mut self, absolute_path: PathBuf) -> anyhow::Result<()> {
        if self.collection.find(|d| d.path == absolute_path).is_some() {
            bail!("Error: file already embeded")
        }
        if validate_file(&absolute_path).is_err() {
            bail!("Error: invalid path")
        }
        let chunks = extract_text_chunks_from_file(&absolute_path)?;
        for c in chunks {
            // Should I parallelise ?
            match self.embedder.embed(&c.content, BatchSize::None) {
                Ok(embedding) => {
                    self.collection.insert(DocumentInsert {
                        chunk_index: c.index,
                        path: format!("{}", absolute_path.display()),
                        embedding,
                    });
                }
                Err(e) => bail!("Error: {}", e),
            }
        }
        self.collection.save()
    }

    fn embed_file(&mut self, path: PathBuf) -> anyhow::Result<()> {
        let absolute_path = path.canonicalize().unwrap_or(path.clone());
        self.embed_file_chunks(absolute_path)
    }

    fn index_directory(&mut self, path: PathBuf) -> anyhow::Result<()> {
        for entry in WalkDir::new(path).into_iter().filter_map(|entry| entry.ok()) {
            let entry_path = entry.path();

            // Skip directories and anything that isn't a supported, readable file.
            if !entry_path.is_file() || validate_file(entry_path).is_err() {
                continue;
            }

            let Ok(absolute_path) = entry_path.canonicalize() else {
                continue;
            };

            match self.embed_file_chunks(absolute_path) {
                Ok(_) => {}
                Err(e) => {
                    println!("{}", e);
                    continue;
                }
            }
        }
        Ok(())
    }

    fn search(&mut self, query: &str, k: usize) -> anyhow::Result<Vec<SearchResult>> {
        match self.embedder.embed(query, BatchSize::None) {
            Ok(embedding) => {
                let results = self.collection.search(&embedding, k);
                Ok(results)
            }
            Err(e) => bail!(e),
        }
    }

    fn run(&mut self) {
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
                        for document in self.collection.documents.iter() {
                            println!("{} - Chunk #{}", document.path, document.chunk_index);
                        }
                    }

                    Commands::Map => {
                        let embeddings = self.collection.embeddings();
                        let points = pca_2d(&embeddings);
                        for (document, point) in self.collection.documents.iter().zip(points.iter())
                        {
                            println!("{} → ({:.3}, {:.3})", document.path, point.x, point.y);
                        }
                    }

                    Commands::IndexDirectory { path } => match self.index_directory(path) {
                        Ok(_) => {}
                        Err(e) => println!("{}", e),
                    },

                    Commands::EmbedFile { path } => match self.embed_file(path) {
                        Ok(_) => {}
                        Err(e) => println!("{}", e),
                    },

                    Commands::Lucky { query } => match self.search(&query, 1) {
                        Ok(results) => println!("Best match: {:#?}", results),
                        Err(e) => println!("An error occurded: {}", e),
                    },

                    Commands::Search { query } => {
                        match self.search(&query, DEFAULT_K_SEARCH_RESULTS) {
                            Ok(results) => println!("Best matches: {:#?}", results),
                            Err(e) => println!("An error occurded: {}", e),
                        }
                    }

                    Commands::DeleteById { id } => match Uuid::parse_str(&id) {
                        Ok(uuid) => {
                            self.collection.delete(uuid);

                            match self.collection.save() {
                                Err(e) => println!("{}", e),
                                Ok(_) => println!("Item deleted."),
                            }
                        }
                        Err(e) => println!("Error: invalid id: {}", e),
                    },

                    Commands::Delete { path_str } => {
                        self.collection.delete_at_path(path_str);

                        match self.collection.save() {
                            Err(e) => println!("{}", e),
                            Ok(_) => println!("Item deleted."),
                        }
                    }

                    Commands::Quit => break,
                },
            }
        }
    }
}

fn main() {
    let embedder = match Embedder::new() {
        Ok(model) => model,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };

    let db_path = format!("{}.db", DEFAULT_COLLECTION_NAME);

    let collection: Collection = match Collection::load(&db_path) {
        Ok(c) => c,
        Err(_) => Collection::new(DEFAULT_COLLECTION_NAME, &db_path),
    };

    Program::new(collection, embedder).run();
}
