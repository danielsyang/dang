use std::collections::{HashMap, VecDeque};

use crate::{
    ast::statements::{
        identifier::Identifier, integer_literal::IntegerLiteral, return_statement::ReturnStatement,
    },
    lexer::{
        lexer::Lexer,
        token::{Token, TokenType},
    },
};

use super::{
    statements::{expression_statement::ExpressionStatement, let_statement::LetStatement},
    tree::{Expression, InfixParseFn, PrefixParseFn, Statement},
};

enum Precedence {
    Int = 0,
    Lowest = 1,
    Equals = 2,
    LessGreater = 3,
    Sum = 4,
    Product = 5,
    Prefix = 6,
    Call = 7,
}

pub struct Parser {
    pub tokens: VecDeque<Token>,
    errors: Vec<String>,
    current_token: Token,
    pub next_token: Token,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lex = Lexer::new(input);
        let mut tokens: VecDeque<Token> = VecDeque::new();

        while let Some(token) = lex.next_token() {
            match token.kind {
                TokenType::Whitespace => {}
                _ => tokens.push_back(token),
            }
        }

        let current_token = tokens
            .pop_front()
            .expect("Input did not produce any token.")
            .clone();
        let next_token = tokens.pop_front().expect("Expected at least EOF.").clone();

        return Self {
            tokens,
            errors: vec![],
            current_token,
            next_token,
        };
    }

    fn consume_token(&mut self) {
        println!(
            "moved current_token {:?} to {:?}",
            self.current_token.kind, self.next_token.kind
        );
        self.current_token = self.next_token.clone();
        self.next_token = self
            .tokens
            .pop_front()
            .expect("Invalid state, there are no more tokens to consume.");
        println!(
            "moved next_token {:?} to {:?}",
            self.current_token.kind, self.next_token.kind
        );
    }

    fn parse_program(&mut self) -> Box<dyn Statement> {
        match self.current_token.kind {
            TokenType::LET => Box::new(self.parse_let_statement()),
            TokenType::Return => Box::new(self.parse_return_statement()),
            _ => panic!("not yet implemented, got {:?}", self.current_token.kind),
        }
    }

    fn expect_next_token(&mut self, kind: TokenType) -> bool {
        if self.next_token.kind == kind {
            self.consume_token();
            return true;
        }
        return false;
    }

    fn parse_let_statement(&mut self) -> LetStatement {
        let let_token = self.current_token.clone();

        if !self.expect_next_token(TokenType::Identifier) {
            panic!(
                "Expected next token to be TokenType::Identifier, got: {:?}",
                self.next_token.kind
            )
        }

        let identifier = Identifier::new(&self.current_token);

        if !self.expect_next_token(TokenType::Asssign) {
            panic!(
                "Expected next token to be TokenType::Assign, got {:?}",
                self.next_token.kind
            )
        }

        self.consume_token();

        let val = self.parse_expression();

        if !self.expect_next_token(TokenType::Semicolon) {
            panic!(
                "Expected next token to be TokenType::Semicolon, got {:?}",
                self.next_token.kind
            )
        }

        return LetStatement::new(let_token, identifier, val);
    }

    fn parse_return_statement(&mut self) -> ReturnStatement {
        let return_token = self.current_token.clone();

        self.consume_token();

        let val = self.parse_expression();

        if self.next_token.kind == TokenType::Semicolon {
            self.consume_token();
        }

        return ReturnStatement::new(return_token, val);
    }

    fn parse_expression(&self) -> Box<dyn Expression> {
        match self.current_token.kind {
            TokenType::Int(v) => Box::new(IntegerLiteral::new(&self.current_token, v)),
            TokenType::Identifier => Box::new(Identifier::new(&self.current_token)),
            _ => panic!("not yet implemented, got {:?}", self.current_token.kind),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::{
            statements::{let_statement::LetStatement, return_statement::{self, ReturnStatement}},
            tree::{Node, Statement},
        },
        lexer::token::TokenType,
    };

    use super::Parser;

    #[test]
    fn parse_let_statement() {
        let input = "
        let x = 5;
        let y = 100;
        let foobar = y;
        ";
        let let_name = ["x", "y", "foobar"];
        let let_val = ["5", "100", "y"];
        let mut p = Parser::new(input);

        let mut result: Vec<Box<dyn Statement>> = vec![];
        loop {
            let parsed = p.parse_program();
            result.push(parsed);

            if p.next_token.kind == TokenType::EOF {
                break;
            }
            p.consume_token();
        }

        for (i, curr) in result.iter().enumerate() {
            let l = curr.as_any().downcast_ref::<LetStatement>().unwrap();
            assert_eq!(l.token.kind, TokenType::LET);
            assert_eq!(l.name.token_literal(), let_name.get(i).unwrap().to_string());
            assert_eq!(l.value.token_literal(), let_val.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn parse_return_statement() {
        let input = "
        return 5;
        return 100;
        return foobar;
        ";
        let mut p = Parser::new(input);
        let let_val = ["5", "100", "foobar"];
        let mut result: Vec<Box<dyn Statement>> = vec![];
        loop {
            let parsed = p.parse_program();
            result.push(parsed);

            if p.next_token.kind == TokenType::EOF {
                break;
            }
            p.consume_token();
        }

        for (i, curr) in result.iter().enumerate() {
            let l = curr.as_any().downcast_ref::<ReturnStatement>().unwrap();
            assert_eq!(l.token.kind, TokenType::Return);
            assert_eq!(l.value.token_literal(), let_val.get(i).unwrap().to_string());
        }
    }
}
