use pom::parser::{is_a, none_of, one_of, seq, sym, Parser};
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    Minus,
    Plus,
    Slash,
    Star,
    Not,
    And,
    Or,
    NotEqual,
    Equal,
    GreaterThanOrEqual,
    LessThanOrEqual,
    GreaterThan,
    LessThan,
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Operator::Minus => write!(f, "-"),
            Operator::Plus => write!(f, "+"),
            Operator::Slash => write!(f, "/"),
            Operator::Star => write!(f, "*"),
            Operator::Not => write!(f, "!"),
            Operator::And => write!(f, "and"),
            Operator::Or => write!(f, "or"),
            Operator::NotEqual => write!(f, "!="),
            Operator::Equal => write!(f, "=="),
            Operator::GreaterThanOrEqual => write!(f, ">="),
            Operator::LessThanOrEqual => write!(f, "<="),
            Operator::GreaterThan => write!(f, ">"),
            Operator::LessThan => write!(f, "<"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Semicolon,
    Dot,

    O(Operator),

    Assign,

    // two character tokens.
    Comment,

    String(String),
    Float(f64),
    Int(i32),
    Bool(bool),
    Identifier(String),
    Nil(()),

    // Keywords.
    Class,
    Else,
    Fun,
    For,
    If,
    Print,
    Return,
    Super,
    This,
    Var,
    While,

    Eof,
    Illegal(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::LeftBrace => write!(f, "{{"),
            Token::RightBrace => write!(f, "}}"),
            Token::Comma => write!(f, ","),
            Token::Dot => write!(f, "."),
            Token::Semicolon => write!(f, ";"),

            Token::O(o) => write!(f, "{}", o),

            Token::Assign => write!(f, "="),
            Token::Comment => write!(f, "//"),

            Token::String(lit) => write!(f, "{}", lit),
            Token::Float(lit) => write!(f, "{}", lit),
            Token::Int(lit) => write!(f, "{}", lit),
            Token::Bool(lit) => write!(f, "{}", lit),
            Token::Identifier(lit) => write!(f, "{}", lit),

            Token::Nil(_) => write!(f, "nil"),

            Token::Class => write!(f, "class"),
            Token::Else => write!(f, "else"),
            Token::Fun => write!(f, "fun"),
            Token::For => write!(f, "for"),
            Token::If => write!(f, "if"),
            Token::Print => write!(f, "print"),
            Token::Return => write!(f, "return"),
            Token::Super => write!(f, "super"),
            Token::This => write!(f, "this"),
            Token::Var => write!(f, "var"),
            Token::While => write!(f, "while"),
            Token::Eof => write!(f, "EOF"),
            Token::Illegal(s) => write!(f, "illegal: {}", s),
        }
    }
}

fn whitespace<'a>() -> Parser<'a, char, ()> {
    one_of(" \t\r\n").repeat(1..).discard()
}

fn one_char<'a>() -> Parser<'a, char, Token> {
    one_of("(){},.;+-/*=!<>").map(|ch| match ch {
        '(' => Token::LeftParen,
        ')' => Token::RightParen,
        '{' => Token::LeftBrace,
        '}' => Token::RightBrace,
        ',' => Token::Comma,
        ';' => Token::Semicolon,
        '.' => Token::Dot,
        '+' => Token::O(Operator::Plus),
        '-' => Token::O(Operator::Minus),
        '/' => Token::O(Operator::Slash),
        '*' => Token::O(Operator::Star),
        '!' => Token::O(Operator::Not),
        '=' => Token::Assign,
        '<' => Token::O(Operator::LessThan),
        '>' => Token::O(Operator::GreaterThan),
        _ => Token::Illegal(ch.to_string()),
    })
}

fn two_char<'a>() -> Parser<'a, char, Token> {
    lazy_static! {
        static ref eqeq: Vec<char> = "==".chars().collect();
        static ref neq: Vec<char> = "!=".chars().collect();
        static ref leq: Vec<char> = "<=".chars().collect();
        static ref geq: Vec<char> = ">=".chars().collect();
        static ref comment: Vec<char> = "//".chars().collect();
    }
    seq(&eqeq).map(|_| Token::O(Operator::Equal))
        | seq(&neq).map(|_| Token::O(Operator::NotEqual))
        | seq(&leq).map(|_| Token::O(Operator::LessThanOrEqual))
        | seq(&geq).map(|_| Token::O(Operator::GreaterThanOrEqual))
        | seq(&comment).map(|_| Token::Comment)
}

fn alpha_num_literal<'a>() -> Parser<'a, char, Token> {
    -is_a(|ch: char| ch.is_alphabetic()) *
    is_a(|ch: char| ch.is_alphanumeric()).repeat(1..).map(|lit| {
        let lit_str: String = lit.into_iter().collect();
        match lit_str.as_str() {
            "and" => Token::O(Operator::And),
            "class" => Token::Class,
            "else" => Token::Else,
            "false" => Token::Bool(false),
            "true" => Token::Bool(true),
            "fun" => Token::Fun,
            "for" => Token::For,
            "if" => Token::If,
            "nil" => Token::Nil(()),
            "or" => Token::O(Operator::Or),
            "print" => Token::Print,
            "return" => Token::Return,
            "super" => Token::Super,
            "this" => Token::This,
            "var" => Token::Var,
            "while" => Token::While,
            lit_str if true => Token::Identifier(lit_str.parse().unwrap()),
            _ => Token::Illegal(lit_str),
        }
    })
}

fn int_literal<'a>() -> Parser<'a, char, Token> {
    is_a(|ch: char| ch.is_numeric())
        .repeat(1..)
        .map(|lit| Token::Float(lit.into_iter().collect::<String>().parse::<f64>().unwrap()))
}

fn float_literal<'a>() -> Parser<'a, char, Token> {
    (is_a(|ch: char| ch.is_numeric()).repeat(1..)
        + sym('.')
        + is_a(|ch: char| ch.is_numeric()).repeat(1..))
    .map(|((lit1, _), lit2)| {
        let lit1 = lit1.into_iter().collect::<String>();
        let lit2 = lit2.into_iter().collect::<String>();
        Token::Float(format!("{}.{}", lit1, lit2).parse::<f64>().unwrap())
    })
}

fn string<'a>() -> Parser<'a, char, Token> {
    (sym('"') * none_of("\"").repeat(0..) - sym('"'))
        .map(|s| Token::String(s.into_iter().collect()))
}

pub fn lexer<'a>() -> Parser<'a, char, Vec<Token>> {
    (whitespace().opt()
        * (alpha_num_literal() | float_literal() | int_literal() | two_char() | one_char() | string())
        - whitespace().opt())
    .repeat(0..)
}

// impl std::convert::From<String> for Token {
//     fn from(s: String) -> Self {
//         let input: Vec<char> = s.chars().collect();
//         lexer().parse(&input).unwrap().pop().unwrap()
//     }
// }
//
#[cfg(test)]
mod tests {
    use super::lexer;
    use super::Token;
    use crate::lexer::Operator;

    #[test]
    fn lex_single_char() {
        //a Vec<char> is the owned form of a &[char]
        let input: Vec<char> = "(} } . ; , + - /* ! = <>".chars().collect();
        let tokens = lexer().parse(&input);
        println!("{:?}", tokens);
        assert_eq!(
            tokens.unwrap(),
            vec![
                Token::LeftParen,
                Token::RightBrace,
                Token::RightBrace,
                Token::Dot,
                Token::Semicolon,
                Token::Comma,
                Token::O(Operator::Plus),
                Token::O(Operator::Minus),
                Token::O(Operator::Slash),
                Token::O(Operator::Star),
                Token::O(Operator::Not),
                Token::Assign,
                Token::O(Operator::LessThan),
                Token::O(Operator::GreaterThan),
            ]
        );
    }

    #[test]
    fn lex_double_char() {
        //a Vec<char> is the owned form of a &[char]
        let input: Vec<char> = "== != <= >= //".chars().collect();
        let tokens = lexer().parse(&input);
        println!("{:?}", tokens);
        assert_eq!(
            vec![
                Token::O(Operator::Equal),
                Token::O(Operator::NotEqual),
                Token::O(Operator::LessThanOrEqual),
                Token::O(Operator::GreaterThanOrEqual),
                Token::Comment,
            ],
            tokens.unwrap()
        );
    }

    #[test]
    fn lex_alpha_literal() {
        //a Vec<char> is the owned form of a &[char]
        let input: Vec<char> = "a b c and if else".chars().collect();
        let tokens = lexer().parse(&input);
        println!("{:?}", tokens);
        assert_eq!(
            vec![
                Token::Identifier(String::from("a")),
                Token::Identifier(String::from("b")),
                Token::Identifier(String::from("c")),
                Token::O(Operator::And),
                Token::If,
                Token::Else,
            ],
            tokens.unwrap()
        );
    }

    #[test]
    fn lex_string() {
        let input: Vec<char> = "\" a \"\"a\"".chars().collect();
        let tokens = lexer().parse(&input);
        println!("{:?}", tokens);
        assert_eq!(
            vec![
                Token::String(String::from(" a ")),
                Token::String(String::from("a")),
            ],
            tokens.unwrap()
        );
    }

    #[test]
    fn lex_float() {
        let input: Vec<char> = "3.33".chars().collect();
        let tokens = lexer().parse(&input);
        println!("{:?}", tokens);
        assert_eq!(tokens.unwrap(), vec![Token::Float(3.33)]);
    }

    #[test]
    fn lex_test() {
        let input: Vec<char> = "
            for (var a = 1; a < 10; a = a + 1) {
              print a;
            }

            var a = 1;
            while (a < 10) {
              print a;
              a = a + 1;
            }

            fun printSum(a, b) {
              print a + b;
            }

            fun addPair(a, b) {
              return a + b;
            }

            fun identity(a) {
              return a;
            }

            print identity(addPair)(1, 2); // Prints
        "
        .chars()
        .collect();
        let tokens = lexer().parse(&input);
        println!("{:#?}", tokens);
        // assert_eq!(tokens.unwrap(), vec![Token::Float(3.33)]);
    }

    #[test]
    fn enum_test() {
        println!("{}", Token::O(Operator::Minus));
        println!("{}", Token::RightBrace);
    }
}
