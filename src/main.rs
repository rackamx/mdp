use clap::{Arg, Command};
use std::fs;
use std::io;
use std::process;

fn main() {
    let matches = Command::new("mdless")
        .arg(Arg::new("file")
            .help("Path to the markdown file to display")
            .required(false))
        .get_matches();

    match matches.get_one::<String>("file") {
        Some(file_path) => {
            match read_file(file_path) {
                Ok(contents) => {
                    println!("{}", contents);
                }
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    process::exit(1);
                }
            }
        }
        None => {
            println!("Usage: mdless <file>");
            process::exit(0);
        }
    }
}

fn read_file(path: &str) -> Result<String, io::Error> {
    fs::read_to_string(path)
}
