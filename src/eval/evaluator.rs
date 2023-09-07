use crate::lex::token::{Token, TokenType};

use super::object::{Boolean, Number, Object, BOOLEAN_OBJ, NUMBER_OBJ};

pub fn eval_infix_expression(
    left: Box<dyn Object>,
    right: Box<dyn Object>,
    operator: &Token,
) -> Box<dyn Object> {
    match (left.kind(), right.kind(), operator.kind) {
        (NUMBER_OBJ, NUMBER_OBJ, TokenType::PlusSign) => {
            let ln = left.inspect().parse::<i64>().unwrap();
            let rn = right.inspect().parse::<i64>().unwrap();
            Box::new(Number::new(ln + rn))
        }
        (NUMBER_OBJ, NUMBER_OBJ, TokenType::MinusSign) => {
            let ln = left.inspect().parse::<i64>().unwrap();
            let rn = right.inspect().parse::<i64>().unwrap();
            Box::new(Number::new(ln - rn))
        }
        (NUMBER_OBJ, NUMBER_OBJ, TokenType::MultiplicationSign) => {
            let ln = left.inspect().parse::<i64>().unwrap();
            let rn = right.inspect().parse::<i64>().unwrap();
            Box::new(Number::new(ln * rn))
        }
        (NUMBER_OBJ, NUMBER_OBJ, TokenType::SlashSign) => {
            let ln = left.inspect().parse::<i64>().unwrap();
            let rn = right.inspect().parse::<i64>().unwrap();
            Box::new(Number::new(ln / rn))
        }
        (NUMBER_OBJ, NUMBER_OBJ, TokenType::Eq) => {
            let ln = left.inspect().parse::<i64>().unwrap();
            let rn = right.inspect().parse::<i64>().unwrap();
            Box::new(Boolean::new(ln == rn))
        }
        (NUMBER_OBJ, NUMBER_OBJ, TokenType::NotEq) => {
            let ln = left.inspect().parse::<i64>().unwrap();
            let rn = right.inspect().parse::<i64>().unwrap();
            Box::new(Boolean::new(ln != rn))
        }
        (NUMBER_OBJ, NUMBER_OBJ, TokenType::GT) => {
            let ln = left.inspect().parse::<i64>().unwrap();
            let rn = right.inspect().parse::<i64>().unwrap();
            Box::new(Boolean::new(ln > rn))
        }
        (NUMBER_OBJ, NUMBER_OBJ, TokenType::LT) => {
            let ln = left.inspect().parse::<i64>().unwrap();
            let rn = right.inspect().parse::<i64>().unwrap();
            Box::new(Boolean::new(ln < rn))
        }
        (BOOLEAN_OBJ, BOOLEAN_OBJ, TokenType::Eq) => {
            let ln = left.inspect().parse::<bool>().unwrap();
            let rn = right.inspect().parse::<bool>().unwrap();
            Box::new(Boolean::new(ln == rn))
        }
        (BOOLEAN_OBJ, BOOLEAN_OBJ, TokenType::NotEq) => {
            let ln = left.inspect().parse::<bool>().unwrap();
            let rn = right.inspect().parse::<bool>().unwrap();
            Box::new(Boolean::new(ln != rn))
        }
        (_, _, _) => panic!("eval_infix_expression: operation not yet implemented"),
    }
}

#[cfg(test)]
mod test {
    use crate::{ast::parser::Parser, eval::environment::Environment};

    #[test]
    fn eval_integer_expression() {
        let mut env = Environment::new();
        let inputs = [
            "5;",
            "10;",
            "-10;",
            "-5;",
            "5 + 5 + 5 + 5 - 10;",
            "2 * 2 * 2 * 2 * 2;",
            "50 / 2 * 2 + 10;",
            "3 * (3 * 3) + 10;",
            "(5 + 10 * 2 + 15 / 3) * 2 + -10;",
        ];
        let expected = ["5", "10", "-10", "-5", "10", "32", "60", "37", "50"];

        for (i, input) in inputs.iter().enumerate() {
            let mut p = Parser::new(input);
            let program = p.build_ast();
            let result = program.eval_statements(&mut env);
            assert_eq!(result.inspect(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_boolean_expression() {
        let mut env = Environment::new();
        let inputs = [
            "true;",
            "false;",
            "1 < 2;",
            "1 > 2;",
            "1 == 2;",
            "1 != 2;",
            "true == true;",
            "true != true;",
            "1 + 2 == 3;",
            "1 + 2 == 2 + 1;",
        ];
        let expected = [
            "true", "false", "true", "false", "false", "true", "true", "false", "true", "true",
        ];

        for (i, input) in inputs.iter().enumerate() {
            let mut p = Parser::new(input);
            let program = p.build_ast();
            let result = program.eval_statements(&mut env);
            assert_eq!(result.inspect(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_bang_expression() {
        let mut env = Environment::new();
        let inputs = ["!true;", "!false;", "!!true;", "!!false;"];
        let expected = ["false", "true", "true", "false"];
        for (i, input) in inputs.iter().enumerate() {
            let mut p = Parser::new(input);
            let program = p.build_ast();
            let result = program.eval_statements(&mut env);
            assert_eq!(result.inspect(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_if_else_expression() {
        let mut env = Environment::new();
        let inputs = [
            "if (true) { 10 };",
            "if (true) { 10 } else { 20 };",
            "if (false) { 10 } else { 20 }",
        ];
        let expected = ["10", "10", "20"];
        for (i, input) in inputs.iter().enumerate() {
            let mut p = Parser::new(input);
            let program = p.build_ast();
            let result = program.eval_statements(&mut env);
            assert_eq!(result.inspect(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_return_statement() {
        let mut env = Environment::new();
        let inputs = [
            "return 10;",
            "return 2 * 5;",
            "return 2 * 5; 9;",
            "
            if (10 > 1) {
                if (10 > 1) {
                    return 10;
                }
                return 1;
            }
            ",
        ];
        let expected = ["10", "10", "10", "10"];

        for (i, input) in inputs.iter().enumerate() {
            let mut p = Parser::new(input);
            let program = p.build_ast();
            let result = program.eval_statements(&mut env);
            assert_eq!(result.inspect(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_let_statements() {
        let mut env = Environment::new();
        let inputs = [
            "let a = 5; a;",
            "let a = 5 * 5; a;",
            "let a = 5; let b = a; b;",
            "let a = 5; let b = a; let c = a + b + 5; c;",
            // TODO: add error handler
            // "foobar;",
        ];
        let expected = ["5", "25", "5", "15", "identifier not found: foobar"];

        for (i, input) in inputs.iter().enumerate() {
            let mut p = Parser::new(input);
            let program = p.build_ast();
            let result = program.eval_statements(&mut env);
            assert_eq!(result.inspect(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_function_block() {
        let mut env = Environment::new();
        let inputs = ["fn abc(x) { x + 2; };"];
        let expected = ["5"];

        for (i, input) in inputs.iter().enumerate() {
            let mut p = Parser::new(input);
            let program = p.build_ast();
            let result = program.eval_statements(&mut env);
            assert_eq!(result.inspect(), expected.get(i).unwrap().to_string());
        }
    }
}
