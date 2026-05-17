use std::fmt::Debug;

use crate::intern::interner::Symbol;

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum TokenType {
    Comma,
    Dot,
    Semicolon,
    Colon,
    Eof,
    Identifier(Symbol),
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Illegal,

    // keywords
    Let,
    Function,
    If,
    Else,
    Return,

    // literals
    Boolean(bool),
    String(String),
    Int(i64),

    // whitespace is a generic term that represents ' ', or '\n', or '\r'
    Whitespace,

    // math
    PlusSign,
    MinusSign,
    MultiplicationSign,
    SlashSign,
    Asssign,
    // -> !
    BangSign,
    LT,
    GT,
    Lte,
    Gte,
    Eq,
    NotEq,
    And,
    Or,

    While,
}

impl TokenType {
    pub fn to_token_kind(&self) -> TokenKind {
        match self {
            TokenType::Comma => TokenKind::Comma,
            TokenType::Dot => TokenKind::Dot,
            TokenType::Semicolon => TokenKind::Semicolon,
            TokenType::Colon => TokenKind::Colon,
            TokenType::Eof => TokenKind::Eof,
            TokenType::Identifier(_) => TokenKind::Identifier,
            TokenType::LeftParen => TokenKind::LeftParen,
            TokenType::RightParen => TokenKind::RightParen,
            TokenType::LeftBrace => TokenKind::LeftBrace,
            TokenType::RightBrace => TokenKind::RightBrace,
            TokenType::LeftBracket => TokenKind::LeftBracket,
            TokenType::RightBracket => TokenKind::RightBracket,
            TokenType::Illegal => TokenKind::Illegal,

            // keywords
            TokenType::Let => TokenKind::Let,
            TokenType::Function => TokenKind::Function,
            TokenType::If => TokenKind::If,
            TokenType::Else => TokenKind::Else,
            TokenType::Return => TokenKind::Return,

            // literals
            TokenType::Boolean(_) => TokenKind::Boolean,
            TokenType::String(_) => TokenKind::String,
            TokenType::Int(_) => TokenKind::Int,

            // whitespace is a generic term that represents ' ', or '\n', or '\r'
            TokenType::Whitespace => TokenKind::Whitespace,

            // math
            TokenType::PlusSign => TokenKind::PlusSign,
            TokenType::MinusSign => TokenKind::MinusSign,
            TokenType::MultiplicationSign => TokenKind::MultiplicationSign,
            TokenType::SlashSign => TokenKind::SlashSign,
            TokenType::Asssign => TokenKind::Asssign,
            // -> !
            TokenType::BangSign => TokenKind::BangSign,
            TokenType::LT => TokenKind::LT,
            TokenType::GT => TokenKind::GT,
            TokenType::Lte => TokenKind::Lte,
            TokenType::Gte => TokenKind::Gte,
            TokenType::Eq => TokenKind::Eq,
            TokenType::NotEq => TokenKind::NotEq,
            TokenType::And => TokenKind::And,
            TokenType::Or => TokenKind::Or,
            TokenType::While => TokenKind::While,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub kind: TokenType,
}

impl Token {
    pub fn new(tt: TokenType) -> Self {
        Self { kind: tt }
    }

    pub fn new_let() -> Self {
        Self {
            kind: TokenType::Let,
        }
    }

    pub fn eof() -> Self {
        Self {
            kind: TokenType::Eof,
        }
    }

    pub fn whitespace() -> Self {
        Self {
            kind: TokenType::Whitespace,
        }
    }

    pub fn assign_sign() -> Self {
        Self {
            kind: TokenType::Asssign,
        }
    }

    pub fn semicolon() -> Self {
        Self {
            kind: TokenType::Semicolon,
        }
    }

    pub fn left_paren() -> Self {
        Self {
            kind: TokenType::LeftParen,
        }
    }

    pub fn right_paren() -> Self {
        Self {
            kind: TokenType::RightParen,
        }
    }

    pub fn left_brace() -> Self {
        Self {
            kind: TokenType::LeftBrace,
        }
    }

    pub fn right_brace() -> Self {
        Self {
            kind: TokenType::RightBrace,
        }
    }

    pub fn left_bracket() -> Self {
        Self {
            kind: TokenType::LeftBracket,
        }
    }

    pub fn right_bracket() -> Self {
        Self {
            kind: TokenType::RightBracket,
        }
    }

    pub fn function() -> Self {
        Self {
            kind: TokenType::Function,
        }
    }

    pub fn comma() -> Self {
        Self {
            kind: TokenType::Comma,
        }
    }

    pub fn bang() -> Self {
        Self {
            kind: TokenType::BangSign,
        }
    }

    pub fn lt() -> Self {
        Self {
            kind: TokenType::LT,
        }
    }

    pub fn gt() -> Self {
        Self {
            kind: TokenType::GT,
        }
    }

    pub fn lte() -> Self {
        Self {
            kind: TokenType::Lte,
        }
    }

    pub fn gte() -> Self {
        Self {
            kind: TokenType::Gte,
        }
    }

    pub fn int(n: i64) -> Self {
        Self {
            kind: TokenType::Int(n),
        }
    }

    pub fn identifier(symbol: Symbol) -> Self {
        Self {
            kind: TokenType::Identifier(symbol),
        }
    }

    pub fn string(string: String) -> Self {
        Self {
            kind: TokenType::String(string.clone()),
        }
    }

    pub fn boolean(b: bool) -> Self {
        Self {
            kind: TokenType::Boolean(b),
        }
    }

    pub fn colon() -> Self {
        Self {
            kind: TokenType::Colon,
        }
    }

    pub fn illegal() -> Self {
        Self {
            kind: TokenType::Illegal,
        }
    }

    pub fn and() -> Self {
        Self {
            kind: TokenType::And,
        }
    }

    pub fn or() -> Self {
        Self {
            kind: TokenType::Or,
        }
    }

    pub fn while_token() -> Self {
        Self {
            kind: TokenType::While,
        }
    }

    pub fn dot() -> Self {
        Self {
            kind: TokenType::Dot,
        }
    }
}

// Enum that mirrors TokenType but without the values.
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum TokenKind {
    Comma,
    Dot,
    Semicolon,
    Colon,
    Eof,
    Identifier,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Illegal,

    // keywords
    Let,
    Function,
    If,
    Else,
    Return,

    // literals
    Boolean,
    String,
    Int,

    // whitespace is a generic term that represents ' ', or '\n', or '\r'
    Whitespace,

    // math
    PlusSign,
    MinusSign,
    MultiplicationSign,
    SlashSign,
    Asssign,
    // -> !
    BangSign,
    LT,
    GT,
    Lte,
    Gte,
    Eq,
    NotEq,
    And,
    Or,

    While,
}
