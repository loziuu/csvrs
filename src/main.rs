use crate::executor::{ColumnarExecutor, IndexVisitor};
use crate::mem::read_columnar;
use clap::Parser;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::query::parser::CmdParser;

mod executor;
pub mod index;
pub mod mem;
pub mod query;

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

    let set = mem::index_heap_columnar(buf)?;

    println!("Working set loaded.");
    println!("Available columns: {:?}", set.columns.keys());
    println!(" ('exit' to quit): ");
    loop {
        print!("> ");
        let index_visitor = ColumnarExecutor { set: &set };

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

        let mut cnt = 0;
        for c in columns {
            cnt += 1;

            for e in c {
                out.write_all(b" | ")?;
                out.write_all(read_columnar(&set, e.0, e.1)).unwrap();
            }
            out.write_all(b"\n")?;
        }
        out.write_all(format!("Got {} records.", cnt).as_bytes())?;
        out.write_all(b"\n")?;
    }
}

fn row_main() -> std::io::Result<()> {
    let args = Args::parse();
    let buf: PathBuf = args.dir.into();

    let set = mem::index_heap_row(buf)?;

    mem::read_all(&set);

    Ok(())
}

fn hashmap_main() -> std::io::Result<()> {
    let args = Args::parse();
    let buf: PathBuf = args.dir.into();

    let set = mem::index(buf)?;

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
