#![expect(dead_code)]

use crate::{
    svec,
    tokenizer::{Token, TokenType},
};

#[derive(Debug, PartialEq)]
pub struct IntLiteral {
    value: i32,
}
#[derive(Debug, PartialEq)]
pub struct Identifier {
    name: String,
}
#[derive(Debug, PartialEq)]
pub struct BinaryOp {
    left: Box<Ast>,
    op: String,
    right: Box<Ast>,
}

#[derive(Debug, PartialEq)]
pub enum Ast {
    IntLiteral(IntLiteral),
    BoolLiteral { value: bool },
    NoneLiteral,
    Identifier(Identifier),
    BinaryOp(BinaryOp),
}

// Return the token at pos or the End token
fn peek(tokens: &Vec<Token>, pos: usize) -> Token {
    if pos < tokens.len() {
        tokens[pos].clone()
    } else {
        Token {
            token_type: TokenType::End,
            value: String::from(""),
            location: (1, 1), // TODO:
        }
    }
}
enum Expected {
    None,
    String(String),
    Strings(Vec<String>),
}

// Consume expected, returns token at pos
fn consume(tokens: &Vec<Token>, pos: &mut usize, expected: Expected) -> Token {
    let token = peek(tokens, *pos);
    match expected {
        Expected::String(value) => {
            if token.value != value {
                let location = token.location;
                panic!("{location:?}: Expected \"{value}\"")
            }
        }
        Expected::Strings(values) => {
            if !values.contains(&token.value) {
                let location = token.location;
                panic!("{location:?}: Expected one of \"{values:?}\"")
            }
        }
        Expected::None => {}
    }

    *pos += 1;
    token
}

// Consumes token at pos and returns the integer literal
fn parse_int_literal(tokens: &Vec<Token>, pos: &mut usize) -> IntLiteral {
    let token = peek(tokens, *pos);
    let location = token.location;
    let value = token.value;

    if token.token_type != TokenType::Integer {
        panic!("{location:?}: Expected integer literal")
    }
    consume(tokens, pos, Expected::None);
    IntLiteral {
        value: value
            .parse::<i32>()
            .unwrap_or_else(|_| panic!("{location:?}: Invalid integer \"{value}\"")),
    }
}

// Consumes token at pos and returns the identifier
fn parse_identifier(tokens: &Vec<Token>, pos: &mut usize) -> Identifier {
    let token = peek(tokens, *pos);
    let location = token.location;
    if token.token_type != TokenType::Identifier {
        panic!("{location:?}: Expected identifier")
    }
    consume(tokens, pos, Expected::None);
    Identifier { name: token.value }
}

fn parse_expression(tokens: &Vec<Token>, pos: &mut usize) -> Ast {
    let mut left = parse_term(tokens, pos);
    while svec!["+", "-"].contains(&peek(tokens, *pos).value) {
        let op = consume(tokens, pos, Expected::None);
        let right = parse_term(tokens, pos);
        left = Ast::BinaryOp(BinaryOp {
            left: Box::new(left),
            op: op.value,
            right: Box::new(right),
        })
    }
    left
}

// Parses * / expressions
fn parse_term(tokens: &Vec<Token>, pos: &mut usize) -> Ast {
    let mut left = parse_factor(tokens, pos);
    while svec!["*", "/"].contains(&peek(tokens, *pos).value) {
        let op = consume(tokens, pos, Expected::None);
        let right = parse_factor(tokens, pos);
        left = Ast::BinaryOp(BinaryOp {
            left: Box::new(left),
            op: op.value,
            right: Box::new(right),
        });
    }
    left
}

fn parse_factor(tokens: &Vec<Token>, pos: &mut usize) -> Ast {
    let token = peek(tokens, *pos);
    let location = token.location;
    match token.token_type {
        TokenType::Integer => Ast::IntLiteral(parse_int_literal(tokens, pos)),
        TokenType::Identifier => Ast::Identifier(parse_identifier(tokens, pos)),
        _ => panic!("{location:?}: Expected an integer literal or an identifier"),
    }
}

pub fn parse(tokens: Vec<Token>) -> Ast {
    let mut pos = 0;

    parse_expression(&tokens, &mut pos)
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::tokenize;

    use super::*;

    // Macro rules to make creating asts manually easier
    macro_rules! bast {
        ($left: expr, $op: expr, $right: expr) => {
            Ast::BinaryOp(BinaryOp {
                left: Box::new($left),
                op: $op.to_owned(),
                right: Box::new($right),
            })
        };
    }
    macro_rules! iast {
        ($value: expr) => {
            Ast::IntLiteral(IntLiteral { value: $value })
        };
    }
    macro_rules! idast {
        ($name: expr) => {
            Ast::Identifier(Identifier {
                name: $name.to_owned(),
            })
        };
    }

    // Tests
    #[test]
    fn test_simple_addition() {
        let tokens = tokenize(String::from("1+1"));
        let expected = bast![iast!(1), "+", iast!(1)];
        assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn test_simple_addition_with_identifiers() {
        let tokens = tokenize(String::from("x-50"));
        let expected = bast![idast!("x"), "-", iast!(50)];
        assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn test_multiple_additions() {
        let tokens = tokenize(String::from("2-3+4"));
        let expected = bast![bast![iast!(2), "-", iast!(3)], "+", iast!(4)];
        assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn test_multiplication() {
        let tokens = tokenize(String::from("1*2/3"));
        let expected = bast![bast![iast!(1), "*", iast!(2)], "/", iast!(3)];
        assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn test_multiplication_and_addition() {
        let tokens = tokenize(String::from("1+2*3"));
        let expected = bast![iast!(1), "+", bast![iast!(2), "*", iast!(3)]];
        assert_eq!(parse(tokens), expected);
    }
}
