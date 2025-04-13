use crate::error::CompileError;
use anyhow::{Context, Result};

#[derive(PartialEq, Debug, Clone)]
pub enum Statement {
    Declaration(Declaration),
    Expression(Vec<ParseToken>),
}

#[derive(PartialEq, Debug, Clone)]
pub struct Declaration {
    name: String,
    args: Vec<String>,
    body: Vec<ParseToken>,
}


impl Declaration {}

pub fn parse(source: &str) -> Result<Vec<Statement>> {
    let mut statements: Vec<Statement> = vec![];
    for (line_num, line) in source.split('\n').enumerate() {
        let tokens = tokenize(line).context(format!("on line {line_num}"))?;
        if tokens.contains(&ParseToken::Assign) {
            let (id, expr) = tokens
                .split_once(|t| t == &ParseToken::Assign)
                .expect("there must be at least one ocurrance of '=' in tokens");
            let (name, args) = split_declaration(id)?;
            let body = infix_to_rpn(expr.to_vec()).context(format!("on line {line_num}"))?;
            statements.push(Statement::Declaration(Declaration { name, args, body }));
        } else {
            let rpn = infix_to_rpn(tokens).context(format!("on line {line_num}"))?;
            statements.push(Statement::Expression(rpn));
        }
    }
    return Ok(statements);
}

fn split_declaration(declaration: &[ParseToken]) -> Result<(String, Vec<String>)> {
    let name = if let Some(ParseToken::Identifier(n)) = declaration.get(0) {
        n
    } else {
        return Err(CompileError::InvalidAssignment.into());
    };

    let args = declaration
        .into_iter()
        .skip(1)
        .filter(|token| token.is_identifier())
        .map(|token| match token {
            ParseToken::Identifier(arg) => arg.to_owned(),
            _ => panic!("impossible"),
        })
        .collect();

    return Ok((name.clone(), args));
}

#[derive(PartialEq, Debug, Clone)]
pub enum ParseToken {
    Add,
    Subtract,
    Multiply,
    Divide,
    Exponent,
    Assign,
    OpenParen,
    CloseParen,
    Comma,
    Identifier(String),
    Number(f64),
}

impl ParseToken {
    pub fn is_operator(&self) -> bool {
        [
            ParseToken::Add,
            ParseToken::Subtract,
            ParseToken::Multiply,
            ParseToken::Divide,
            ParseToken::Exponent,
        ]
        .contains(self)
    }

    fn presidence(&self) -> i32 {
        match self {
            ParseToken::Add => 2,
            ParseToken::Subtract => 2,
            ParseToken::Multiply => 3,
            ParseToken::Divide => 3,
            ParseToken::Exponent => 4,
            _ => 1,
        }
    }

    fn is_left_associative(&self) -> bool {
        match self {
            ParseToken::Add | ParseToken::Subtract | ParseToken::Multiply | ParseToken::Divide => {
                true
            }
            ParseToken::Exponent => false,
            _ => false,
        }
    }

    pub fn is_number(&self) -> bool {
        match self {
            ParseToken::Number(_) => true,
            _ => false,
        }
    }

    pub fn is_identifier(&self) -> bool {
        match self {
            ParseToken::Identifier(_) => true,
            _ => false,
        }
    }
}

pub enum EvalUnit {
    Number(f64),
    Variable(String),
    Operation(String, i32),
}

const DIGITS: &str = ".0123456789";
const ALPHABET: &str = "_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
fn tokenize(source: &str) -> Result<Vec<ParseToken>> {
    let mut tokens: Vec<ParseToken> = vec![];
    let mut chars = source.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '0'..'9' | '.' => {
                let mut number = ch.to_string();
                while let Some(next_digit) = chars.peek() {
                    if DIGITS.contains(*next_digit) {
                        number.push(*next_digit);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let parsed = number
                    .parse::<f64>()
                    .context(format!("failed to parse float literal: {}", number))?;
                tokens.push(ParseToken::Number(parsed));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut identifier = ch.to_string();
                while let Some(next_letter) = chars.peek() {
                    if ALPHABET.contains(*next_letter) {
                        identifier.push(*next_letter);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(ParseToken::Identifier(identifier));
            }
            '+' => tokens.push(ParseToken::Add),
            '-' => tokens.push(ParseToken::Subtract),
            '*' => tokens.push(ParseToken::Multiply),
            '/' => tokens.push(ParseToken::Divide),
            '^' => tokens.push(ParseToken::Exponent),
            '=' => tokens.push(ParseToken::Assign),
            ',' => tokens.push(ParseToken::Comma),
            '(' => tokens.push(ParseToken::OpenParen),
            ')' => tokens.push(ParseToken::CloseParen),
            ' ' | '\t' => {}
            _ => return Err(CompileError::InvalidCharacter(ch).into()),
        }
    }
    return Ok(tokens);
}

/// Converts an infix expression to reverse polish notation to make evaluation simpler.
/// This function is an implementation of the shunting yard algorithm.
/// https://en.wikipedia.org/wiki/Shunting_yard_algorithm#The_algorithm_in_detail
fn infix_to_rpn(expr: Vec<ParseToken>) -> Result<Vec<ParseToken>> {
    let mut output: Vec<ParseToken> = vec![];
    let mut stack: Vec<ParseToken> = vec![];
    let mut tokens = expr.iter().peekable();

    let should_pop = |t: &ParseToken, stack: &Vec<ParseToken>| {
        if stack.is_empty() {
            return false;
        }
        let last = stack[stack.len() - 1].clone();
        last != ParseToken::OpenParen
            && (last.presidence() > t.presidence()
                || last.presidence() >= t.presidence() && t.is_left_associative())
    };

    while let Some(token) = tokens.next() {
        let next_is_opening = if let Some(next) = tokens.peek() {
            next == &&ParseToken::OpenParen
        } else {
            false
        };
        match token {
            ParseToken::OpenParen => stack.push(token.clone()),
            ParseToken::CloseParen => {
                /* TOMORROW refactor this so this function returns eval units, 
                and function args are counted, perhaps store values in a buf 
                so if a function is reached you can push the args and count them */
                while !stack.is_empty()
                    && let Some(top) = stack.pop()
                {
                    if top == ParseToken::OpenParen {
                        if !stack.is_empty()
                            && let Some(next_top) = stack.last()
                            && next_top.is_identifier()
                        {
                            output.push(stack.pop().unwrap());
                        }
                        break;
                    } else {
                        output.push(top.clone());
                    }
                }
            }
            ParseToken::Comma => {
                while !stack.is_empty()
                    && let Some(top) = stack.pop()
                {
                    if top == ParseToken::OpenParen {
                        break;
                    } else {
                        output.push(top.clone());
                    }
                }
            }
            ParseToken::Identifier(_) if next_is_opening => {
                stack.push(ParseToken::OpenParen);
                stack.push(token.clone());
            }
            ParseToken::Identifier(_) | ParseToken::Number(_) => output.push(token.clone()),
            _ => {
                // any operator
                while should_pop(token, &stack) {
                    output.push(stack.pop().unwrap());
                }
                stack.push(token.clone());
            }
        }
    }
    stack.iter().rev().for_each(|op| output.push(op.clone()));
    return Ok(output);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_line() {
        let input = "1+2";
        let expected = vec![Statement::Expression(vec![
            ParseToken::Number(1.0),
            ParseToken::Number(2.0),
            ParseToken::Add,
        ])];
        let parsed = parse(&input);
        if let Ok(statements) = parsed {
            assert_eq!(statements, expected)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn parses_multiple_lines() {
        let input = "1+2\n3-4";
        let expected = vec![
            Statement::Expression(vec![
                ParseToken::Number(1.0),
                ParseToken::Number(2.0),
                ParseToken::Add,
            ]),
            Statement::Expression(vec![
                ParseToken::Number(3.0),
                ParseToken::Number(4.0),
                ParseToken::Subtract,
            ]),
        ];
        let parsed = parse(&input);
        if let Ok(statements) = parsed {
            assert_eq!(statements, expected)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn parses_variable_declaration() {
        let input = "var=3";
        let expected = vec![Statement::Declaration(Declaration {
            name: "var".to_string(),
            args: vec![],
            body: vec![ParseToken::Number(3.0)],
        })];
        let parsed = parse(&input);
        if let Ok(statements) = parsed {
            assert_eq!(statements, expected)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn parses_function_declaration() {
        let input = "f(x) = x";
        let expected = vec![Statement::Declaration(Declaration {
            name: "f".to_string(),
            args: vec!["x".to_string()],
            body: vec![ParseToken::Identifier("x".to_string())],
        })];
        let parsed = parse(&input);
        if let Ok(statements) = parsed {
            assert_eq!(statements, expected)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn parses_function_args() {
        let input = "f(x, y, z) = x + y + z";
        let expected = vec![Statement::Declaration(Declaration {
            name: "f".to_string(),
            args: vec!["x".to_string(), "y".to_string(), "z".to_string()],
            body: vec![
                ParseToken::Identifier("x".to_string()),
                ParseToken::Identifier("y".to_string()),
                ParseToken::Add,
                ParseToken::Identifier("z".to_string()),
                ParseToken::Add,
            ],
        })];
        let parsed = parse(&input);
        if let Ok(statements) = parsed {
            assert_eq!(statements, expected)
        } else if let Err(e) = parsed {
            println!("{:?}", e);
            assert!(false)
        }
    }

    #[test]
    fn parses_function_call() {
        let input = "func(x, 3)";
        let expected = vec![Statement::Expression(vec![
            ParseToken::Identifier("x".to_string()),
            ParseToken::Number(3.0),
            ParseToken::Identifier("func".to_string()),
        ])];
        let parsed = parse(&input);
        if let Ok(statements) = parsed {
            assert_eq!(statements, expected)
        } else {
            assert!(false)
        }
    }
    #[test]
    fn parses_function_call_with_arg_expression() {
        let input = "func(x, 3 + 4 * 2)";
        let expected = vec![Statement::Expression(vec![
            // [ x 3 4 2 * + func() ]
            ParseToken::Identifier("x".to_string()),
            ParseToken::Number(3.0),
            ParseToken::Number(4.0),
            ParseToken::Number(2.0),
            ParseToken::Multiply,
            ParseToken::Add,
            ParseToken::Identifier("func".to_string()),
        ])];
        let parsed = parse(&input);
        if let Ok(statements) = parsed {
            assert_eq!(statements, expected)
        } else {
            assert!(false)
        }
    }
    #[test]
    fn tokenize_single_number() {
        let source = "1";
        let expected = vec![ParseToken::Number(1.0)];

        let tokenized = tokenize(&source);
        if let Ok(tokens) = tokenized {
            assert_eq!(expected, tokens)
        } else {
            assert!(false);
        }
    }

    #[test]
    fn tokenize_decimal_number() {
        let source = "1.0";
        let expected = vec![ParseToken::Number(1.0)];

        let tokenized = tokenize(&source);
        if let Ok(tokens) = tokenized {
            assert_eq!(expected, tokens)
        } else {
            assert!(false);
        }
    }

    #[test]
    fn tokenize_single_identifier() {
        let source = "var";
        let expected = vec![ParseToken::Identifier("var".to_string())];

        let tokenized = tokenize(&source);
        if let Ok(tokens) = tokenized {
            assert_eq!(expected, tokens)
        } else {
            assert!(false);
        }
    }

    #[test]
    fn tokenize_1_plus_2() {
        let source = "1+2";
        let expected = vec![
            ParseToken::Number(1.0),
            ParseToken::Add,
            ParseToken::Number(2.0),
        ];

        let tokenized = tokenize(&source);
        if let Ok(tokens) = tokenized {
            assert_eq!(expected, tokens)
        } else {
            assert!(false);
        }
    }

    #[test]
    fn tokenize_with_whitespace() {
        let source = "1 +  2 -\t3";
        let expected = vec![
            ParseToken::Number(1.0),
            ParseToken::Add,
            ParseToken::Number(2.0),
            ParseToken::Subtract,
            ParseToken::Number(3.0),
        ];

        let tokenized = tokenize(&source);
        if let Ok(tokens) = tokenized {
            assert_eq!(expected, tokens)
        } else {
            assert!(false);
        }
    }

    #[test]
    fn tokenize_all_operators() {
        let source = "1+2-3*4/5^6";
        let expected = vec![
            ParseToken::Number(1.0),
            ParseToken::Add,
            ParseToken::Number(2.0),
            ParseToken::Subtract,
            ParseToken::Number(3.0),
            ParseToken::Multiply,
            ParseToken::Number(4.0),
            ParseToken::Divide,
            ParseToken::Number(5.0),
            ParseToken::Exponent,
            ParseToken::Number(6.0),
        ];

        let tokenized = tokenize(&source);
        if let Ok(tokens) = tokenized {
            assert_eq!(expected, tokens)
        } else {
            assert!(false);
        }
    }

    #[test]
    fn tokenize_empty_parenthesis() {
        let source = "()";
        let expected = vec![ParseToken::OpenParen, ParseToken::CloseParen];

        let tokenized = tokenize(&source);
        if let Ok(tokens) = tokenized {
            assert_eq!(expected, tokens)
        } else {
            assert!(false);
        }
    }

    #[test]
    fn tokenize_parenthesis_operation() {
        let source = "1+(2-3)";
        let expected = vec![
            ParseToken::Number(1.0),
            ParseToken::Add,
            ParseToken::OpenParen,
            ParseToken::Number(2.0),
            ParseToken::Subtract,
            ParseToken::Number(3.0),
            ParseToken::CloseParen,
        ];

        let tokenized = tokenize(&source);
        if let Ok(tokens) = tokenized {
            assert_eq!(expected, tokens)
        } else {
            assert!(false);
        }
    }

    #[test]
    fn tokenize_zero_arg_function() {
        let source = "f()";
        let expected = vec![
            ParseToken::Identifier("f".to_string()),
            ParseToken::OpenParen,
            ParseToken::CloseParen,
        ];

        let tokenized = tokenize(&source);
        if let Ok(tokens) = tokenized {
            assert_eq!(expected, tokens)
        } else {
            assert!(false);
        }
    }

    #[test]
    fn tokenize_simple_function() {
        let source = "func(x)";
        let expected = vec![
            ParseToken::Identifier("func".to_string()),
            ParseToken::OpenParen,
            ParseToken::Identifier("x".to_string()),
            ParseToken::CloseParen,
        ];

        let tokenized = tokenize(&source);
        if let Ok(tokens) = tokenized {
            assert_eq!(expected, tokens)
        } else {
            assert!(false);
        }
    }

    #[test]
    fn tokenize_function_complex_args() {
        let source = "f(1+2,g(x),var)";
        let expected = vec![
            ParseToken::Identifier("f".to_string()),
            ParseToken::OpenParen,
            ParseToken::Number(1.0),
            ParseToken::Add,
            ParseToken::Number(2.0),
            ParseToken::Comma,
            ParseToken::Identifier("g".to_string()),
            ParseToken::OpenParen,
            ParseToken::Identifier("x".to_string()),
            ParseToken::CloseParen,
            ParseToken::Comma,
            ParseToken::Identifier("var".to_string()),
            ParseToken::CloseParen,
        ];

        let tokenized = tokenize(&source);
        if let Ok(tokens) = tokenized {
            assert_eq!(expected, tokens)
        } else {
            assert!(false);
        }
    }

    #[test]
    fn tokenize_errors_invalid_number() {
        let source = "10.4.5";
        let tokenized = tokenize(source);
        assert!(tokenized.is_err())
    }

    #[test]
    fn tokenize_errors_invalid_character() {
        let source = "1+}3";
        let tokenized = tokenize(&source);
        assert!(tokenized.is_err())
    }

    #[test]
    fn rpn_conversion_1_plus_2() {
        let input = vec![
            // [1 + 2]
            ParseToken::Number(1.0),
            ParseToken::Add,
            ParseToken::Number(2.0),
        ];
        let expected = vec![
            // [1 2 +]
            ParseToken::Number(1.0),
            ParseToken::Number(2.0),
            ParseToken::Add,
        ];
        let result = infix_to_rpn(input);
        if let Ok(output) = result {
            assert_eq!(output, expected)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn rpn_conversion_all_operators() {
        let input = vec![
            // [1 + 2 - 3 * 4 / 5 ^ 6]
            ParseToken::Number(1.0),
            ParseToken::Add,
            ParseToken::Number(2.0),
            ParseToken::Subtract,
            ParseToken::Number(3.0),
            ParseToken::Multiply,
            ParseToken::Number(4.0),
            ParseToken::Divide,
            ParseToken::Number(5.0),
            ParseToken::Exponent,
            ParseToken::Number(6.0),
        ];
        let expected = vec![
            // [1 2 + 3 4 * 5 6 ^ / -]
            ParseToken::Number(1.0),
            ParseToken::Number(2.0),
            ParseToken::Add,
            ParseToken::Number(3.0),
            ParseToken::Number(4.0),
            ParseToken::Multiply,
            ParseToken::Number(5.0),
            ParseToken::Number(6.0),
            ParseToken::Exponent,
            ParseToken::Divide,
            ParseToken::Subtract,
        ];
        let result = infix_to_rpn(input);
        if let Ok(output) = result {
            assert_eq!(output, expected)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn rpn_conversion_with_parenthesis() {
        let input = vec![
            // [1 + ( 2 + 3)]
            ParseToken::Number(1.0),
            ParseToken::Add,
            ParseToken::OpenParen,
            ParseToken::Number(2.0),
            ParseToken::Subtract,
            ParseToken::Number(3.0),
            ParseToken::CloseParen,
        ];
        let expected = vec![
            // [1 2 3 - +]
            ParseToken::Number(1.0),
            ParseToken::Number(2.0),
            ParseToken::Number(3.0),
            ParseToken::Subtract,
            ParseToken::Add,
        ];
        let result = infix_to_rpn(input);
        if let Ok(output) = result {
            assert_eq!(output, expected)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn rpn_conversion_with_functions() {
        let input = vec![
            // [1 + ( f ( x , y) - 3 ) ]
            ParseToken::Number(1.0),
            ParseToken::Add,
            ParseToken::OpenParen,
            ParseToken::Identifier("f".to_string()),
            ParseToken::OpenParen,
            ParseToken::Identifier("x".to_string()),
            ParseToken::Comma,
            ParseToken::Identifier("y".to_string()),
            ParseToken::CloseParen,
            ParseToken::Subtract,
            ParseToken::Number(3.0),
            ParseToken::CloseParen,
        ];
        let expected = vec![
            // [ 1 x y f() 3 - +]
            ParseToken::Number(1.0),
            ParseToken::Identifier("x".to_string()),
            ParseToken::Identifier("y".to_string()),
            ParseToken::Identifier("f".to_string()),
            ParseToken::Number(3.0),
            ParseToken::Subtract,
            ParseToken::Add,
        ];
        let result = infix_to_rpn(input);
        if let Ok(output) = result {
            assert_eq!(output, expected)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn rpn_conversion_function_arguments() {
        let input = vec![
            // [f ( 1 + 2 , 3 - 4 / 5 ) + 6]
            ParseToken::Identifier("f".to_string()),
            ParseToken::OpenParen,
            ParseToken::Number(1.0),
            ParseToken::Add,
            ParseToken::Number(2.0),
            ParseToken::Comma,
            ParseToken::Number(3.0),
            ParseToken::Subtract,
            ParseToken::Number(4.0),
            ParseToken::Divide,
            ParseToken::Number(5.0),
            ParseToken::CloseParen,
            ParseToken::Add,
            ParseToken::Number(6.0),
        ];
        let expected = vec![
            // [1 2 + , 3 4 5 / - f() 6 +]
            ParseToken::Number(1.0),
            ParseToken::Number(2.0),
            ParseToken::Add,
            ParseToken::Number(3.0),
            ParseToken::Number(4.0),
            ParseToken::Number(5.0),
            ParseToken::Divide,
            ParseToken::Subtract,
            ParseToken::Identifier("f".to_string()),
            ParseToken::Number(6.0),
            ParseToken::Add,
        ];
        let result = infix_to_rpn(input);
        if let Ok(output) = result {
            assert_eq!(output, expected)
        } else {
            assert!(false)
        }
    }
}
