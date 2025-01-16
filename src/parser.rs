#![expect(dead_code)]

use crate::tokenizer::{Token, TokenType};

#[derive(Debug)]
pub struct IntLiteral {
    value: i32,
}
#[derive(Debug)]
pub struct BinaryOp {
    left: Box<Ast>,
    op: String,
    right: Box<Ast>,
}

#[derive(Debug)]
pub enum Ast {
    IntLiteral(IntLiteral),
    BoolLiteral { value: bool },
    NoneLiteral,
    Identifier { name: String },
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
enum Consume {
    None,
    String(String),
    Strings(Vec<String>),
}

// Consume expected, returns token at pos
fn consume(tokens: &Vec<Token>, pos: &mut usize, expected: Consume) -> Token {
    let token = peek(tokens, *pos);
    match expected {
        Consume::String(value) => {
            if token.value != value {
                let location = token.location;
                panic!("{location:?}: Expected \"{value}\"")
            }
        }
        Consume::Strings(values) => {
            if !values.contains(&token.value) {
                let location = token.location;
                panic!("{location:?}: Expected one of \"{values:?}\"")
            }
        }
        Consume::None => {}
    }

    *pos += 1;
    token
}

// Consumes token at pos and returns the integer literal
fn parse_int_literal(tokens: &Vec<Token>, pos: &mut usize) -> IntLiteral {
    let token = peek(tokens, *pos);
    let location = token.location;
    if token.token_type != TokenType::Integer {
        panic!("{location:?}: Expected integer literal")
    }
    consume(tokens, pos, Consume::None);
    IntLiteral {
        value: token
            .value
            .parse::<i32>()
            .unwrap_or_else(|_| panic!("{location:?}: Invalid integer")),
    }
}

fn parse_expression(tokens: &Vec<Token>, pos: &mut usize) -> BinaryOp {
    let left = parse_int_literal(tokens, pos);
    let op = consume(
        tokens,
        pos,
        Consume::Strings(vec![String::from("+"), String::from("-")]),
    );
    let right = parse_int_literal(tokens, pos);
    BinaryOp {
        left: Box::new(Ast::IntLiteral(left)),
        op: op.value,
        right: Box::new(Ast::IntLiteral(right)),
    }
}

pub fn parse(tokens: Vec<Token>) -> Ast {
    let mut pos = 0;

    Ast::BinaryOp(parse_expression(&tokens, &mut pos))
}
