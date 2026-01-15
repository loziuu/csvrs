use crate::{
    index::heap::TOffset,
    mem::{ColumnsWorkingSet, WorkingSet},
    query::parser::{self, Visitor},
};

pub(crate) struct ColumnarExecutor<'a> {
    pub(crate) set: &'a ColumnsWorkingSet,
}

impl Visitor<Vec<Vec<(usize, (usize, TOffset))>>> for ColumnarExecutor<'_> {
    fn visit(&self, expr: &parser::Statement) -> Vec<Vec<(usize, (usize, TOffset))>> {
        match expr {
            parser::Statement::Get(expr, _, _) => {
                let get_columns = get_column_names(expr);

                let ids: Vec<usize> = get_columns
                    .iter()
                    .map(|col| {
                        let val = self.set.columns.get(col).expect("Missing column");
                        *val
                    })
                    .collect();

                let mut res = vec![];
                for row in &self.set.rows {
                    let mut obj = vec![];

                    for i in &ids {
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
