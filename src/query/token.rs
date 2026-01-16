use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TokenType {
    /// Keywords
    Get,
    Where,

    /// Operators
    Bang,
    Equals,
    At,

    /// Conditionals
    And,
    Or,

    /// Others
    Identifier,
    QuotedValue,

    /// Specials
    Eof,
}

impl FromStr for TokenType {
    type Err = String;

    fn from_str(input: &str) -> Result<TokenType, Self::Err> {
        match input.to_lowercase().as_str() {
            "get" => Ok(TokenType::Get),
            "where" => Ok(TokenType::Where),
            "=" => Ok(TokenType::Equals),
            "!" => Ok(TokenType::Bang),
            "@" => Ok(TokenType::At),
            "and" => Ok(TokenType::And),
            "or" => Ok(TokenType::Or),
            _ => Ok(TokenType::Identifier),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Token {
    pub(crate) t: TokenType,
    pub(crate) literal: TokenLiteral,
    pub(crate) position: usize,
    lexeme: String,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TokenLiteral {
    Str(String),
}

impl ToString for TokenLiteral {
    fn to_string(&self) -> String {
        match self {
            TokenLiteral::Str(s) => s.clone(),
        }
    }
}

impl Token {
    pub fn new(position: usize, t: TokenType, lexeme: String) -> Self {
        Token {
            position,
            t,
            literal: TokenLiteral::Str(lexeme),
            lexeme: "".to_string(),
        }
    }
}
