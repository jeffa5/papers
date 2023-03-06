#![deny(missing_docs)]

//! Library items for the CLI

/// CLI resources.
pub mod cli;
/// Config file resources.
pub mod config;
/// Multiple ids.
pub mod ids;

/// Type for handling either urls or local file paths.
pub mod url_path;

/// Type for handling either files or stdin.
pub mod file_or_stdin;

/// Type for displaying papers in a table.
pub mod table;

/// Collection of error reporting utils.
pub mod err;

/// Interactive input handling.
pub mod interactive;

/// Rename files to match db entries.
pub mod rename_files;

/// Fuzzy searching.
pub mod fuzzy;
