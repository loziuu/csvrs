use crate::{WorkingSet, token::TokenLiteral};

struct Statement<'a, T> {
    set: &'a WorkingSet,
    exectute: Exec<'a, T>,
}

type Exec<'a, T> = fn(&'a WorkingSet) -> T;

/// Return values from Statement.
/// (Coulumns, Values)
type ResultSet = (Vec<usize>, Vec<usize>);

fn parse_string<T>(command: &str) -> Expr {
    //(|set| {})
    Expr::Literal(TokenLiteral::Str(command.to_string()))
}

#[cfg(test)]
mod tests {}

// TODO: Add it later: #[derive(Debug)]
pub enum Expr {
    Literal(TokenLiteral),
}
