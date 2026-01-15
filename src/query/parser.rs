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

    // TODO: Make it Result<>
    pub(crate) fn parse_string(mut self, command: &str) -> Statement {
        //(|set| {})
        let mut scanner = Scanner::new(command);

        let mut token = scanner.next_token();
        while token.t != TokenType::Eof {
            self.tokens.push(Rc::new(token));
            token = scanner.next_token();
        }
        dbg!(&self.tokens);

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
        let expr = self.multiple();

        //if self.current().t == Eof {
        Statement::Get(expr, None, None)
        //}
    }

    fn multiple(&mut self) -> Expr {
        let mut left = self.term();

        while !self.finised()
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

    fn finised(&self) -> bool {
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
}

// TODO: Add it later: #[derive(Debug)]
#[derive(Debug)]
pub(crate) enum Expr {
    Literal(Rc<Token>),
    Term(Box<Expr>),
    Multiple(Box<Expr>, Box<Expr>),
    Conditional(Box<Expr>, Rc<Token>, Rc<Expr>),
}

#[derive(Debug)]
pub(crate) enum Statement {
    /// "get" token ("," + token)* "@" token "where" conditional_expr
    Get(Expr, Option<Token>, Option<Expr>),
}

impl Statement {
    pub fn accept<R: Sized>(&self, visitor: &dyn Visitor<R>) -> R {
        visitor.visit(self)
    }
}

pub trait Visitor<R: Sized> {
    fn visit(&self, expr: &Statement) -> R;
}

// TODO: add proper tests
#[cfg(test)]
mod tests {
    use crate::parser::CmdParser;

    //#[test]
    fn test_simple_query() {
        let p = CmdParser::new();

        let statement = p.parse_string("get name age city");

        dbg!(statement);
    }
}
