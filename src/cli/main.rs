use std::fmt;

use crate::text_processing::tokenize::tokenize;
use clap::{Parser, Subcommand};

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
    Delete { id: String },
    Quit,
}

#[derive(Debug)]
pub enum CmdError {
    Invalid(String),
    // MissingArgument(String),
    UnknownCommand(String),
}

impl fmt::Display for CmdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CmdError::Invalid(message) => {
                write!(f, "invalid command: {message}")
            }
            // CmdError::MissingArgument(argument) => {
            //     write!(f, "missing argument: {argument}")
            // }
            CmdError::UnknownCommand(command) => {
                write!(f, "unknown command: {command}")
            }
        }
    }
}

impl std::error::Error for CmdError {}

pub fn parse_command(user_input: &str) -> Result<Commands, CmdError> {
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
            ["delete", id] => Ok(Commands::Delete { id: id.to_string() }),
            _ => Err(CmdError::Invalid(tokens.as_slice().join(" "))),
        }
    } else {
        Err(CmdError::UnknownCommand(user_input.to_string()))
    }
}
