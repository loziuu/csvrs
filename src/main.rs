use clap::Parser;
use std::{collections::HashMap, fs::File, io::BufReader, path::PathBuf};
use std::{
    io::{self, Write},
    path::PathBuf,
};

use crate::{
    index::WorkingSet,
    parser::{CmdParser, Visitor},
};

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
    println!(" ('exit' to quit): ");
    loop {
        print!("> ");
        let index_visitor = IndexVisitor { set: &set };

        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input)?;

        if input.trim() == "exit" {
            return Ok(());
        }

        println!("Searching for column: {}", input.trim());
        let parsed = CmdParser::new();

        let columns = parsed.parse_string(input.trim()).accept(&index_visitor);

        let stdout = io::stdout();
        let mut out = stdout.lock();
        for val in set.values.iter() {
            for &col in columns.iter() {
                out.write_all(b" | ")?;
                out.write_all(val[col].as_bytes())?;
            }

            out.write_all(b"\n")?;
        }
    }
}

struct IndexVisitor<'a> {
    set: &'a WorkingSet,
}

impl Visitor<Vec<usize>> for IndexVisitor<'_> {
    fn visit(&self, expr: &parser::Statement) -> Vec<usize> {
        match expr {
            parser::Statement::Get(expr, _, _) => {
                let get_columns = get_column_names(expr);

                get_columns
                    .iter()
                    .map(|col| {
                        let val = self.set.columns.get(col).expect("Missing coulmn");
                        *val
                    })
                    .collect()
            }
        }
    }
}

fn get_column_names(expr: &parser::Expr) -> Vec<String> {
    match expr {
        parser::Expr::Literal(token) => vec![token.literal.to_string()],
        parser::Expr::Multiple(left, right) => {
            let mut names = get_column_names(left);
            names.extend(get_column_names(right));
            names
        }
        _ => panic!("Invalid syntax"),
    }
}
