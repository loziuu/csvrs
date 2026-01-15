use crate::index::heap::{self, Heap, TOffset};
use std::{collections::HashMap, fs::File, io::BufReader, path::PathBuf};

pub(crate) struct WorkingSet {
    pub(crate) columns: HashMap<String, usize>,
    pub(crate) values: Vec<Vec<String>>,
}

pub(crate) struct RowWorkingSet {
    pub(crate) rows: Vec<(usize, TOffset)>,
    heap: Heap,
}

pub(crate) struct ColumnsWorkingSet {
    pub(crate) columns: HashMap<String, usize>,
    pub(crate) data: Vec<Heap>,
    pub(crate) rows: Vec<Vec<(usize, TOffset)>>,
}

pub(crate) fn read_columnar(set: &ColumnsWorkingSet, heap: usize, ptr: (usize, TOffset)) -> &[u8] {
    let heap = &set.data[heap];
    heap.read(ptr.0, ptr.1).unwrap()
}

pub(crate) fn index_heap_columnar(buf: PathBuf) -> std::io::Result<ColumnsWorkingSet> {
    if buf.is_file() {
        let mut columns = HashMap::new();

        let file = File::open(buf)?;
        let mut csv_reader = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_reader(BufReader::new(file));
        if !csv_reader.has_headers() {
            panic!("CSV without headers not supported yet.");
        }

        csv_reader.headers().into_iter().for_each(|value| {
            for (i, v) in value.iter().enumerate() {
                columns.insert(v.trim().to_string(), i);
            }
        });

        let mut data = Vec::with_capacity(columns.len());
        for _ in 0..columns.len() {
            data.push(Heap::new());
        }

        let mut rows = Vec::with_capacity(columns.len());
        for values in csv_reader.records() {
            let mut row = vec![];
            let record = values?;

            for (i, value) in record.iter().enumerate() {
                row.push(data[i].allocate(value.as_bytes()));
            }
            rows.push(row);
        }

        Ok(ColumnsWorkingSet {
            columns,
            data,
            rows,
        })
    } else {
        panic!("Path is not a file. Directory scanning not implemented yet.");
    }
}

pub(crate) fn index_heap_row(buf: PathBuf) -> std::io::Result<RowWorkingSet> {
    let mut heap = Heap::new();

    if buf.is_file() {
        let file = File::open(buf)?;

        let mut csv_reader = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_reader(BufReader::new(file));

        let mut columns = HashMap::new();
        if !csv_reader.has_headers() {
            panic!("CSV without headers not supported yet.");
        }

        csv_reader.headers().into_iter().for_each(|value| {
            for (i, v) in value.iter().enumerate() {
                columns.insert(v.trim().to_string(), i);
            }
        });

        let mut rows = vec![];
        for values in csv_reader.records() {
            let v = values?;
            let bytes = v.into_byte_record();

            rows.push(heap.allocate(bytes.as_slice()));
        }

        Ok(RowWorkingSet { rows, heap })
    } else {
        panic!("Path is not a file. Directory scanning not implemented yet.");
    }
}

pub(crate) fn read_all(set: &RowWorkingSet) {
    for (block_id, offset) in &set.rows {
        if let Some(bytes) = set.heap.read(*block_id, *offset) {
            println!("{}", str::from_utf8(bytes).unwrap());
        } else {
            panic!("Failed to serialize bytes to utf8");
        }
    }
}

pub(crate) fn index(buf: PathBuf) -> std::io::Result<WorkingSet> {
    println!("Opening working set from {:?}", buf);

    if buf.is_file() {
        let file = File::open(buf)?;

        let mut csv_reader = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_reader(BufReader::new(file));

        let mut columns = HashMap::new();
        if !csv_reader.has_headers() {
            panic!("CSV without headers not supported yet.");
        }

        csv_reader.headers().into_iter().for_each(|value| {
            for (i, v) in value.iter().enumerate() {
                columns.insert(v.trim().to_string(), i);
            }
        });

        let mut rows = vec![];
        for values in csv_reader.records() {
            let mut row = vec![];
            let record = values?;
            for value in record.iter() {
                row.push(value.to_string());
            }
            rows.push(row);
        }

        Ok(WorkingSet {
            columns,
            values: rows,
        })
    } else {
        panic!("Path is not a file. Directory scanning not implemented yet.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_index_valid_csv_file() -> std::io::Result<()> {
        let mut file = NamedTempFile::new()?;
        writeln!(file, "name;age;city")?;
        writeln!(file, "Alice;25;NYC")?;
        writeln!(file, "Bob;30;LA")?;

        let working_set = index(file.path().to_path_buf())?;

        assert_eq!(working_set.columns.len(), 3);
        assert_eq!(working_set.values.len(), 2);
        assert!(working_set.columns.contains_key("name"));
        assert!(working_set.columns.contains_key("age"));
        assert!(working_set.columns.contains_key("city"));
        Ok(())
    }

    #[test]
    fn test_column_index_mapping() -> std::io::Result<()> {
        let mut file = NamedTempFile::new()?;
        writeln!(file, "first;second;third")?;
        writeln!(file, "a;b;c")?;

        let working_set = index(file.path().to_path_buf())?;

        assert_eq!(*working_set.columns.get("first").unwrap(), 0);
        assert_eq!(*working_set.columns.get("second").unwrap(), 1);
        assert_eq!(*working_set.columns.get("third").unwrap(), 2);
        Ok(())
    }

    #[test]
    fn test_headers_with_whitespace() -> std::io::Result<()> {
        let mut file = NamedTempFile::new()?;
        writeln!(file, " name ; age ; city ")?; // Headers with spaces
        writeln!(file, "Alice;25;NYC")?;

        let working_set = index(file.path().to_path_buf())?;

        assert!(working_set.columns.contains_key("name"));
        assert!(working_set.columns.contains_key("age"));
        assert!(working_set.columns.contains_key("city"));
        assert!(!working_set.columns.contains_key(" name "));
        Ok(())
    }

    #[test]
    fn test_data_values_preserved() -> std::io::Result<()> {
        let mut file = NamedTempFile::new()?;
        writeln!(file, "name;score")?;
        writeln!(file, "Alice;100")?;
        writeln!(file, "Bob;95")?;

        let working_set = index(file.path().to_path_buf())?;

        let name_idx = *working_set.columns.get("name").unwrap();
        let score_idx = *working_set.columns.get("score").unwrap();
        assert_eq!(working_set.values[0][name_idx], "Alice");
        assert_eq!(working_set.values[0][score_idx], "100");
        assert_eq!(working_set.values[1][name_idx], "Bob");
        assert_eq!(working_set.values[1][score_idx], "95");
        Ok(())
    }

    #[test]
    #[should_panic(expected = "Path is not a file. Directory scanning not implemented yet.")]
    fn test_nonexistent_file() {
        let result = index(PathBuf::from("/nonexistent/file.csv"));

        assert!(result.is_err()); // Should return IO error
    }
}
