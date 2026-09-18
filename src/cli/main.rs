use clap::{Parser, Subcommand};

use crate::text_processing::tokenize::tokenize;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Insert { doc: String },
    Search { query: String },
    Quit,
}

pub fn parse_command(user_input: &str) -> Result<Commands, String> {
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
