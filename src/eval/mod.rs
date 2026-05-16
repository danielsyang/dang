use std::collections::HashMap;

use crate::ast::statement::Block;

use self::{env::Environment, object::Object};

pub mod env;
pub mod object;
pub mod program;

fn builtin_len(args: Vec<Object>) -> Object {
    if args.len() != 1 {
        return Object::Error(format!(
            "'len' does not accept more than 1 argument, got: {:?}",
            args
        ));
    }

    match args.get(0).unwrap() {
        Object::String(s) => Object::Number(s.len().try_into().unwrap()),
        Object::Array(arr) => Object::Number(arr.len() as i64),
        _ => Object::Error(format!("invalid argument, got: {:?}", args)),
    }
}

fn builtin_first(args: Vec<Object>) -> Object {
    if args.len() != 1 {
        return Object::Error(format!(
            "'first' does not accept more than 1 argument, got: {:?}",
            args
        ));
    }

    match args.get(0).unwrap() {
        Object::Array(arr) => match arr.first() {
            None => Object::None,
            Some(v) => v.clone(),
        },
        _ => Object::Error(format!("invalid argument, got: {:?}", args)),
    }
}

fn builtin_last(args: Vec<Object>) -> Object {
    if args.len() != 1 {
        return Object::Error(format!(
            "'last' does not accept more than 1 argument, got: {:?}",
            args
        ));
    }

    match args.get(0).unwrap() {
        Object::Array(arr) => match arr.last() {
            None => Object::None,
            Some(v) => v.clone(),
        },
        _ => Object::Error(format!("invalid argument, got: {:?}", args)),
    }
}

fn builtin_print(args: Vec<Object>) -> Object {
    let line = args
        .iter()
        .map(|a| a.to_string())
        .collect::<Vec<_>>()
        .join(" ");

    println!("{}", line);

    Object::None
}

pub fn eval_block(block: &Block, env: &mut Environment) -> Object {
    let mut result = Object::None;

    for sttm in block {
        match sttm.eval(env) {
            Object::Return(r) => return Object::Return(r),
            _ => result = sttm.eval(env),
        }
    }

    result
}

pub fn eval_function_block(block: &Block, env: &mut Environment) -> Option<Object> {
    for sttm in block {
        match sttm.eval(env) {
            Object::Return(r) => {
                let mut result = r.as_ref().clone();

                while let Object::Return(l) = &result {
                    result = l.as_ref().clone();
                }

                return Some(result);
            }
            _ => sttm.eval(env),
        };
    }

    None
}

pub fn builtin_functions() -> Environment {
    let len_func = Object::Builtin { func: builtin_len };
    let first_func = Object::Builtin {
        func: builtin_first,
    };
    let last_func = Object::Builtin { func: builtin_last };

    let print_func = Object::Builtin {
        func: builtin_print,
    };

    let mut store: HashMap<String, Object> = HashMap::new();

    store.insert(String::from("len"), len_func);
    store.insert(String::from("first"), first_func);
    store.insert(String::from("last"), last_func);
    store.insert(String::from("print"), print_func);

    Environment { store }
}

#[cfg(test)]
mod test {
    use super::env::Environment;
    use crate::ast::parser::Parser;

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
            let program = Parser::build_ast(input);
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
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
            "1 >= 1",
            "1 <= 2",
            "true && true",
            "true || false",
            "false && false",
            "(false || true) || false",
        ];
        let expected = [
            "true", "false", "true", "false", "false", "true", "true", "false", "true", "true",
            "true", "true", "true", "true", "false", "true",
        ];

        for (i, input) in inputs.iter().enumerate() {
            let program = Parser::build_ast(input);
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_bang_expression() {
        let mut env = Environment::new();
        let inputs = ["!true;", "!false;", "!!true;", "!!false;"];
        let expected = ["false", "true", "true", "false"];
        for (i, input) in inputs.iter().enumerate() {
            let program = Parser::build_ast(input);
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_if_else_expression() {
        let mut env = Environment::new();
        let inputs = [
            "if (true) { return 10; };",
            "if (true) { return 10; } else { return 20; };",
            "if (false) { return 10; } else { return 20; }",
            "if (false) { return 10; };",
        ];
        let expected = ["10", "10", "20", "None"];
        for (i, input) in inputs.iter().enumerate() {
            let program = Parser::build_ast(input);
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
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
            let program = Parser::build_ast(input);
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
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
            "foobar;",
        ];
        let expected = ["5", "25", "5", "15", "error: identifier not found: foobar"];

        for (i, input) in inputs.iter().enumerate() {
            let program = Parser::build_ast(input);
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_reassign_statements() {
        let mut env = Environment::new();
        let inputs = ["let a = 5; a = 10; a;", "b = 10;"];
        let expected = ["10", "error: Identifier not found: b"];

        for (i, input) in inputs.iter().enumerate() {
            let program = Parser::build_ast(input);
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_function_block() {
        let mut env = Environment::new();
        let inputs = ["fn abc(x) { x + 2; };"];
        let expected = ["Fn abc ( x ) { + Left Ident (x) , Right Number (2) }"];

        for (i, input) in inputs.iter().enumerate() {
            let program = Parser::build_ast(input);
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_function_application() {
        let mut env = Environment::new();
        let inputs = [
            "fn abc(x) { return x * x; }; abc(5);",
            "fn add(x, y) { return x + y; }; add(5 + 5, add(5, 5));",
            "let abc = fn(x) { return x * x; }; abc(5)",
            "let a = fn() {
                let b = 10;
                return b + 10;
            }; a()",
        ];
        let expected = ["25", "20", "25", "20"];

        for (i, input) in inputs.iter().enumerate() {
            let program = Parser::build_ast(input);
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_function_closures() {
        let mut env = Environment::new();
        let inputs = [
            "fn abc(x) {
                return fn inner(y) {
                    return x + y;
                };
            };
            let first = abc(2);
            first(2);",
            "let abcd = fn(a, b) {
                let c = a + b;

                fn inner(d) {
                    return c + d;
                };

                return inner;
            };
            let first = abcd(2, 2);
            first(2);",
        ];
        let expected = ["4", "6"];

        for (i, input) in inputs.iter().enumerate() {
            let program = Parser::build_ast(input);
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_builtin_len() {
        let mut env = Environment::new();
        let inputs = [
            "len(\"\")",
            "len(\"four\")",
            "len(\"hello world\")",
            "len(1)",
        ];
        let expected = ["0", "4", "11", "error: invalid argument, got: [Number(1)]"];

        for (i, input) in inputs.iter().enumerate() {
            let program = Parser::build_ast(input);
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_arrays_expression() {
        let mut env = Environment::new();
        let inputs = ["[1, 2, 3];", "[1, 2 + 2, 3 + 3];"];
        let expected = ["[ 1, 2, 3 ]", "[ 1, 4, 6 ]"];

        for (i, input) in inputs.iter().enumerate() {
            let program = Parser::build_ast(input);
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_indexes_arrays() {
        let mut env = Environment::new();
        let inputs = ["[1, 2, 3][0];", "[1, 2 + 2, 3 + 3][2];"];
        let expected = ["1", "6"];

        for (i, input) in inputs.iter().enumerate() {
            let program = Parser::build_ast(input);
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_hashmaps() {
        let mut env = Environment::new();
        let inputs = ["{\"one\": 10 - 9 }"];
        let expected = ["{ \"one\" : 1 }"];

        for (i, input) in inputs.iter().enumerate() {
            let program = Parser::build_ast(input);
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_hashmaps_value() {
        let mut env = Environment::new();
        let inputs =
            ["let a = {\"one\": 10 - 9, \"two\": (1 * 1) + 1, 3: \"three\" }; a[\"one\"];"];
        let expected = ["1"];

        for (i, input) in inputs.iter().enumerate() {
            let program = Parser::build_ast(input);
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_while_loops() {
        let mut env = Environment::new();
        let inputs = ["
        let i = 0;
        while (i < 10) {
            i = i + 1;
        }
        i;
        "];
        let expected = ["10"];

        for (i, input) in inputs.iter().enumerate() {
            let program = Parser::build_ast(input);
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_while_condition_expression() {
        let mut env = Environment::new();
        let inputs = ["
        let i = 0;
        fn isBigger(i) {
            if (i < 10) {
                return true;
            }
            return false;
        }
        while (isBigger(i)) {
            i = i + 1;
        }
        i;
        "];
        let expected = ["10"];

        for (i, input) in inputs.iter().enumerate() {
            let program = Parser::build_ast(input);
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_dot_expressions() {
        let mut env = Environment::new();
        let inputs = ["
            let myHashMap = {
                \"one\": 1,
                \"two\": 2,
            };

            myHashMap.one;
        "];
        let expected = ["1"];

        for (i, input) in inputs.iter().enumerate() {
            let program = Parser::build_ast(input);
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_dot_expressions_none() {
        let mut env = Environment::new();
        let inputs = ["
            let myHashMap = {
                \"one\": 1,
                \"two\": 2,
            };

            myHashMap.none;
        "];
        let expected = ["None"];

        for (i, input) in inputs.iter().enumerate() {
            let program = Parser::build_ast(input);
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn len_of_string() {
        let inputs = [
            r#"len("Hello")"#,
            r#"len([1, 2, 3])"#,
            r#"first([10, 20, 30])"#,
        ];
        let expected = [5, 3, 10];

        for (i, input) in inputs.iter().enumerate() {
            let mut env = Environment::new();
            let program = Parser::build_ast(input);
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
        }
    }
}
