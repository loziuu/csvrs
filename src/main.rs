use clap::Parser;
use std::{collections::HashMap, fs::File, io::BufReader, path::PathBuf};

mod index;
mod parser;
mod scanner;
mod token;

#[derive(Parser, Debug)]
struct Args {
    /// Directory to scan for csv files
    #[arg(short, long, default_value = "~")]
    dir: String,

    /// Include archives
    #[arg(short, long, default_value = "false")]
    _include_archives: bool,
}

enum WorkingEnv {
    Csv(PathBuf),
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let buf: PathBuf = args.dir.into();

    let set = index::index(buf)?;

    println!("Working set loaded.");
    println!("Available columns: {:?}", set.columns.keys());
    loop {
        println!("Enter column name to index (or 'exit' to quit): ");

        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input)?;

        if input.trim() == "exit" {
            return Ok(());
        }

        println!("Searching for column: {}", input.trim());
        match set.columns.get(input.trim()) {
            Some(idx) => {
                for v in set.values.iter() {
                    println!("{}", v[*idx]);
                }
            }
            None => println!("Column not found."),
        }
    }
}

struct WorkingSet {
    columns: HashMap<String, usize>,
    values: Vec<Vec<String>>,
}
