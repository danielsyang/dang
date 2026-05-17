use crate::{intern::interner::Interner, lex::token::TokenType};

use super::token::Token;

pub struct Lexer {
    input: String,
    position: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
            position: 0,
        }
    }

    pub fn next_token(&mut self, interner: &mut Interner) -> Option<Token> {
        if self.input.len() < self.position {
            return None;
        }
        if self.input.len() == self.position {
            self.position += 1;
            return Some(Token::eof());
        }

        let curr = self.consume_char();

        if Lexer::skip_whitespace_or_new_line(curr) {
            return Some(Token::whitespace());
        }

        if curr.is_alphabetic() {
            let word = self.consume_word(curr);
            match word.as_str() {
                "let" => return Some(Token::new_let()),
                "fn" => return Some(Token::function()),
                "true" => return Some(Token::boolean(true)),
                "false" => return Some(Token::boolean(false)),
                "return" => return Some(Token::new(TokenType::Return)),
                "if" => return Some(Token::new(TokenType::If)),
                "else" => return Some(Token::new(TokenType::Else)),
                "while" => return Some(Token::while_token()),
                _ => {
                    let symbol = interner.intern(word.as_ref());
                    return Some(Token::identifier(symbol));
                }
            }
        }

        if curr.is_ascii_digit() {
            let number = self.consume_number(curr);
            return Some(Token::int(number));
        }

        match curr {
            '=' => match self.peek() {
                Some('=') => {
                    self.consume_char();
                    Some(Token::new(TokenType::Eq))
                }
                _ => Some(Token::assign_sign()),
            },
            '!' => match self.peek() {
                Some('=') => {
                    self.consume_char();
                    Some(Token::new(TokenType::NotEq))
                }
                _ => Some(Token::bang()),
            },
            '.' => Some(Token::dot()),
            ';' => Some(Token::semicolon()),
            '+' => Some(Token::new(TokenType::PlusSign)),
            '-' => Some(Token::new(TokenType::MinusSign)),
            '*' => Some(Token::new(TokenType::MultiplicationSign)),
            '/' => Some(Token::new(TokenType::SlashSign)),
            '{' => Some(Token::left_brace()),
            '}' => Some(Token::right_brace()),
            '(' => Some(Token::left_paren()),
            ')' => Some(Token::right_paren()),
            '[' => Some(Token::left_bracket()),
            ']' => Some(Token::right_bracket()),
            ',' => Some(Token::comma()),
            '<' => match self.peek() {
                Some('=') => {
                    self.consume_char();
                    Some(Token::lte())
                }
                _ => Some(Token::lt()),
            },
            '>' => match self.peek() {
                Some('=') => {
                    self.consume_char();
                    Some(Token::gte())
                }
                _ => Some(Token::gt()),
            },
            '"' => Some(Token::string(self.consume_string())),
            ':' => Some(Token::colon()),
            '&' => match self.peek() {
                Some('&') => {
                    self.consume_char();
                    Some(Token::and())
                }
                // Bitwise operation
                _ => Some(Token::illegal()),
            },
            '|' => match self.peek() {
                Some('|') => {
                    self.consume_char();
                    Some(Token::or())
                }
                // Bitwise operation
                _ => Some(Token::illegal()),
            },
            _ => Some(Token::illegal()),
        }
    }

    fn consume_char(&mut self) -> char {
        let c = self
            .input
            .chars()
            .nth(self.position)
            .expect("Invalid lexer state, current position is larger than input");

        self.position += 1;

        c
    }

    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.position)
    }

    fn consume_word(&mut self, mut initial_char: char) -> String {
        let mut word = String::from("");

        loop {
            word.push(initial_char);

            match self.peek() {
                Some(d) => {
                    if !d.is_alphanumeric() {
                        break;
                    }
                }
                None => {
                    break;
                }
            }

            initial_char = self.consume_char();
        }

        word
    }

    fn consume_string(&mut self) -> String {
        let mut curr = self.consume_char();
        let mut string = String::from("");

        loop {
            match curr {
                '"' => break,
                _ => {
                    string.push(curr);
                    curr = self.consume_char();
                }
            }
        }

        string
    }

    fn consume_number(&mut self, mut initial_char: char) -> i64 {
        let mut number: i64 = 0;

        loop {
            // safely assume we can parse and unwrap because we have validation down below.
            let d = initial_char.to_digit(10).unwrap() as i64;
            number = number * 10 + d;

            match self.peek() {
                Some(v) => {
                    if !v.is_ascii_digit() {
                        break;
                    }
                }
                _ => {
                    break;
                }
            }

            initial_char = self.consume_char();
        }

        number
    }

    fn skip_whitespace_or_new_line(c: char) -> bool {
        if c == ' ' || c == '\n' || c == '\r' {
            return true;
        };

        false
    }
}

#[cfg(test)]
mod test {
    use crate::{
        intern::interner::Interner,
        lex::{
            lexer::Lexer,
            token::{Token, TokenType},
        },
    };

    fn run_tokenizer(mut lex: Lexer, interner: &mut Interner) -> Vec<Token> {
        let mut tokens: Vec<Token> = vec![];

        while let Some(t) = lex.next_token(interner) {
            match t.kind {
                TokenType::Whitespace => {}
                _ => tokens.push(t),
            }
        }

        return tokens;
    }

    #[test]
    fn tokenize_let_statement() {
        let input = "
            let x = 512;
            let y = 256;
            let x1 = 128;
        ";

        let lex = Lexer::new(input);
        let mut interner = Interner::new();

        let x_sym = interner.intern("x");
        let y_sym = interner.intern("y");
        let x1_sym = interner.intern("x1");

        let expected: Vec<Token> = vec![
            Token::new_let(),
            Token::identifier(x_sym),
            Token::assign_sign(),
            Token::int(512),
            Token::semicolon(),
            Token::new_let(),
            Token::identifier(y_sym),
            Token::assign_sign(),
            Token::int(256),
            Token::semicolon(),
            Token::new_let(),
            Token::identifier(x1_sym),
            Token::assign_sign(),
            Token::int(128),
            Token::semicolon(),
            Token::eof(),
        ];
        let result = run_tokenizer(lex, &mut interner);

        assert_eq!(expected, result)
    }

    #[test]
    fn tokenize_if_else_statement() {
        let input = "
            if (x < 10) {
                return 10;
            } else if (x > 12) {
                return 20;
            } else {
                return 30;
            }
        ";

        let lex = Lexer::new(input);
        let mut interner = Interner::new();

        let x_sym = interner.intern("x");

        let expected: Vec<Token> = vec![
            Token::new(TokenType::If),
            Token::left_paren(),
            Token::identifier(x_sym),
            Token::lt(),
            Token::int(10),
            Token::right_paren(),
            Token::left_brace(),
            Token::new(TokenType::Return),
            Token::int(10),
            Token::semicolon(),
            Token::right_brace(),
            Token::new(TokenType::Else),
            Token::new(TokenType::If),
            Token::left_paren(),
            Token::identifier(x_sym),
            Token::gt(),
            Token::int(12),
            Token::right_paren(),
            Token::left_brace(),
            Token::new(TokenType::Return),
            Token::int(20),
            Token::semicolon(),
            Token::right_brace(),
            Token::new(TokenType::Else),
            Token::left_brace(),
            Token::new(TokenType::Return),
            Token::int(30),
            Token::semicolon(),
            Token::right_brace(),
            Token::eof(),
        ];
        let result = run_tokenizer(lex, &mut interner);

        assert_eq!(expected, result)
    }

    #[test]
    fn tokenize_function_statement() {
        let input = "
            let a = fn(x, y) { };

            fn myFunc() { }
        ";

        let lex = Lexer::new(input);
        let mut interner = Interner::new();

        let a_sym = interner.intern("a");
        let x_sym = interner.intern("x");
        let y_sym = interner.intern("y");
        let my_func_sym = interner.intern("myFunc");

        let expected: Vec<Token> = vec![
            Token::new_let(),
            Token::identifier(a_sym),
            Token::assign_sign(),
            Token::function(),
            Token::left_paren(),
            Token::identifier(x_sym),
            Token::comma(),
            Token::identifier(y_sym),
            Token::right_paren(),
            Token::left_brace(),
            Token::right_brace(),
            Token::semicolon(),
            Token::function(),
            Token::identifier(my_func_sym),
            Token::left_paren(),
            Token::right_paren(),
            Token::left_brace(),
            Token::right_brace(),
            Token::eof(),
        ];
        let result = run_tokenizer(lex, &mut interner);

        assert_eq!(expected, result)
    }

    #[test]
    fn tokenize_strings() {
        let input = "
            let abc = \"HELLO\";
            let cde = \"Hello world\";
        ";

        let lex = Lexer::new(input);
        let mut interner = Interner::new();

        let abc_sym = interner.intern("abc");
        let cde_sym = interner.intern("cde");

        let expected: Vec<Token> = vec![
            Token::new_let(),
            Token::identifier(abc_sym),
            Token::assign_sign(),
            Token::string("HELLO".to_string()),
            Token::semicolon(),
            Token::new_let(),
            Token::identifier(cde_sym),
            Token::assign_sign(),
            Token::string("Hello world".to_string()),
            Token::semicolon(),
            Token::eof(),
        ];
        let result = run_tokenizer(lex, &mut interner);

        assert_eq!(expected, result)
    }

    #[test]
    fn tokenize_arrays() {
        let input = "
            let abc = [1, 2, \"hello world\"];
        ";

        let lex = Lexer::new(input);
        let mut interner = Interner::new();

        let abc_sym = interner.intern("abc");

        let expected: Vec<Token> = vec![
            Token::new_let(),
            Token::identifier(abc_sym),
            Token::assign_sign(),
            Token::left_bracket(),
            Token::int(1),
            Token::comma(),
            Token::int(2),
            Token::comma(),
            Token::string("hello world".to_string()),
            Token::right_bracket(),
            Token::semicolon(),
            Token::eof(),
        ];
        let result = run_tokenizer(lex, &mut interner);

        assert_eq!(expected, result)
    }

    #[test]
    fn tokenize_indexes() {
        let input = "
            arr[1];
            [1, 2, 3][100];
        ";

        let lex = Lexer::new(input);
        let mut interner = Interner::new();

        let arr_sym = interner.intern("arr");

        let expected: Vec<Token> = vec![
            Token::identifier(arr_sym),
            Token::left_bracket(),
            Token::int(1),
            Token::right_bracket(),
            Token::semicolon(),
            Token::left_bracket(),
            Token::int(1),
            Token::comma(),
            Token::int(2),
            Token::comma(),
            Token::int(3),
            Token::right_bracket(),
            Token::left_bracket(),
            Token::int(100),
            Token::right_bracket(),
            Token::semicolon(),
            Token::eof(),
        ];
        let result = run_tokenizer(lex, &mut interner);

        assert_eq!(expected, result)
    }

    #[test]
    fn tokenize_hashmaps() {
        let input = "
            {\"foobar\": 10}
        ";

        let lex = Lexer::new(input);
        let mut interner = Interner::new();

        let expected: Vec<Token> = vec![
            Token::left_brace(),
            Token::string("foobar".to_string()),
            Token::colon(),
            Token::int(10),
            Token::right_brace(),
            Token::eof(),
        ];
        let result = run_tokenizer(lex, &mut interner);

        assert_eq!(expected, result)
    }

    #[test]
    fn tokenize_lte_gte() {
        let input = "
            1 >= 2;
            2 <= 1;
        ";

        let lex = Lexer::new(input);
        let mut interner = Interner::new();

        let expected: Vec<Token> = vec![
            Token::int(1),
            Token::gte(),
            Token::int(2),
            Token::semicolon(),
            Token::int(2),
            Token::lte(),
            Token::int(1),
            Token::semicolon(),
            Token::eof(),
        ];
        let result = run_tokenizer(lex, &mut interner);

        assert_eq!(expected, result)
    }

    #[test]
    fn tokenize_and_or() {
        let input = "
            5 && 5;
            5 || 5;
        ";

        let lex = Lexer::new(input);
        let mut interner = Interner::new();

        let expected: Vec<Token> = vec![
            Token::int(5),
            Token::and(),
            Token::int(5),
            Token::semicolon(),
            Token::int(5),
            Token::or(),
            Token::int(5),
            Token::semicolon(),
            Token::eof(),
        ];
        let result = run_tokenizer(lex, &mut interner);

        assert_eq!(expected, result)
    }

    #[test]
    fn closures() {
        let input = "
            let closure = fn(a, b) {
                let c = a + b;
                return fn(d) {
                    return c + d;
                };
            };

            let closure2 = fn() {
                fn test() {

                }
                return test;
            };
        ";

        let lex = Lexer::new(input);
        let mut interner = Interner::new();

        let closure_sym = interner.intern("closure");
        let a_sym = interner.intern("a");
        let b_sym = interner.intern("b");
        let c_sym = interner.intern("c");
        let d_sym = interner.intern("d");
        let closure2_sym = interner.intern("closure2");
        let test_sym = interner.intern("test");

        let expected: Vec<Token> = vec![
            Token::new_let(),
            Token::identifier(closure_sym),
            Token::assign_sign(),
            Token::function(),
            Token::left_paren(),
            Token::identifier(a_sym),
            Token::comma(),
            Token::identifier(b_sym),
            Token::right_paren(),
            Token::left_brace(),
            Token::new_let(),
            Token::identifier(c_sym),
            Token::assign_sign(),
            Token::identifier(a_sym),
            Token::new(TokenType::PlusSign),
            Token::identifier(b_sym),
            Token::semicolon(),
            Token::new(TokenType::Return),
            Token::function(),
            Token::left_paren(),
            Token::identifier(d_sym),
            Token::right_paren(),
            Token::left_brace(),
            Token::new(TokenType::Return),
            Token::identifier(c_sym),
            Token::new(TokenType::PlusSign),
            Token::identifier(d_sym),
            Token::semicolon(),
            Token::right_brace(),
            Token::semicolon(),
            Token::right_brace(),
            Token::semicolon(),
            Token::new_let(),
            Token::identifier(closure2_sym),
            Token::assign_sign(),
            Token::function(),
            Token::left_paren(),
            Token::right_paren(),
            Token::left_brace(),
            Token::function(),
            Token::identifier(test_sym),
            Token::left_paren(),
            Token::right_paren(),
            Token::left_brace(),
            Token::right_brace(),
            Token::new(TokenType::Return),
            Token::identifier(test_sym),
            Token::semicolon(),
            Token::right_brace(),
            Token::semicolon(),
            Token::eof(),
        ];
        let result = run_tokenizer(lex, &mut interner);

        assert_eq!(expected, result)
    }

    #[test]
    fn while_statements() {
        let input = "
            while (true) {
                let a = 0;
            }
        ";

        let lex = Lexer::new(input);
        let mut interner = Interner::new();

        let a_sym = interner.intern("a");

        let expected: Vec<Token> = vec![
            Token::while_token(),
            Token::left_paren(),
            Token::boolean(true),
            Token::right_paren(),
            Token::left_brace(),
            Token::new_let(),
            Token::identifier(a_sym),
            Token::assign_sign(),
            Token::int(0),
            Token::semicolon(),
            Token::right_brace(),
            Token::eof(),
        ];
        let result = run_tokenizer(lex, &mut interner);

        assert_eq!(expected, result)
    }
    #[test]
    fn dot_operator() {
        let input = "
        test.interval;
        ";

        let lex = Lexer::new(input);
        let mut interner = Interner::new();

        let test_sym = interner.intern("test");
        let interval_sym = interner.intern("interval");

        let expected: Vec<Token> = vec![
            Token::identifier(test_sym),
            Token::dot(),
            Token::identifier(interval_sym),
            Token::semicolon(),
            Token::eof(),
        ];
        let result = run_tokenizer(lex, &mut interner);

        assert_eq!(expected, result)
    }
}
