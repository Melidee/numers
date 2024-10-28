use anyhow::{Context, Result};

#[derive(PartialEq, Debug, Clone)]
enum Token {
    Add,
    Subtract,
    Multiply,
    Divide,
    Exponent,
    Identifier(String),
    Number(f64),
    Assign,
    OpenParen,
    CloseParen,
}

const DIGITS: &str = ".0123456789";
const ALPHABET: &str = "_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
fn tokenize(source: &str) -> Result<Vec<Token>> {
    let mut tokens: Vec<Token> = vec![];
    let mut chars = source.chars().peekable();
    while let Some(char) = chars.next() {
        match char {
            '0'..'9' | '.' => {
                let mut number = char.to_string();
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
                let mut identifier = char.to_string();
                while let Some(next_letter) = chars.peek() {
                    if ALPHABET.contains(*next_letter) {
                        identifier.push(*next_letter);
                        chars.next();
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
            '(' => tokens.push(Token::OpenParen),
            ')' => tokens.push(Token::CloseParen),
            ' ' | '\t' => {}
            _ => return Err(todo!()),
        }
    }
    return Ok(tokens);
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
}
