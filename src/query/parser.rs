use core::panic;
use std::rc::Rc;

use crate::{
    query::scanner::Scanner,
    query::token::{Token, TokenType},
};

/// Return values from Statement.
/// (Coulumns, Values)
type ResultSet = (Vec<usize>, Vec<usize>);

pub(crate) struct CmdParser {
    tokens: Vec<Rc<Token>>,
    current: usize,
}

impl CmdParser {
    pub(crate) fn new() -> CmdParser {
        Self {
            tokens: vec![],
            current: 0,
        }
    }

    fn peek_expect(&self, token_type: TokenType) -> bool {
        !self.finished() && self.current().t == token_type
    }

    // TODO: Make it Result<>
    pub(crate) fn parse_string(mut self, command: &str) -> Statement {
        //(|set| {})
        let mut scanner = Scanner::new(command);

        let mut token = scanner.next_token();
        while token.t != TokenType::Eof {
            self.tokens.push(Rc::new(token));
            token = scanner.next_token();
        }

        self.statement()
    }

    fn statement(&mut self) -> Statement {
        let token = self.consume();
        match token.t {
            TokenType::Get => self.get_statement(),
            _ => panic!("Parser Error: Expected unexpected operation"),
        }
    }

    fn get_statement(&mut self) -> Statement {
        let columns = self.multiple();
        let mut tables = None;
        let mut condition = None;

        if self.peek_expect(TokenType::At) {
            self.consume();
            tables = Some(self.term());
        }

        if self.peek_expect(TokenType::Where) {
            self.consume();
            condition = Some(self.conditional());
        }

        Statement::Get(columns, tables, condition)
    }

    fn conditional(&mut self) -> Expr {
        let mut left = self.comparison();

        while !self.finished() && matches!(self.current().t, TokenType::And | TokenType::Or) {
            let operator = self.consume();
            let right = self.comparison();
            left = Expr::Conditional(Box::new(left), operator, Box::new(right));
        }

        left
    }

    fn comparison(&mut self) -> Expr {
        let left = self.term();
        let operator = self.consume_if(|t| matches!(t, TokenType::Equals));
        let right = self.term();

        Expr::Conditional(Box::new(left), operator, Box::new(right))
    }

    fn multiple(&mut self) -> Expr {
        let mut left = self.term();

        while !self.finished()
            && matches!(
                self.current().t,
                TokenType::Identifier | TokenType::QuotedValue
            )
        {
            let right = self.term();
            left = Expr::Multiple(Box::new(left), Box::new(right));
        }

        left
    }

    fn term(&mut self) -> Expr {
        let current = self.consume();

        match current.t {
            TokenType::Identifier | TokenType::QuotedValue => Expr::Literal(current),
            _ => panic!("Parser Error: Expected identifier, got {:?}", current.t),
        }
    }

    fn finished(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn current(&self) -> &Token {
        if self.current >= self.tokens.len() {
            panic!("Parser Error: Reached end of tokens");
        }

        self.tokens
            .get(self.current)
            .expect("Something really bad happened")
    }

    fn consume(&mut self) -> Rc<Token> {
        if self.current >= self.tokens.len() {
            panic!("Parser Error: Reached end of tokens");
        }

        let token = self
            .tokens
            .get(self.current)
            .expect("Something really bad happened");
        self.current += 1;
        token.clone()
    }

    fn consume_expect(&mut self, expected: TokenType) -> Rc<Token> {
        let token = self.consume();
        if token.t != expected {
            panic!(
                "Parser Error: Expected token {:?}, found {:?}",
                expected, token.t
            );
        }
        token.clone()
    }

    fn consume_if<F>(&mut self, predicate: F) -> Rc<Token>
    where
        F: Fn(&TokenType) -> bool,
    {
        let token = self.consume();
        if !predicate(&token.t) {
            panic!("Parser Error: Unexpected token {:?}", token.t);
        }
        token
    }
}

// TODO: Add it later: #[derive(Debug)]
#[derive(Debug)]
pub(crate) enum Expr {
    Literal(Rc<Token>),
    Multiple(Box<Expr>, Box<Expr>),
    Conditional(Box<Expr>, Rc<Token>, Box<Expr>),
}

#[derive(Debug)]
pub(crate) enum Statement {
    /// "get" token ("," + token)* "@" token "where" conditional_expr
    Get(Expr, Option<Expr>, Option<Expr>),
}

impl Statement {
    pub fn accept<R: Sized>(&self, visitor: &dyn Visitor<R>) -> R {
        visitor.visit(self)
    }
}

pub(crate) trait Visitor<R: Sized> {
    fn visit(&self, expr: &Statement) -> R;
}

#[cfg(test)]
mod tests {
    use crate::query::parser::{CmdParser, Expr, Statement};

    fn extract_columns(expr: &Expr) -> Vec<String> {
        match expr {
            Expr::Literal(token) => vec![token.literal.to_string()],
            Expr::Multiple(left, right) => {
                let mut cols = extract_columns(left);
                cols.extend(extract_columns(right));
                cols
            }
            _ => vec![],
        }
    }

    #[test]
    fn test_get_single_column() {
        let p = CmdParser::new();
        let statement = p.parse_string("get name");

        match statement {
            Statement::Get(expr, None, None) => {
                let cols = extract_columns(&expr);
                assert_eq!(cols, vec!["name"]);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_get_multiple_columns() {
        let p = CmdParser::new();
        let statement = p.parse_string("get name age city");

        match statement {
            Statement::Get(expr, None, None) => {
                let cols = extract_columns(&expr);
                assert_eq!(cols, vec!["name", "age", "city"]);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_ast_structure_two_columns() {
        let p = CmdParser::new();
        let statement = p.parse_string("get a b");

        match statement {
            Statement::Get(Expr::Multiple(left, right), None, None) => {
                assert!(matches!(left.as_ref(), Expr::Literal(_)));
                assert!(matches!(right.as_ref(), Expr::Literal(_)));
            }
            _ => unreachable!(),
        }
    }

    fn extract_table(expr: &Expr) -> String {
        match expr {
            Expr::Literal(token) => token.literal.to_string(),
            _ => panic!("Expected literal for table name"),
        }
    }

    #[test]
    fn test_get_with_table_identifier() {
        let p = CmdParser::new();
        let statement = p.parse_string("get name @ users");

        dbg!(&statement);
        match statement {
            Statement::Get(cols, Some(table), None) => {
                assert_eq!(extract_columns(&cols), vec!["name"]);
                assert_eq!(extract_table(&table), "users");
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_get_with_table_quoted() {
        let p = CmdParser::new();
        let statement = p.parse_string("get name @ \"users\"");

        match statement {
            Statement::Get(cols, Some(table), None) => {
                assert_eq!(extract_columns(&cols), vec!["name"]);
                assert_eq!(extract_table(&table), "users");
            }
            _ => unreachable!(),
        }
    }

    #[test]
    #[should_panic(expected = "Parser Error")]
    fn test_empty_get_panics() {
        let p = CmdParser::new();
        p.parse_string("get");
    }

    #[test]
    #[should_panic(expected = "Parser Error")]
    fn test_invalid_statement_panics() {
        let p = CmdParser::new();
        p.parse_string("select name");
    }

    #[test]
    fn test_get_with_where_condition() {
        let p = CmdParser::new();
        let statement = p.parse_string(r#"get name @ users where first = "john""#);

        match statement {
            Statement::Get(cols, Some(table), Some(condition)) => {
                assert_eq!(extract_columns(&cols), vec!["name"]);
                assert_eq!(extract_table(&table), "users");

                match condition {
                    Expr::Conditional(left, op, right) => {
                        assert_eq!(extract_table(&left), "first");
                        assert_eq!(op.literal.to_string(), "=");
                        assert_eq!(extract_table(&right), "john");
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }
}
