use crate::text_processing::tokenize::tokenize;
use clap::{Parser, Subcommand};
use std::{fmt, path::PathBuf};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    All,
    Lucky { query: String },
    Search { query: String },
    DeleteById { id: String },
    Delete { path_str: String },
    EmbedFile { path: PathBuf },
    IndexDirectory { path: PathBuf },
    Map,
    Quit,
}

#[derive(Debug)]
pub enum CmdError {
    Invalid(String),
    UnknownCommand(String),
}

impl fmt::Display for CmdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CmdError::Invalid(message) => {
                write!(f, "invalid command: {message}")
            }
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
            ["all"] => Ok(Commands::All),
            ["quit"] | ["exit"] => Ok(Commands::Quit),
            ["embed", path] => Ok(Commands::EmbedFile {
                path: PathBuf::from(path),
            }),
            ["index", path] => Ok(Commands::IndexDirectory {
                path: PathBuf::from(path),
            }),
            ["search", item] => Ok(Commands::Search {
                query: item.to_string(),
            }),
            ["lucky", item] => Ok(Commands::Lucky {
                query: item.to_string(),
            }),
            ["delete", path] => Ok(Commands::Delete {
                path_str: path.to_string(),
            }),
            ["map"] => Ok(Commands::Map),
            ["deleteById", id] => Ok(Commands::DeleteById { id: id.to_string() }),
            _ => Err(CmdError::Invalid(tokens.as_slice().join(" "))),
        }
    } else {
        Err(CmdError::UnknownCommand(user_input.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_commands() {
        assert!(matches!(parse_command("all").unwrap(), Commands::All));
        assert!(matches!(parse_command("map").unwrap(), Commands::Map));
        assert!(matches!(parse_command("exit").unwrap(), Commands::Quit));
    }

    #[test]
    fn parses_search_query_with_spaces() {
        match parse_command(r#"search "hello world""#).unwrap() {
            Commands::Search { query } => assert_eq!(query, "hello world"),
            other => panic!("expected search command, got {other:?}"),
        }
    }

    #[test]
    fn parses_file_paths() {
        match parse_command("embed ./notes.md").unwrap() {
            Commands::EmbedFile { path } => assert_eq!(path, PathBuf::from("./notes.md")),
            other => panic!("expected embed command, got {other:?}"),
        }
    }

    #[test]
    fn rejects_unknown_commands() {
        assert!(parse_command("frobnicate").is_err());
        assert!(parse_command("search").is_err());
    }
}
