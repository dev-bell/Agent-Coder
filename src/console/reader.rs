use std::io::{self};

/// Reads a line from stdin, trims trailing newline, returns the String.
pub fn read_line() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim_end().to_string()
}