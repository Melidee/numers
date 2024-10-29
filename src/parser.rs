use crate::error::ParseError;
use anyhow::{Context, Result};

#[derive(PartialEq, Debug, Clone)]
enum Token {
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

impl Token {
    fn is_operator(&self) -> bool {
        [
            Token::Add,
            Token::Subtract,
            Token::Multiply,
            Token::Divide,
            Token::Exponent,
        ]
        .contains(self)
    }

    fn presidence(&self) -> i32 {
        match self {
            Token::Add => 2,
            Token::Subtract => 2,
            Token::Multiply => 3,
            Token::Divide => 3,
            Token::Exponent => 4,
            _ => 1,
        }
    }

    fn is_left_associative(&self) -> bool {
        match self {
            Token::Add | Token::Subtract | Token::Multiply | Token::Divide => true,
            Token::Exponent => false,
            _ => false,
        }
    }

    fn is_number(&self) -> bool {
        match self {
            Token::Number(_) => true,
            _ => false,
        }
    }

    fn is_identifier(&self) -> bool {
        match self {
            Token::Identifier(_) => true,
            _ => false,
        }
    }
}

const DIGITS: &str = ".0123456789";
const ALPHABET: &str = "_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
fn tokenize(source: &str) -> Result<Vec<Token>> {
    let mut tokens: Vec<Token> = vec![];
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
                tokens.push(Token::Number(parsed));
            }
            'a'..'z' | 'A'..'Z' | '_' => {
                let mut identifier = ch.to_string();
                while let Some(next_letter) = chars.peek() {
                    if ALPHABET.contains(*next_letter) {
                        identifier.push(*next_letter);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Identifier(identifier));
            }
            '+' => tokens.push(Token::Add),
            '-' => tokens.push(Token::Subtract),
            '*' => tokens.push(Token::Multiply),
            '/' => tokens.push(Token::Divide),
            '^' => tokens.push(Token::Exponent),
            '=' => tokens.push(Token::Assign),
            ',' => tokens.push(Token::Comma),
            '(' => tokens.push(Token::OpenParen),
            ')' => tokens.push(Token::CloseParen),
            ' ' | '\t' => {}
            _ => return Err(ParseError::InvalidCharacter(ch).into()),
        }
    }
    return Ok(tokens);
}

/// Converts an infix expression to reverse polish notation to make evaluation simpler.
/// This function is an implementation of the shunting yard algorithm.
/// https://en.wikipedia.org/wiki/Shunting_yard_algorithm#The_algorithm_in_detail
fn infix_to_rpn(expr: Vec<Token>) -> Result<Vec<Token>> {
    let mut output: Vec<Token> = vec![];
    let mut stack: Vec<Token> = vec![];
    let mut tokens = expr.iter().peekable();

    let should_pop = |t: &Token, stack: &Vec<Token>| {
        if stack.is_empty() {
            return false;
        }
        let last = stack[stack.len() - 1].clone();
        last != Token::OpenParen
            && (last.presidence() > t.presidence()
                || last.presidence() >= t.presidence() && t.is_left_associative())
    };

    while let Some(token) = tokens.next() {
        let next_is_opening = if let Some(next) = tokens.peek() {
            next == &&Token::OpenParen
        } else {
            false
        };
        match token {
            Token::OpenParen => stack.push(token.clone()),
            Token::CloseParen => {
                while let Some(t) = stack.pop() {
                    if t != Token::OpenParen {
                        output.push(t.clone());
                    }
                }
            }
            Token::Comma => {
                while let Some(t) = stack.last() {
                    if t != &Token::OpenParen {
                        output.push(t.clone());
                        stack.pop();
                    }
                }
            }
            Token::Identifier(_) if next_is_opening => stack.push(token.clone()), // if identifier is a function
            Token::Identifier(_) | Token::Number(_) => output.push(token.clone()),
            _ => {
                // any operator
                while should_pop(token, &stack) {
                    output.push(stack.pop().unwrap());
                }
                stack.push(token.clone());
            }
        }
    }
    eprintln!("stack: {:?}\noutput: {:?}\ntokens: {:?}\n", stack, output, tokens.collect::<Vec<&Token>>());
    stack.iter().rev().for_each(|op| output.push(op.clone()));
    return Ok(output);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_single_number() {
        let source = "1";
        let expected = vec![Token::Number(1.0)];

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
        let expected = vec![Token::Number(1.0)];

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
        let expected = vec![Token::Identifier("var".to_string())];

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
        let expected = vec![Token::Number(1.0), Token::Add, Token::Number(2.0)];

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
            Token::Number(1.0),
            Token::Add,
            Token::Number(2.0),
            Token::Subtract,
            Token::Number(3.0),
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
            Token::Number(1.0),
            Token::Add,
            Token::Number(2.0),
            Token::Subtract,
            Token::Number(3.0),
            Token::Multiply,
            Token::Number(4.0),
            Token::Divide,
            Token::Number(5.0),
            Token::Exponent,
            Token::Number(6.0),
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
        let expected = vec![Token::OpenParen, Token::CloseParen];

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
            Token::Number(1.0),
            Token::Add,
            Token::OpenParen,
            Token::Number(2.0),
            Token::Subtract,
            Token::Number(3.0),
            Token::CloseParen,
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
            Token::Identifier("f".to_string()),
            Token::OpenParen,
            Token::CloseParen,
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
            Token::Identifier("func".to_string()),
            Token::OpenParen,
            Token::Identifier("x".to_string()),
            Token::CloseParen,
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
            Token::Identifier("f".to_string()),
            Token::OpenParen,
            Token::Number(1.0),
            Token::Add,
            Token::Number(2.0),
            Token::Comma,
            Token::Identifier("g".to_string()),
            Token::OpenParen,
            Token::Identifier("x".to_string()),
            Token::CloseParen,
            Token::Comma,
            Token::Identifier("var".to_string()),
            Token::CloseParen,
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
            Token::Number(1.0),
            Token::Add,
            Token::Number(2.0),
        ];
        let expected = vec![
            // [1 2 +]
            Token::Number(1.0),
            Token::Number(2.0),
            Token::Add,
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
            Token::Number(1.0),
            Token::Add,
            Token::Number(2.0),
            Token::Subtract,
            Token::Number(3.0),
            Token::Multiply,
            Token::Number(4.0),
            Token::Divide,
            Token::Number(5.0),
            Token::Exponent,
            Token::Number(6.0),
        ];
        let expected = vec![
            // [1 2 + 3 4 * 5 6 ^ / -]
            Token::Number(1.0),
            Token::Number(2.0),
            Token::Add,
            Token::Number(3.0),
            Token::Number(4.0),
            Token::Multiply,
            Token::Number(5.0),
            Token::Number(6.0),
            Token::Exponent,
            Token::Divide,
            Token::Subtract,
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
            Token::Number(1.0),
            Token::Add,
            Token::OpenParen,
            Token::Number(2.0),
            Token::Subtract,
            Token::Number(3.0),
            Token::CloseParen,
        ];
        let expected = vec![
            // [1 2 3 - +]
            Token::Number(1.0),
            Token::Number(2.0),
            Token::Number(3.0),
            Token::Subtract,
            Token::Add,
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
            Token::Number(1.0),
            Token::Add,
            Token::OpenParen,
            Token::Identifier("f".to_string()),
            Token::OpenParen,
            Token::Identifier("x".to_string()),
            Token::Comma,
            Token::Identifier("y".to_string()),
            Token::CloseParen,
            Token::Subtract,
            Token::Number(3.0),
            Token::CloseParen,
        ];
        let expected = vec![
            // [ 1 x y f() 3 - +]
            Token::Number(1.0),
            Token::Identifier("x".to_string()),
            Token::Identifier("y".to_string()),
            Token::Identifier("f".to_string()),
            Token::Number(3.0),
            Token::Subtract,
            Token::Add,
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
            Token::Identifier("f".to_string()),
            Token::OpenParen,
            Token::Number(1.0),
            Token::Add,
            Token::Number(2.0),
            Token::Comma,
            Token::Number(3.0),
            Token::Subtract,
            Token::Number(4.0),
            Token::Divide,
            Token::Number(5.0),
            Token::CloseParen,
            Token::Add,
            Token::Number(6.0),
        ];
        let expected = vec![
            // [f ( 1 2 + , 3 4 5 / - ) 6 +]
            Token::Number(1.0),
            Token::Number(2.0),
            Token::Add,
            Token::Number(3.0),
            Token::Number(4.0),
            Token::Number(5.0),
            Token::Divide,
            Token::Subtract,
            Token::Identifier("f".to_string()),
            Token::Number(6.0),
            Token::Add,
            
        ];
        let result = infix_to_rpn(input);
        if let Ok(output) = result {
            assert_eq!(output, expected)
        } else {
            assert!(false)
        }
    }
}
