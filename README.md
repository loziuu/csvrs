# csvrs

A lightweight, in-memory CSV query engine with a SQL-like interface written in Rust. Load CSV files and query them interactively using a simple query language.

## Features

- **Columnar Storage**: Efficient memory layout using fixed-size 8KB buffer pools
- **Interactive REPL**: Query your data interactively with rustyline-powered editing
- **SQL-like Queries**: Familiar GET/WHERE syntax for filtering and selecting data
- **Logical Operators**: Support for AND/OR conditions in WHERE clauses

## Building

```bash
cargo build --release
```

## Usage

### Opening a CSV File

```bash
# Load a CSV file and start the REPL
./target/release/csvrs --dir /path/to/your/file.csv

# Or simply
cargo run -- --dir /path/to/your/file.csv
```

### CSV Format Requirements

- **Delimiter**: Semicolon (`;`)
- **Headers**: Required (first row must contain column names)
- **Encoding**: UTF-8

Example CSV:
```csv
name;age;city
Alice;25;NYC
Bob;30;LA
Charlie;35;Chicago
```

### REPL Commands

Once the file is loaded, you'll see the available columns and a prompt:

```
Available columns: {"name": 0, "age": 1, "city": 2}
@>
```

Type `exit` to quit the REPL.

## Query Syntax

```
GET <columns> [@ <table>] [WHERE <conditions>]
```

### Selecting Columns

```sql
-- Select a single column
get name

-- Select multiple columns (space-separated)
get name age city

-- Use quoted identifiers for column names with spaces
get "first name" "last name" age
```

### Table Specification

The `@` operator specifies a table (currently symbolic as all data is in a single working set):

```sql
get name @ users
get name @ "my table"
```

### WHERE Clause

Filter rows using equality conditions:

```sql
-- Single condition
get name where age = "25"

-- AND condition (both must match)
get name where age = "25" and city = "NYC"

-- OR condition (either can match)
get name where city = "NYC" or city = "LA"

-- Combined conditions
get name city where age = "25" and city = "NYC" or city = "Chicago"
```

### Query Examples

| Query | Description |
|-------|-------------|
| `get name` | Select the name column |
| `get name age city` | Select multiple columns |
| `get name @ users` | Select from a specific table |
| `get name where age = "25"` | Filter by age |
| `get name where first = "John" and last = "Doe"` | Multiple AND conditions |
| `get name where city = "NYC" or city = "LA"` | OR conditions |

### Output Format

Results are displayed in a pipe-delimited format:

```
@> get name city where age = "25"
| Alice | NYC |
Got 1 records.
```

## Supported Operators

| Operator | Description |
|----------|-------------|
| `=` | Equality comparison |
| `and` | Logical AND |
| `or` | Logical OR |
| `@` | Table selector |

## Architecture

```
src/
├── main.rs           # REPL entry point and CLI
├── mem.rs            # CSV loading and in-memory data structures
├── executor.rs       # Query execution engine (Visitor pattern)
├── query/
│   ├── scanner.rs    # Lexical analysis (tokenization)
│   ├── parser.rs     # Recursive descent parser
│   └── token.rs      # Token type definitions
└── index/
    ├── heap.rs       # Buffer pool and block management
    └── tree.rs       # B-tree infrastructure (WIP)
```

## Current Limitations

- Only equality (`=`) comparisons (no `<`, `>`, `<=`, `>=`, `!=`)
- No aggregation functions (SUM, COUNT, AVG, etc.)
- No ORDER BY or GROUP BY
- Single file loading (no directory scanning)
- Semicolon delimiter is hardcoded

## Dependencies

- [clap](https://crates.io/crates/clap) - Command line argument parsing
- [csv](https://crates.io/crates/csv) - CSV file parsing
- [rustyline](https://crates.io/crates/rustyline) - REPL line editing

## License

This project is for educational and personal use.
