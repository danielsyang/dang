use std::collections::BTreeMap;

use crate::{
    eval::program::Program,
    intern::interner::Interner,
    lex::{
        lexer::Lexer,
        token::{Token, TokenKind, TokenType},
    },
};

use super::{
    expression::{Expression, Operator, Prefix},
    literal::Literal,
    statement::{Block, Identifier, Statement},
};

#[derive(Clone, Copy, Debug)]
enum Precedence {
    _Int = 0,
    Lowest = 1,
    Dot = 2,
    Equals = 3,
    LessGreaterOrEqual = 4,
    AndOr = 5,
    Sum = 6,
    Product = 7,
    Exponent = 8,
    Prefix = 9,
    Call = 10,
    Index = 11,
}

pub struct Parser<'a> {
    lex: Lexer,
    current_token: Token,
    next_token: Token,
    interner: &'a mut Interner,
}

impl<'a> Parser<'a> {
    fn analyze_next_token(lex: &mut Lexer, interner: &mut Interner) -> Token {
        while let Some(t) = lex.next_token(interner) {
            if t.kind != TokenType::Whitespace {
                return t;
            }
        }

        Token::illegal()
    }

    fn consume_token(&mut self) {
        self.current_token = self.next_token.clone();
        self.next_token = Parser::analyze_next_token(&mut self.lex, self.interner);
    }

    fn expect_next_token(&mut self, kind: TokenKind) -> bool {
        if self.next_token.kind.to_token_kind() == kind {
            self.consume_token();
            return true;
        }
        false
    }

    pub fn build_ast(input: &str, interner: &mut Interner) -> Program {
        let mut lex = Lexer::new(input);
        let mut result: Vec<Statement> = vec![];

        let first = Parser::analyze_next_token(&mut lex, interner);
        let second = Parser::analyze_next_token(&mut lex, interner);

        let mut parser = Parser {
            lex,
            current_token: first,
            next_token: second,
            interner,
        };

        loop {
            let parsed = parser.parse_statement();

            match parsed {
                Statement::Error(_) => break,
                _ => result.push(parsed),
            }

            if parser.next_token.kind == TokenType::Eof {
                break;
            }
            parser.consume_token();
        }

        Program { statements: result }
    }

    fn parse_statement(&mut self) -> Statement {
        match (&self.current_token.kind, &self.next_token.kind) {
            (TokenType::Let, _) => self.parse_let_statement(),
            (TokenType::Return, _) => self.parse_return_statement(),
            (TokenType::Identifier(_), TokenType::Asssign) => self.parse_assignment_statement(),
            (TokenType::While, _) => self.parse_while_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Statement {
        if !self.expect_next_token(TokenKind::Identifier) {
            return Statement::Error(format!(
                "Expected next token to be TokenType::Identifier, got: {:?}",
                self.next_token.kind
            ));
        }

        let identifier = match self.current_token.kind {
            TokenType::Identifier(val) => val,
            _ => unreachable!("parse_let_statement failed to get TokenType::Identifier name"),
        };

        if !self.expect_next_token(TokenKind::Asssign) {
            return Statement::Error(format!(
                "Expected next token to be TokenType::Assign, got {:?}",
                self.next_token.kind
            ));
        }

        self.consume_token();

        let val = self.parse_expression(Precedence::Lowest);

        if !self.expect_next_token(TokenKind::Semicolon) {
            return Statement::Error(format!(
                "Expected next token to be TokenType::Semicolon, got {:?}",
                self.next_token.kind
            ));
        }

        Statement::Let(identifier, val)
    }

    fn parse_return_statement(&mut self) -> Statement {
        self.consume_token();

        let return_val = self.parse_expression(Precedence::Lowest);

        if self.next_token.kind == TokenType::Semicolon {
            self.consume_token();
        }

        Statement::Return(return_val)
    }

    fn parse_assignment_statement(&mut self) -> Statement {
        let identifier = match self.current_token.kind {
            TokenType::Identifier(val) => val,
            _ => {
                unreachable!("parse_assignment_statament failed to get TokenType::Identifier name")
            }
        };

        self.consume_token();
        self.consume_token();

        let exp = self.parse_expression(Precedence::Lowest);

        if self.next_token.kind == TokenType::Semicolon {
            self.consume_token();
        }

        Statement::Assignment(identifier, exp)
    }

    fn parse_while_statement(&mut self) -> Statement {
        if !self.expect_next_token(TokenKind::LeftParen) {
            return Statement::Error(format!(
                "Expected next token to be TokenType::LeftParen, got: {:?}",
                self.next_token.kind
            ));
        }

        let condition = self.parse_expression(Precedence::Lowest);

        if !self.expect_next_token(TokenKind::LeftBrace) {
            return Statement::Error(format!(
                "Expected next token to be TokenType::LeftBrace, got {:?}",
                self.next_token.kind
            ));
        }

        let body = self.parse_block_statement();

        Statement::While { condition, body }
    }

    fn parse_expression(&mut self, p: Precedence) -> Expression {
        let mut left_exp = match &self.current_token.kind {
            TokenType::Int(v) => Expression::Literal(Literal::Number(*v)),
            TokenType::Identifier(val) => Expression::Identifier(*val),
            TokenType::String(s) => Expression::Literal(Literal::String(s.clone())),
            TokenType::Boolean(b) => Expression::Literal(Literal::Boolean(*b)),
            TokenType::BangSign => self.parse_prefix_expression(Prefix::Bang),
            TokenType::MinusSign => self.parse_prefix_expression(Prefix::Minus),
            TokenType::LeftParen => self.parse_grouped_expression(),
            TokenType::If => self.parse_if_expression(),
            TokenType::Function => self.parse_function_expression(),
            TokenType::LeftBracket => {
                Expression::Array(self.parse_elements_list(TokenType::RightBracket))
            }
            TokenType::LeftBrace => self.parse_hashmaps_literal(),
            _ => {
                return Expression::Error(format!(
                    "parse_expression: not yet implemented, got {:?}",
                    self.current_token.kind
                ));
            }
        };

        while (p as u8) < self.next_precedence() && self.next_token.kind != TokenType::Semicolon {
            left_exp = match self.next_token.kind {
                TokenType::PlusSign => self.parse_infix_expression(left_exp, Operator::Plus),
                TokenType::MinusSign => self.parse_infix_expression(left_exp, Operator::Minus),
                TokenType::MultiplicationSign => {
                    self.parse_infix_expression(left_exp, Operator::Multiply)
                }
                TokenType::SlashSign => self.parse_infix_expression(left_exp, Operator::Divide),
                TokenType::Eq => self.parse_infix_expression(left_exp, Operator::Equal),
                TokenType::NotEq => self.parse_infix_expression(left_exp, Operator::NotEqual),
                TokenType::LT => self.parse_infix_expression(left_exp, Operator::LessThan),
                TokenType::GT => self.parse_infix_expression(left_exp, Operator::GreaterThan),
                TokenType::Lte => self.parse_infix_expression(left_exp, Operator::LessThanOrEqual),
                TokenType::Gte => {
                    self.parse_infix_expression(left_exp, Operator::GreaterThanOrEqual)
                }
                TokenType::And => self.parse_infix_expression(left_exp, Operator::And),
                TokenType::Or => self.parse_infix_expression(left_exp, Operator::Or),
                TokenType::ExponentSign => {
                    self.parse_infix_expression(left_exp, Operator::Exponent)
                }
                TokenType::LeftParen => self.parse_call_expression(left_exp),
                TokenType::LeftBracket => self.parse_index_expression(left_exp),
                TokenType::Dot => self.parse_dot_expression(left_exp),
                _ => left_exp,
            };
        }

        left_exp
    }

    fn parse_expression_statement(&mut self) -> Statement {
        let exp = self.parse_expression(Precedence::Lowest);

        if self.next_token.kind == TokenType::Semicolon {
            self.consume_token();
        }

        Statement::Expression(exp)
    }

    fn parse_infix_expression(&mut self, left: Expression, op: Operator) -> Expression {
        self.consume_token();
        let precedence = self.current_precedence();
        self.consume_token();

        let right_expression = self.parse_expression(precedence);

        Expression::Infix(op, Box::new(left), Box::new(right_expression))
    }

    fn parse_prefix_expression(&mut self, pr: Prefix) -> Expression {
        self.consume_token();

        let expr = self.parse_expression(Precedence::Prefix);

        Expression::Prefix(pr, Box::new(expr))
    }

    fn parse_grouped_expression(&mut self) -> Expression {
        self.consume_token();

        let exp = self.parse_expression(Precedence::Lowest);

        if !self.expect_next_token(TokenKind::RightParen) {
            return Expression::Error("unexpected next token: TokenType::RightParen".to_string());
        }

        exp
    }

    fn parse_if_expression(&mut self) -> Expression {
        if !self.expect_next_token(TokenKind::LeftParen) {
            return Expression::Error(format!(
                "expected token: TokenType::LeftParen, got: {:?}",
                self.next_token.kind
            ));
        }

        self.consume_token();

        let condition = self.parse_expression(Precedence::Lowest);

        if !self.expect_next_token(TokenKind::RightParen) {
            return Expression::Error(format!(
                "expected token: TokenType::RightParen, got: {:?}",
                self.next_token.kind
            ));
        }

        if !self.expect_next_token(TokenKind::LeftBrace) {
            return Expression::Error(format!(
                "expected token: TokenType::LeftBrace, got: {:?}",
                self.next_token.kind
            ));
        }

        let consequence = self.parse_block_statement();

        let mut alternative: Option<Block> = None;

        if self.next_token.kind == TokenType::Else {
            self.consume_token();

            if !self.expect_next_token(TokenKind::LeftBrace) {
                return Expression::Error(format!(
                    "else: expected token: TokenType::LeftBrace, got {:?}",
                    self.next_token.kind
                ));
            }

            alternative = Some(self.parse_block_statement());
        }

        Expression::If {
            condition: Box::new(condition),
            consequence,
            alternative,
        }
    }

    fn parse_block_statement(&mut self) -> Block {
        let mut statements: Vec<Statement> = vec![];

        self.consume_token();

        while self.current_token.kind != TokenType::RightBrace
            && self.current_token.kind != TokenType::Eof
        {
            statements.push(self.parse_statement());
            self.consume_token();
        }

        statements
    }

    fn parse_function_expression(&mut self) -> Expression {
        match self.next_token.kind {
            TokenType::Identifier(name) => {
                self.consume_token();

                if !self.expect_next_token(TokenKind::LeftParen) {
                    return Expression::Error(format!(
                        "expected TokenType::LeftParen, got {:?}",
                        self.next_token.kind
                    ));
                }

                let params = self.parse_function_parameters();

                if params.is_none() {
                    return Expression::Error(format!(
                        "expected TokenType::RightParen, got {:?}",
                        self.next_token.kind
                    ));
                }

                if !self.expect_next_token(TokenKind::LeftBrace) {
                    return Expression::Error(format!(
                        "expected TokenType::LeftParen, got {:?}",
                        self.next_token.kind
                    ));
                }

                let body = self.parse_block_statement();

                Expression::Function {
                    identifier: Some(name),
                    parameters: params.unwrap(),
                    body,
                }
            }
            _ => {
                if !self.expect_next_token(TokenKind::LeftParen) {
                    return Expression::Error(format!(
                        "expected TokenType::LeftParen, got {:?}",
                        self.next_token.kind
                    ));
                }

                let params = self.parse_function_parameters();

                if params.is_none() {
                    return Expression::Error(format!(
                        "expected TokenType::RightParen, got {:?}",
                        self.next_token.kind
                    ));
                }

                if !self.expect_next_token(TokenKind::LeftBrace) {
                    return Expression::Error(format!(
                        "expected TokenType::LeftParen, got {:?}",
                        self.next_token.kind
                    ));
                }

                let body = self.parse_block_statement();

                Expression::Function {
                    identifier: None,
                    parameters: params.unwrap(),
                    body,
                }
            }
        }
    }

    fn parse_function_parameters(&mut self) -> Option<Vec<Identifier>> {
        let mut identifiers: Vec<Identifier> = vec![];

        if self.next_token.kind == TokenType::RightParen {
            self.consume_token();
            return Some(identifiers);
        }

        self.consume_token();

        match self.current_token.kind {
            TokenType::Identifier(val) => {
                identifiers.push(val);
            }
            _ => {
                unreachable!(
                    "parse_function_parameters error, unable to get TokenType::Identifier name"
                )
            }
        };

        while self.next_token.kind == TokenType::Comma {
            self.consume_token();
            self.consume_token();

            match self.current_token.kind {
                TokenType::Identifier(val) => {
                    identifiers.push(val);
                }
                _ => {
                    unreachable!(
                        "parse_function_parameters error, unable to get TokenType::Identifier name"
                    )
                }
            };
        }

        if !self.expect_next_token(TokenKind::RightParen) {
            return None;
        }

        Some(identifiers)
    }

    fn parse_call_expression(&mut self, function: Expression) -> Expression {
        self.consume_token();
        let args = self.parse_elements_list(TokenType::RightParen);

        Expression::Call {
            function: Box::new(function),
            arguments: args,
        }
    }

    fn parse_elements_list(&mut self, end: TokenType) -> Vec<Expression> {
        let mut elements: Vec<Expression> = vec![];

        if self.next_token.kind == end {
            self.consume_token();
            return elements;
        }

        self.consume_token();
        elements.push(self.parse_expression(Precedence::Lowest));

        while self.next_token.kind == TokenType::Comma {
            self.consume_token();
            self.consume_token();
            elements.push(self.parse_expression(Precedence::Lowest));
        }

        if !self.expect_next_token(end.to_token_kind()) {
            Expression::Error(format!(
                "parse_call_arguments expected {:?}, got {:?}",
                end, self.next_token.kind
            ));
        }

        elements
    }

    fn parse_index_expression(&mut self, left_exp: Expression) -> Expression {
        self.consume_token();

        let index = self.parse_expression(Precedence::Lowest);

        Expression::Index {
            left: Box::new(left_exp),
            index: Box::new(index),
        }
    }

    fn parse_dot_expression(&mut self, left: Expression) -> Expression {
        self.consume_token();

        let attribute = self.parse_expression(Precedence::Lowest);

        match attribute {
            Expression::Identifier(name) => {
                self.consume_token();

                Expression::Dot {
                    identifier: Box::new(left),
                    attribute: name,
                }
            }
            _ => Expression::Error(format!("Attribute is not valid, got {:?}", attribute)),
        }
    }

    fn parse_hashmaps_literal(&mut self) -> Expression {
        let mut btm: BTreeMap<Expression, Expression> = BTreeMap::new();

        while self.next_token.kind != TokenType::RightBrace {
            self.consume_token();
            let key = self.parse_expression(Precedence::Lowest);

            if !self.expect_next_token(TokenKind::Colon) {
                return Expression::Error(format!(
                    "expected TokenType::Colon, got {:?}",
                    self.next_token
                ));
            }

            self.consume_token();
            let value = self.parse_expression(Precedence::Lowest);

            btm.insert(key, value);

            if self.next_token.kind != TokenType::RightBrace
                && !self.expect_next_token(TokenKind::Comma)
            {
                return Expression::Error(format!(
                    "Expected TokenType::Comma, got {:?}",
                    self.next_token
                ));
            }
        }

        if !self.expect_next_token(TokenKind::RightBrace) {
            return Expression::Error(format!(
                "Expected TokenType::RightBrace, got: {:?}",
                self.next_token.kind
            ));
        }

        Expression::HashMap { pairs: btm }
    }

    fn current_precedence(&self) -> Precedence {
        match self.current_token.kind {
            TokenType::Eq => Precedence::Equals,
            TokenType::NotEq => Precedence::Equals,
            TokenType::LT => Precedence::LessGreaterOrEqual,
            TokenType::GT => Precedence::LessGreaterOrEqual,
            TokenType::Lte => Precedence::LessGreaterOrEqual,
            TokenType::Gte => Precedence::LessGreaterOrEqual,
            TokenType::And => Precedence::AndOr,
            TokenType::Or => Precedence::AndOr,
            TokenType::PlusSign => Precedence::Sum,
            TokenType::MinusSign => Precedence::Sum,
            TokenType::SlashSign => Precedence::Product,
            TokenType::MultiplicationSign => Precedence::Product,
            TokenType::LeftParen => Precedence::Call,
            TokenType::LeftBracket => Precedence::Index,
            TokenType::Dot => Precedence::Dot,
            _ => Precedence::Lowest,
        }
    }

    fn next_precedence(&self) -> u8 {
        match self.next_token.kind {
            TokenType::Eq => Precedence::Equals as u8,
            TokenType::NotEq => Precedence::Equals as u8,
            TokenType::LT => Precedence::LessGreaterOrEqual as u8,
            TokenType::GT => Precedence::LessGreaterOrEqual as u8,
            TokenType::Lte => Precedence::LessGreaterOrEqual as u8,
            TokenType::Gte => Precedence::LessGreaterOrEqual as u8,
            TokenType::And => Precedence::AndOr as u8,
            TokenType::Or => Precedence::AndOr as u8,
            TokenType::PlusSign => Precedence::Sum as u8,
            TokenType::MinusSign => Precedence::Sum as u8,
            TokenType::SlashSign => Precedence::Product as u8,
            TokenType::MultiplicationSign => Precedence::Product as u8,
            TokenType::LeftParen => Precedence::Call as u8,
            TokenType::LeftBracket => Precedence::Index as u8,
            TokenType::Dot => Precedence::Dot as u8,
            TokenType::ExponentSign => Precedence::Exponent as u8,
            _ => Precedence::Lowest as u8,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::{
            expression::{Expression, Operator},
            literal::Literal,
            statement::Statement,
        },
        intern::interner::{Interner, WithInterner},
    };

    use super::Parser;

    #[test]
    fn parse_let_statement() {
        let mut interner = Interner::new();
        let input = "
        let x = 5;
        let y = 100;
        let foobar = y;
        let barfoo = false;
        let myString = \"My string\";
        ";
        let expected = [
            "Let x Number (5)",
            "Let y Number (100)",
            "Let foobar Ident (y)",
            "Let barfoo Bool (false)",
            "Let myString String (My string)",
        ];

        let result = Parser::build_ast(input, &mut interner);

        for (i, curr) in result.statements.iter().enumerate() {
            assert_eq!(
                WithInterner {
                    value: curr,
                    interner: &interner
                }
                .to_string(),
                expected.get(i).unwrap().to_string()
            );
        }
    }

    #[test]
    fn parse_return_statement() {
        let mut interner = Interner::new();

        let input = "
        return 5;
        return 100;
        return foobar + 2;
        ";

        let expected = [
            "Return Number (5)",
            "Return Number (100)",
            "Return + Left Ident (foobar) , Right Number (2)",
        ];
        let result = Parser::build_ast(input, &mut interner);

        for (i, curr) in result.statements.iter().enumerate() {
            assert_eq!(
                WithInterner {
                    value: curr,
                    interner: &interner
                }
                .to_string(),
                expected.get(i).unwrap().to_string()
            );
        }
    }

    #[test]
    fn parse_prefix_expression() {
        let mut interner = Interner::new();

        let input = "
        !5;
        -15;
        !foobar;
        -foobar;
        !true;
        !false;
        5;
        ";

        let expected = [
            "! Number (5)",
            "- Number (15)",
            "! Ident (foobar)",
            "- Ident (foobar)",
            "! Bool (true)",
            "! Bool (false)",
            "Number (5)",
        ];

        let result = Parser::build_ast(input, &mut interner);

        for (i, curr) in result.statements.iter().enumerate() {
            assert_eq!(
                WithInterner {
                    value: curr,
                    interner: &interner
                }
                .to_string(),
                expected.get(i).unwrap().to_string()
            );
        }
    }

    #[test]
    fn parse_infix_expression() {
        let mut interner = Interner::new();

        let input = "
        5 + 5;
        5 - 5;
        5 * 5;
        5 / 5;
        5 > 5;
        5 < 5;
        5 == 5;
        5 != 5;
        5 >= 5;
        5 <= 5;
        false && true;
        true || true;
        foobar + foobar;
        bar - bar;
        bar * bar;
        true == true;
        false != true;
        5 + 5 * 5;
        -1 + 2;
        a + b * c + d / e - f;
        3 > 5 == false;
        ";

        let expected = [
            "+ Left Number (5) , Right Number (5)",
            "- Left Number (5) , Right Number (5)",
            "* Left Number (5) , Right Number (5)",
            "/ Left Number (5) , Right Number (5)",
            "> Left Number (5) , Right Number (5)",
            "< Left Number (5) , Right Number (5)",
            "== Left Number (5) , Right Number (5)",
            "!= Left Number (5) , Right Number (5)",
            ">= Left Number (5) , Right Number (5)",
            "<= Left Number (5) , Right Number (5)",
            "&& Left Bool (false) , Right Bool (true)",
            "|| Left Bool (true) , Right Bool (true)",
            "+ Left Ident (foobar) , Right Ident (foobar)",
            "- Left Ident (bar) , Right Ident (bar)",
            "* Left Ident (bar) , Right Ident (bar)",
            "== Left Bool (true) , Right Bool (true)",
            "!= Left Bool (false) , Right Bool (true)",
            "+ Left Number (5) , Right * Left Number (5) , Right Number (5)",
            "+ Left - Number (1) , Right Number (2)",
            "- Left + Left + Left Ident (a) , Right * Left Ident (b) , Right Ident (c) , Right / Left Ident (d) , Right Ident (e) , Right Ident (f)",
            "== Left > Left Number (3) , Right Number (5) , Right Bool (false)",
        ];

        let result = Parser::build_ast(input, &mut interner);

        for (i, curr) in result.statements.iter().enumerate() {
            assert_eq!(
                WithInterner {
                    value: curr,
                    interner: &interner
                }
                .to_string(),
                expected.get(i).unwrap().to_string()
            );
        }
    }

    #[test]
    fn parse_grouped_expression() {
        let mut interner = Interner::new();

        let input = "
        1 + (2 + 3) + 4;
        (5 + 5) * 2;
        2 / (5 + 5);
        -(5 + 5);
        ";

        let expected = [
            "+ Left + Left Number (1) , Right + Left Number (2) , Right Number (3) , Right Number (4)",
            "* Left + Left Number (5) , Right Number (5) , Right Number (2)",
            "/ Left Number (2) , Right + Left Number (5) , Right Number (5)",
            "- + Left Number (5) , Right Number (5)",
        ];

        let result = Parser::build_ast(input, &mut interner);

        for (i, curr) in result.statements.iter().enumerate() {
            assert_eq!(
                WithInterner {
                    value: curr,
                    interner: &interner
                }
                .to_string(),
                expected.get(i).unwrap().to_string()
            );
        }
    }

    #[test]
    fn parse_if_expression() {
        let mut interner = Interner::new();

        let input = "
        if (x > y) {
            return x;
        }
        ";

        let expected = ["If > Left Ident (x) , Right Ident (y) { Return Ident (x) }"];

        let result = Parser::build_ast(input, &mut interner);

        for (i, curr) in result.statements.iter().enumerate() {
            assert_eq!(
                WithInterner {
                    value: curr,
                    interner: &interner
                }
                .to_string(),
                expected.get(i).unwrap().to_string()
            );
        }
    }

    #[test]
    fn parse_if_else_expression() {
        let mut interner = Interner::new();

        let input = "
        if (x > y) {
            return x;
        } else {
            return y;
        }
        ";

        let expected =
            ["If > Left Ident (x) , Right Ident (y) { Return Ident (x) } else Return Ident (y)"];

        let result = Parser::build_ast(input, &mut interner);

        for (i, curr) in result.statements.iter().enumerate() {
            assert_eq!(
                WithInterner {
                    value: curr,
                    interner: &interner
                }
                .to_string(),
                expected.get(i).unwrap().to_string()
            );
        }
    }

    #[test]
    fn parse_function_parameters() {
        let mut interner = Interner::new();

        let input = "
        fn abc(x, y, w, z, a, b, c) { }
        ";

        let expected = ["Fn abc ( x, y, w, z, a, b, c ) "];

        let result = Parser::build_ast(input, &mut interner);

        for (i, curr) in result.statements.iter().enumerate() {
            assert_eq!(
                WithInterner {
                    value: curr,
                    interner: &interner
                }
                .to_string(),
                expected.get(i).unwrap().to_string()
            );
        }
    }

    #[test]
    fn parse_function_expression() {
        let mut interner = Interner::new();

        let input = "
        fn abc(x, y) {
            return x;
        }

        fn xyz(a) {
            return a + 3;
        }
        ";

        let expected = [
            "Fn abc ( x, y ) Return Ident (x)",
            "Fn xyz ( a ) Return + Left Ident (a) , Right Number (3)",
        ];
        let result = Parser::build_ast(input, &mut interner);

        for (i, curr) in result.statements.iter().enumerate() {
            assert_eq!(
                WithInterner {
                    value: curr,
                    interner: &interner
                }
                .to_string(),
                expected.get(i).unwrap().to_string()
            );
        }
    }

    #[test]
    fn parse_call_expression() {
        let mut interner = Interner::new();

        let input = "
        add(1, 2 * 3, 4 + 5);
        multiply (1, 2);
        ";

        let expected = [
            "Call Ident (add) , Number (1), * Left Number (2) , Right Number (3), + Left Number (4) , Right Number (5)",
            "Call Ident (multiply) , Number (1), Number (2)",
        ];
        let result = Parser::build_ast(input, &mut interner);

        for (i, curr) in result.statements.iter().enumerate() {
            assert_eq!(
                WithInterner {
                    value: curr,
                    interner: &interner
                }
                .to_string(),
                expected.get(i).unwrap().to_string()
            );
        }
    }

    #[test]
    fn parse_string_expression() {
        let mut interner = Interner::new();

        let input = "
        \"Hello world\";
        ";

        let expected = ["String (Hello world)"];
        let result = Parser::build_ast(input, &mut interner);

        for (i, curr) in result.statements.iter().enumerate() {
            assert_eq!(
                WithInterner {
                    value: curr,
                    interner: &interner
                }
                .to_string(),
                expected.get(i).unwrap().to_string()
            );
        }
    }

    #[test]
    fn parse_arrays_expression() {
        let mut interner = Interner::new();

        let input = "
        [1, 2, 3];
        let a = [\"hello\", \"world\"];
        let b = [];
        ";

        let expected = [
            "[ Number (1), Number (2), Number (3) ]",
            "Let a [ String (hello), String (world) ]",
            "Let b [  ]",
        ];
        let result = Parser::build_ast(input, &mut interner);

        for (i, curr) in result.statements.iter().enumerate() {
            assert_eq!(
                WithInterner {
                    value: curr,
                    interner: &interner
                }
                .to_string(),
                expected.get(i).unwrap().to_string()
            );
        }
    }

    #[test]
    fn parse_index_operators() {
        let mut interner = Interner::new();

        let input = "
        arr[1];
        [1, 2, 3][100];
        ";

        let expected = [
            "(Ident (arr) [[ Number (1) ]])",
            "([ Number (1), Number (2), Number (3) ] [[ Number (100) ]])",
        ];
        let result = Parser::build_ast(input, &mut interner);

        for (i, curr) in result.statements.iter().enumerate() {
            assert_eq!(
                WithInterner {
                    value: curr,
                    interner: &interner
                }
                .to_string(),
                expected.get(i).unwrap().to_string()
            );
        }
    }

    #[test]
    fn parse_hashmap_operations() {
        let mut interner = Interner::new();

        let input = "
        let a = {};
        let b = { \"one\": 1, \"two\": 2, \"three\": \"three\" };
        {\"one\": 0 + 1, \"two\": 2 * 1, \"three\": (0 + 1) * 3 }
        ";

        let expected = [
            "Let a {  }",
            "Let b { String (one) : Number (1), String (three) : String (three), String (two) : Number (2) }",
            "{ String (one) : + Left Number (0) , Right Number (1), String (three) : * Left + Left Number (0) , Right Number (1) , Right Number (3), String (two) : * Left Number (2) , Right Number (1) }",
        ];
        let result = Parser::build_ast(input, &mut interner);

        for (i, curr) in result.statements.iter().enumerate() {
            assert_eq!(
                WithInterner {
                    value: curr,
                    interner: &interner
                }
                .to_string(),
                expected.get(i).unwrap().to_string()
            );
        }
    }

    #[test]
    fn parse_closure() {
        let mut interner = Interner::new();

        let input = "
        let a = fn() {
            let b = fn(a) {
                return a;
            };
            return b;
        };

        let c = fn() {
            fn d(a) {
                return a;
            };
            return d;
        };
        ";

        let expected = [
            "Let a Fn (  ) Let b Fn ( a ) Return Ident (a), Return Ident (b)",
            "Let c Fn (  ) Fn d ( a ) Return Ident (a), Return Ident (d)",
        ];
        let result = Parser::build_ast(input, &mut interner);

        for (i, curr) in result.statements.iter().enumerate() {
            assert_eq!(
                WithInterner {
                    value: curr,
                    interner: &interner
                }
                .to_string(),
                expected.get(i).unwrap().to_string()
            );
        }
    }

    #[test]
    fn parse_while_statements() {
        let mut interner = Interner::new();

        let input = "
            while (i < 10) {
                let a = 0;
                a = 11;
            }
        ";

        let expected = [
            "while ( < Left Ident (i) , Right Number (10) ) { [Let a Number (0), = a Number (11)] }",
        ];
        let result = Parser::build_ast(input, &mut interner);

        for (i, curr) in result.statements.iter().enumerate() {
            assert_eq!(
                WithInterner {
                    value: curr,
                    interner: &interner
                }
                .to_string(),
                expected.get(i).unwrap().to_string()
            );
        }
    }

    #[test]
    fn parse_dot_expressions() {
        let mut interner = Interner::new();

        let input = "
            myIdentifier.myAttribute;
        ";

        let expected = ["myAttribute of Ident (myIdentifier)"];
        let result = Parser::build_ast(input, &mut interner);

        for (i, curr) in result.statements.iter().enumerate() {
            assert_eq!(
                WithInterner {
                    value: curr,
                    interner: &interner
                }
                .to_string(),
                expected.get(i).unwrap().to_string()
            );
        }
    }

    #[test]
    fn parse_exponent_expressions() {
        let mut interner = Interner::new();

        let input = "2**3;";
        let result = Parser::build_ast(input, &mut interner);

        assert_eq!(
            result.statements,
            vec![Statement::Expression(Expression::Infix(
                Operator::Exponent,
                Box::new(Expression::Literal(Literal::Number(2))),
                Box::new(Expression::Literal(Literal::Number(3))),
            ))]
        )
    }

    #[test]
    fn parse_exponent_expressions_identifier() {
        let mut interner = Interner::new();

        let var_sym = interner.intern("var");
        let other_var_sym = interner.intern("other_var");

        let input = "var**other_var;";
        let result = Parser::build_ast(input, &mut interner);

        assert_eq!(
            result.statements,
            vec![Statement::Expression(Expression::Infix(
                Operator::Exponent,
                Box::new(Expression::Identifier(var_sym)),
                Box::new(Expression::Identifier(other_var_sym)),
            ))]
        )
    }

    #[test]
    fn parse_exponent_expressions_associativity() {
        let mut interner = Interner::new();

        let input = "2**3**2;";
        let result = Parser::build_ast(input, &mut interner);

        assert_eq!(
            result.statements,
            vec![Statement::Expression(Expression::Infix(
                Operator::Exponent,
                Box::new(Expression::Literal(Literal::Number(2))),
                Box::new(Expression::Infix(
                    Operator::Exponent,
                    Box::new(Expression::Literal(Literal::Number(3))),
                    Box::new(Expression::Literal(Literal::Number(2)))
                )),
            ))]
        )
    }
}
