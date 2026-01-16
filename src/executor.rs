use core::panic;

use crate::{
    index::heap::BlockOffset,
    mem::{ColumnsWorkingSet, WorkingSet, read_columnar},
    query::parser::{self, Expr, Visitor},
};

pub(crate) struct ColumnarExecutor<'a> {
    pub(crate) set: &'a ColumnsWorkingSet,
}

type RowResult = Vec<Vec<(usize, (usize, BlockOffset))>>;

/// Type after parsing where statement. It returns (usize, PredicateFunc).
/// First usize is id of BufferPool for given column
type RowFilter = (usize, Box<dyn Fn(&[u8]) -> bool>);

impl ColumnarExecutor<'_> {
    fn build_filter(&self, cond: &Expr) -> RowFilter {
        match cond {
            Expr::Conditional(left, _op, right) => {
                let col_name = self.parse_term(left);
                let col_idx = *self
                    .set
                    .columns
                    .get(&col_name)
                    .expect("Missing filter column");

                let expected = self.parse_term(right);
                let expected_bytes = expected.into_bytes();

                (
                    col_idx,
                    Box::new(move |actual: &[u8]| actual == expected_bytes.as_slice()),
                )
            }
            _ => panic!("Expected conditional"),
        }
    }

    fn parse_term(&self, term: &Expr) -> String {
        if let Expr::Literal(token) = term {
            return token.literal.to_string();
        }
        panic!("Invalid token");
    }
}

impl Visitor<RowResult> for ColumnarExecutor<'_> {
    fn visit(&self, expr: &parser::Statement) -> RowResult {
        match expr {
            parser::Statement::Get(expr, _table, conditions) => {
                let get_columns = get_column_names(expr);

                let selected_cols_ids: Vec<usize> = get_columns
                    .iter()
                    .map(|col| {
                        let val = self.set.columns.get(col).expect("Missing column");
                        *val
                    })
                    .collect();

                let filter = conditions.as_ref().map(|c| self.build_filter(c));

                let mut res = vec![];
                for row in &self.set.rows {
                    // Apply WHERE filter
                    if let Some((filter_idx, ref predicate)) = filter {
                        let (block_id, offset) = row[filter_idx];

                        let actual_bytes = read_columnar(self.set, filter_idx, (block_id, offset));
                        if !predicate(actual_bytes) {
                            continue;
                        }
                    }

                    let mut obj = vec![];
                    for i in &selected_cols_ids {
                        obj.push((*i, row[*i]));
                    }

                    res.push(obj);
                }
                res
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

pub(crate) struct IndexVisitor<'a> {
    pub(crate) set: &'a WorkingSet,
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
