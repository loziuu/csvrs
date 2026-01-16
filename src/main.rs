use crate::executor::ColumnarExecutor;
use crate::mem::read_columnar;
use clap::Parser;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
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

    let mut rl = DefaultEditor::new().expect("Failed to create editor");
    let stdout = io::stdout();

    loop {
        let input = match rl.readline("@> ") {
            Ok(line) => {
                let _ = rl.add_history_entry(&line);
                line
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                return Ok(());
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                return Ok(());
            }
        };

        if input.trim() == "exit" {
            return Ok(());
        }

        let mut out = stdout.lock();
        let index_visitor = ColumnarExecutor { set: &set };

        let parsed = CmdParser::new();
        let statement = parsed.parse_string(input.trim());

        if statement.is_err() {
            let err = statement.err().unwrap();
            out.write_all(err.to_string().as_bytes()).unwrap();
            out.write_all(b"\n")?;
            continue;
        }

        let columns = statement.unwrap().accept(&index_visitor);

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
