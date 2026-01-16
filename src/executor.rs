use core::panic;

use crate::{
    index::heap::BlockOffset,
    mem::{ColumnsWorkingSet, read_columnar},
    query::parser::{self, Expr, Visitor},
    query::token::TokenType,
};

pub(crate) struct ColumnarExecutor<'a> {
    pub(crate) set: &'a ColumnsWorkingSet,
}

type RowResult = Vec<Vec<(usize, (usize, BlockOffset))>>;

/// Type after parsing where statement. It returns (usize, PredicateFunc).
/// First usize is id of BufferPool for given column
type RowFilter = (usize, Box<dyn Fn(&[u8]) -> bool>);

#[derive(Clone, Copy, Debug)]
enum LogicalOp {
    And,
    Or,
    None, // First condition has no preceding operator
}

type CompoundFilter = Vec<(LogicalOp, RowFilter)>;

impl ColumnarExecutor<'_> {
    fn build_filters(&self, cond: &Expr) -> CompoundFilter {
        let mut filters = Vec::new();
        self.collect_filters(cond, LogicalOp::None, &mut filters);
        filters
    }

    fn collect_filters(&self, cond: &Expr, op: LogicalOp, filters: &mut CompoundFilter) {
        match cond {
            Expr::Conditional(left, token, right) => {
                match token.t {
                    TokenType::And | TokenType::Or => {
                        // Nested conditional - recurse into both sides
                        self.collect_filters(left, op, filters);
                        let next_op = if token.t == TokenType::And {
                            LogicalOp::And
                        } else {
                            LogicalOp::Or
                        };
                        self.collect_filters(right, next_op, filters);
                    }
                    TokenType::Equals => {
                        // Leaf comparison: col = value
                        let filter = self.build_comparison(left, right);
                        filters.push((op, filter));
                    }
                    _ => panic!("Unexpected operator in conditional: {:?}", token.t),
                }
            }
            _ => panic!("Expected conditional"),
        }
    }

    fn build_comparison(&self, left: &Expr, right: &Expr) -> RowFilter {
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

    fn parse_term(&self, term: &Expr) -> String {
        if let Expr::Literal(token) = term {
            return token.literal.to_string();
        }
        panic!("Invalid token");
    }

    fn evaluate_filters(&self, filters: &CompoundFilter, row: &[(usize, BlockOffset)]) -> bool {
        let mut result = true;

        for (op, (col_idx, predicate)) in filters {
            let (block_id, offset) = row[*col_idx];
            let actual_bytes = read_columnar(self.set, *col_idx, (block_id, offset));
            let matches = predicate(actual_bytes);

            match op {
                LogicalOp::None => result = matches,
                LogicalOp::And => result = result && matches,
                LogicalOp::Or => result = result || matches,
            }
        }

        result
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

                let filters = conditions.as_ref().map(|c| self.build_filters(c));

                let mut res = vec![];
                for row in &self.set.rows {
                    // Apply WHERE filters
                    if let Some(ref compound_filter) = filters
                        && !self.evaluate_filters(compound_filter, row)
                    {
                        continue;
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
