/// Scans the input string and produces tokens.
/// TODO: Rename it to tokenizer.rs
use std::str::FromStr;

use crate::query::token::{Token, TokenType};

#[derive(Debug, PartialEq)]
enum State {
    StartCmd,
    InTerm,
    InQuoted,
    EndQuoted,
    EndTerm,
    EndCmd,
}

pub(crate) struct Scanner<'a> {
    base: &'a [u8],

    start: usize,
    len: usize,
    position: usize,

    state: State,
}

enum Statement {}

impl<'a> Scanner<'a> {
    pub fn new(input_string: &'a str) -> Self {
        let len = input_string.len();

        Scanner {
            base: input_string.as_bytes(),
            position: 0,
            start: 0,
            len,
            state: State::StartCmd,
        }
    }

    pub(crate) fn next_token(&mut self) -> Token {
        if self.position >= self.len {
            return Token::new(self.position, TokenType::Eof, "".to_string());
        }

        self.start = self.position;
        let mut value = vec![];
        loop {
            match (self.peek(), &self.state) {
                (Some(b), State::StartCmd) if b.is_ascii_whitespace() => {
                    self.advance();
                }
                (Some(b), State::StartCmd) if matches!(b, b'!' | b'=' | b'@') => {
                    value.push(b);
                    self.advance();
                    self.state = State::EndTerm
                }
                (Some(b), State::StartCmd) if b == b"\""[0] => {
                    self.state = State::InQuoted;
                    self.start = self.position;
                    self.advance();
                }
                (Some(b), State::InTerm) if b.is_ascii_whitespace() => {
                    self.state = State::EndTerm;
                }
                (Some(_), State::StartCmd) => {
                    self.state = State::InTerm;
                    self.start = self.position;
                }
                (Some(b), State::InTerm) => {
                    value.push(b);
                    self.advance();
                }
                (Some(b), State::InQuoted) if b == b"\""[0] => {
                    self.state = State::EndQuoted;
                }
                (Some(b), State::InQuoted) => {
                    value.push(b);
                    self.advance();
                }
                (None, State::InQuoted) => {
                    // TODO: Return error
                    panic!("Expected '\"'. Position: {}", self.position);
                }
                (Some(_), State::EndQuoted) => {
                    self.advance();
                    self.state = State::StartCmd;

                    let lexeme = str::from_utf8(&value).expect("Invalid UTF-8");
                    return Token::new(self.start, TokenType::QuotedValue, lexeme.to_string());
                }
                (None, State::InTerm) => {
                    self.state = State::EndTerm;
                }
                (_, State::EndTerm) => {
                    self.state = State::StartCmd;
                    break;
                }
                (_, State::EndCmd) => {
                    return Token::new(self.start, TokenType::Eof, "".to_string());
                }
                (None, _) => self.state = State::EndCmd,
            }
        }

        let lexeme = str::from_utf8(&value).expect("Invalid UTF-8");
        let token_type = TokenType::from_str(lexeme)
            .expect("Critically bad token value. Should not ever happen.");
        Token::new(self.start, token_type, lexeme.to_string())
    }

    // TODO: Make it Option maybe?
    #[inline]
    fn advance(&mut self) {
        self.position += 1;
    }

    #[inline]
    fn peek(&mut self) -> Option<u8> {
        if self.position >= self.len {
            return None;
        }

        let next = self.base[self.position];
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use crate::query::{
        scanner::Scanner,
        token::{TokenLiteral, TokenType},
    };

    #[test]
    fn test_simple_command() {
        let input = "get id";

        let mut scanner = Scanner::new(input);

        let t = scanner.next_token();
        assert_eq!(t.literal, TokenLiteral::Str("get".to_string()));
        assert_eq!(t.t, TokenType::Get);

        let t = scanner.next_token();
        assert_eq!(t.literal, TokenLiteral::Str("id".to_string()));
        assert_eq!(t.t, TokenType::Identifier);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::Eof);
    }

    #[test]
    fn test_position_tracking() {
        let input = "get id";
        let mut scanner = Scanner::new(input);

        let t1 = scanner.next_token();
        assert_eq!(t1.position, 0);

        let t2 = scanner.next_token();
        assert_eq!(t2.position, 4);
    }

    #[test]
    fn test_eof_token() {
        let input = "get";
        let mut scanner = Scanner::new(input);

        scanner.next_token();
        let eof = scanner.next_token();

        assert_eq!(eof.t, TokenType::Eof);
    }

    #[test]
    fn test_multiple_whitespaces() {
        let input = "get    id";
        let mut scanner = Scanner::new(input);

        let t1 = scanner.next_token();
        assert_eq!(t1.literal, TokenLiteral::Str("get".to_string()));

        let t2 = scanner.next_token();
        assert_eq!(t2.literal, TokenLiteral::Str("id".to_string()));
    }

    #[test]
    fn test_simple_quoted_value() {
        let input = r#""hello""#;
        let mut scanner = Scanner::new(input);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::QuotedValue);
        assert_eq!(t.literal, TokenLiteral::Str("hello".to_string()));
    }

    #[test]
    fn test_quoted_value_with_spaces() {
        let input = r#""hello world""#;
        let mut scanner = Scanner::new(input);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::QuotedValue);
        assert_eq!(t.literal, TokenLiteral::Str("hello world".to_string()));
    }

    #[test]
    fn test_get_with_quoted_column() {
        let input = r#"get "column name""#;
        let mut scanner = Scanner::new(input);

        let t1 = scanner.next_token();
        assert_eq!(t1.t, TokenType::Get);
        assert_eq!(t1.literal, TokenLiteral::Str("get".to_string()));

        let t2 = scanner.next_token();
        assert_eq!(t2.t, TokenType::QuotedValue);
        assert_eq!(t2.literal, TokenLiteral::Str("column name".to_string()));

        let t3 = scanner.next_token();
        assert_eq!(t3.t, TokenType::Eof);
    }

    #[test]
    fn test_multiple_quoted_values() {
        let input = r#""first" "second" "third""#;
        let mut scanner = Scanner::new(input);

        let t1 = scanner.next_token();
        assert_eq!(t1.t, TokenType::QuotedValue);
        assert_eq!(t1.literal, TokenLiteral::Str("first".to_string()));

        let t2 = scanner.next_token();
        assert_eq!(t2.t, TokenType::QuotedValue);
        assert_eq!(t2.literal, TokenLiteral::Str("second".to_string()));

        let t3 = scanner.next_token();
        assert_eq!(t3.t, TokenType::QuotedValue);
        assert_eq!(t3.literal, TokenLiteral::Str("third".to_string()));

        let eof = scanner.next_token();
        assert_eq!(eof.t, TokenType::Eof);
    }

    #[test]
    fn test_empty_quoted_value() {
        let input = r#""""#;
        let mut scanner = Scanner::new(input);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::QuotedValue);
        assert_eq!(t.literal, TokenLiteral::Str("".to_string()));
    }

    #[test]
    fn test_quoted_value_with_special_characters() {
        let input = r#""name@email.com""#;
        let mut scanner = Scanner::new(input);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::QuotedValue);
        assert_eq!(t.literal, TokenLiteral::Str("name@email.com".to_string()));
    }

    #[test]
    fn test_mixed_quoted_and_unquoted() {
        let input = r#"get "user name" age"#;
        let mut scanner = Scanner::new(input);

        let t1 = scanner.next_token();
        assert_eq!(t1.t, TokenType::Get);

        let t2 = scanner.next_token();
        assert_eq!(t2.t, TokenType::QuotedValue);
        assert_eq!(t2.literal, TokenLiteral::Str("user name".to_string()));

        let t3 = scanner.next_token();
        assert_eq!(t3.t, TokenType::Identifier);
        assert_eq!(t3.literal, TokenLiteral::Str("age".to_string()));
    }

    #[test]
    fn test_quoted_value_position_tracking() {
        let input = r#"get "column""#;
        let mut scanner = Scanner::new(input);

        let t1 = scanner.next_token();
        assert_eq!(t1.position, 0);

        let t2 = scanner.next_token();
        assert_eq!(t2.position, 4);
    }

    #[test]
    fn test_quoted_value_with_leading_whitespace() {
        let input = r#"   "hello""#;
        let mut scanner = Scanner::new(input);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::QuotedValue);
        assert_eq!(t.literal, TokenLiteral::Str("hello".to_string()));
    }

    #[test]
    fn test_quoted_value_with_trailing_whitespace() {
        let input = r#""hello"   "#;
        let mut scanner = Scanner::new(input);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::QuotedValue);
        assert_eq!(t.literal, TokenLiteral::Str("hello".to_string()));

        let eof = scanner.next_token();
        assert_eq!(eof.t, TokenType::Eof);
    }

    #[test]
    fn test_quoted_value_with_tabs() {
        let input = "get\t\"column\"\tid";
        let mut scanner = Scanner::new(input);

        let t1 = scanner.next_token();
        assert_eq!(t1.t, TokenType::Get);

        let t2 = scanner.next_token();
        assert_eq!(t2.t, TokenType::QuotedValue);
        assert_eq!(t2.literal, TokenLiteral::Str("column".to_string()));

        let t3 = scanner.next_token();
        assert_eq!(t3.t, TokenType::Identifier);
    }

    #[test]
    fn test_quoted_value_with_newlines_outside() {
        let input = "get\n\"column\"\nid";
        let mut scanner = Scanner::new(input);

        let t1 = scanner.next_token();
        assert_eq!(t1.t, TokenType::Get);

        let t2 = scanner.next_token();
        assert_eq!(t2.t, TokenType::QuotedValue);

        let t3 = scanner.next_token();
        assert_eq!(t3.t, TokenType::Identifier);
    }

    #[test]
    fn test_quoted_value_with_numbers() {
        let input = r#""12345""#;
        let mut scanner = Scanner::new(input);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::QuotedValue);
        assert_eq!(t.literal, TokenLiteral::Str("12345".to_string()));
    }

    #[test]
    fn test_quoted_value_with_mixed_content() {
        let input = r#""abc123xyz""#;
        let mut scanner = Scanner::new(input);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::QuotedValue);
        assert_eq!(t.literal, TokenLiteral::Str("abc123xyz".to_string()));
    }

    #[test]
    fn test_quoted_value_with_punctuation() {
        let input = r#""Hello, World!""#;
        let mut scanner = Scanner::new(input);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::QuotedValue);
        assert_eq!(t.literal, TokenLiteral::Str("Hello, World!".to_string()));
    }

    #[test]
    fn test_quoted_value_with_symbols() {
        let input = r#""$price >= 100""#;
        let mut scanner = Scanner::new(input);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::QuotedValue);
        assert_eq!(t.literal, TokenLiteral::Str("$price >= 100".to_string()));
    }

    #[test]
    fn test_quoted_value_with_tabs_inside() {
        let input = "\"hello\tworld\"";
        let mut scanner = Scanner::new(input);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::QuotedValue);
        assert_eq!(t.literal, TokenLiteral::Str("hello\tworld".to_string()));
    }

    #[test]
    fn test_quoted_value_with_newlines_inside() {
        let input = "\"hello\nworld\"";
        let mut scanner = Scanner::new(input);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::QuotedValue);
        assert_eq!(t.literal, TokenLiteral::Str("hello\nworld".to_string()));
    }

    #[test]
    fn test_long_quoted_value() {
        let long_string = "a".repeat(1000);
        let input = format!(r#""{}""#, long_string);
        let mut scanner = Scanner::new(&input);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::QuotedValue);
        assert_eq!(t.literal, TokenLiteral::Str(long_string));
    }

    #[test]
    fn test_quoted_value_with_unicode() {
        let input = r#""Hello 世界 🌍""#;
        let mut scanner = Scanner::new(input);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::QuotedValue);
        assert_eq!(t.literal, TokenLiteral::Str("Hello 世界 🌍".to_string()));
    }

    #[test]
    fn test_only_quoted_value() {
        let input = r#""only""#;
        let mut scanner = Scanner::new(input);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::QuotedValue);

        let eof = scanner.next_token();
        assert_eq!(eof.t, TokenType::Eof);
    }

    #[test]
    fn test_quoted_value_at_end() {
        let input = r#"get id "last column""#;
        let mut scanner = Scanner::new(input);

        scanner.next_token();
        scanner.next_token();

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::QuotedValue);
        assert_eq!(t.literal, TokenLiteral::Str("last column".to_string()));

        let eof = scanner.next_token();
        assert_eq!(eof.t, TokenType::Eof);
    }

    #[test]
    fn test_multiple_spaces_between_quoted_values() {
        let input = r#""first"     "second""#;
        let mut scanner = Scanner::new(input);

        let t1 = scanner.next_token();
        assert_eq!(t1.t, TokenType::QuotedValue);
        assert_eq!(t1.literal, TokenLiteral::Str("first".to_string()));

        let t2 = scanner.next_token();
        assert_eq!(t2.t, TokenType::QuotedValue);
        assert_eq!(t2.literal, TokenLiteral::Str("second".to_string()));
    }

    #[test]
    fn test_single_character_quoted() {
        let input = r#""a""#;
        let mut scanner = Scanner::new(input);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::QuotedValue);
        assert_eq!(t.literal, TokenLiteral::Str("a".to_string()));
    }

    #[test]
    fn test_quoted_space() {
        let input = r#"" ""#;
        let mut scanner = Scanner::new(input);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::QuotedValue);
        assert_eq!(t.literal, TokenLiteral::Str(" ".to_string()));
    }

    #[test]
    fn test_quoted_multiple_spaces() {
        let input = r#""     ""#;
        let mut scanner = Scanner::new(input);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::QuotedValue);
        assert_eq!(t.literal, TokenLiteral::Str("     ".to_string()));
    }

    #[test]
    fn test_case_sensitivity_in_keywords() {
        let input = "GET get Get";
        let mut scanner = Scanner::new(input);

        let t1 = scanner.next_token();
        assert_eq!(t1.t, TokenType::Get);

        let t2 = scanner.next_token();
        assert_eq!(t2.t, TokenType::Get);

        let t3 = scanner.next_token();
        assert_eq!(t3.t, TokenType::Get);
    }

    #[test]
    fn test_consecutive_eof_calls() {
        let input = "get";
        let mut scanner = Scanner::new(input);

        scanner.next_token();

        let eof1 = scanner.next_token();
        assert_eq!(eof1.t, TokenType::Eof);

        let eof2 = scanner.next_token();
        assert_eq!(eof2.t, TokenType::Eof);

        let eof3 = scanner.next_token();
        assert_eq!(eof3.t, TokenType::Eof);
    }

    #[test]
    fn test_identifier_with_numbers() {
        let input = "column123";
        let mut scanner = Scanner::new(input);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::Identifier);
        assert_eq!(t.literal, TokenLiteral::Str("column123".to_string()));
    }

    #[test]
    fn test_identifier_with_underscore() {
        let input = "user_name";
        let mut scanner = Scanner::new(input);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::Identifier);
        assert_eq!(t.literal, TokenLiteral::Str("user_name".to_string()));
    }

    #[test]
    fn test_where_equals() {
        let input = "get where name = \"test\"";
        let mut scanner = Scanner::new(input);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::Get);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::Where);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::Identifier);
        assert_eq!(t.literal, TokenLiteral::Str("name".to_string()));

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::Equals);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::QuotedValue);
        assert_eq!(t.literal, TokenLiteral::Str("test".to_string()));

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::Eof);
    }

    #[test]
    fn test_where_bang_equals() {
        let input = "get where name != \"test\"";
        let mut scanner = Scanner::new(input);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::Get);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::Where);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::Identifier);
        assert_eq!(t.literal, TokenLiteral::Str("name".to_string()));

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::Bang);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::Equals);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::QuotedValue);
        assert_eq!(t.literal, TokenLiteral::Str("test".to_string()));

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::Eof);
    }

    #[test]
    fn test_get_at() {
        let input = "get name @ users";
        let mut scanner = Scanner::new(input);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::Get);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::Identifier);
        assert_eq!(t.literal, TokenLiteral::Str("name".to_string()));

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::At);

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::Identifier);
        assert_eq!(t.literal, TokenLiteral::Str("users".to_string()));

        let t = scanner.next_token();
        assert_eq!(t.t, TokenType::Eof);
    }
}
