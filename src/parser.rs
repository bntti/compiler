use crate::{
    loc,
    tokenizer::{Token, TokenType},
    util::Location,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Unk,
    Unit,
    Int,
    Bool,
    Function { params: Vec<Type>, ret: Box<Type> },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Ast {
    Root {
        stats: Vec<Ast>,
        typ: Type,
        loc: Location,
    },
    Block {
        stats: Vec<Ast>,
        typ: Type,
        loc: Location,
    },
    Function {
        name: String,
        params: Vec<Ast>,
        typ: Type,
        loc: Location,
    },
    IntLiteral {
        val: i64,
        typ: Type,
        loc: Location,
    },
    BoolLiteral {
        val: bool,
        typ: Type,
        loc: Location,
    },
    NoneLiteral {
        typ: Type,
        loc: Location,
    },
    Identifier {
        name: String,
        typ: Type,
        loc: Location,
    },
    Var {
        name: String,
        value: Box<Ast>,
        typ: Type,
        loc: Location,
    },
    While {
        cond: Box<Ast>,
        then: Box<Ast>,
        typ: Type,
        loc: Location,
    },
    If {
        cond: Box<Ast>,
        then: Box<Ast>,
        els: Box<Option<Ast>>,
        typ: Type,
        loc: Location,
    },
    Minus {
        stat: Box<Ast>,
        typ: Type,
        loc: Location,
    },
    Negate {
        stat: Box<Ast>,
        typ: Type,
        loc: Location,
    },
    BinaryOp {
        left: Box<Ast>,
        op: String,
        right: Box<Ast>,
        typ: Type,
        loc: Location,
    },
}

// Return the token at pos or the End token
fn peek(tokens: &[Token], pos: usize) -> Token {
    if pos < tokens.len() {
        tokens[pos].clone()
    } else {
        Token {
            token_type: TokenType::End,
            value: String::from(""),
            location: loc!(0, 0), // TODO:
        }
    }
}
// Return the token at pos or the End token
fn peek_back(tokens: &[Token], pos: usize) -> Token {
    if pos == 0 {
        panic!("Tried to call peek_back when pos was 0");
    }
    tokens[pos - 1].clone()
}

enum Expected {
    None,
    String(String),
    Token(TokenType),
    Semi,
}

// Consume expected, returns token at pos
fn consume(tokens: &[Token], pos: &mut usize, expected: Expected) -> Token {
    let token = peek(tokens, *pos);
    let location = token.location;
    match expected {
        Expected::String(value) => {
            if token.value != value {
                panic!("{location:?}: Expected \"{value}\"")
            }
        }
        Expected::Token(expected_token) => {
            if token.token_type != expected_token {
                panic!("{location:?}: Expected \"{expected_token:?}\"")
            }
        }
        Expected::Semi => {
            if &token.value != ";" {
                panic!("{location:?}: Expected ';'")
            }
        }
        Expected::None => {}
    }

    *pos += 1;
    token
}

// Consumes token at pos and returns the integer literal
fn parse_int_literal(tokens: &[Token], pos: &mut usize) -> Ast {
    let token = consume(tokens, pos, Expected::Token(TokenType::Integer));
    let location = token.location;
    let value = token.value;
    Ast::IntLiteral {
        val: value
            .parse::<i64>()
            .unwrap_or_else(|_| panic!("{location:?}: Invalid integer \"{value}\"")), // Should never happen?
        typ: Type::Unk,
        loc: token.location,
    }
}

pub fn parse(tokens: Vec<Token>) -> Ast {
    let mut pos = 0;
    parse_block(
        &tokens,
        &mut pos,
        Ast::Root {
            stats: vec![],
            typ: Type::Unk,
            loc: loc!(0, 0),
        },
    )
}

fn parse_block(tokens: &Vec<Token>, pos: &mut usize, parent: Ast) -> Ast {
    let mut statements = vec![];
    match parent {
        Ast::Root {
            stats: _,
            typ: _,
            loc: location,
        } => {
            while peek(tokens, *pos).token_type != TokenType::End {
                statements.push(parse_line(tokens, pos));

                // Don't require ';' after braces
                if peek_back(tokens, *pos).value == "}" && peek(tokens, *pos).value != ";" {
                    continue;
                }

                consume(tokens, pos, Expected::Semi);
            }
            Ast::Root {
                stats: statements,
                typ: Type::Unk,
                loc: location,
            }
        }
        Ast::Block {
            stats: _,
            typ: _,
            loc: block_location,
        } => {
            while peek(tokens, *pos).token_type != TokenType::End {
                statements.push(parse_line(tokens, pos));

                let mut semi = false;
                let token = peek(tokens, *pos);
                let location = token.location;
                if token.value.as_str() == ";" {
                    consume(tokens, pos, Expected::Semi);
                    semi = true;
                }

                let token = peek(tokens, *pos);
                if token.value.as_str() == "}" {
                    if semi {
                        statements.push(Ast::NoneLiteral {
                            typ: Type::Unk,
                            loc: location,
                        });
                    }
                    consume(tokens, pos, Expected::String(String::from("}")));
                    return Ast::Block {
                        stats: statements,
                        typ: Type::Unk,
                        loc: block_location,
                    };
                }

                // Don't require ';' after braces
                if !semi && peek_back(tokens, *pos).value != "}" {
                    panic!("{location:?}: expected ';'");
                }
            }
            panic!("Unexpected end of code, missing '}}'");
        }
        _ => panic!("Expected block"),
    }
}

fn parse_line(tokens: &Vec<Token>, pos: &mut usize) -> Ast {
    let token = peek(tokens, *pos);

    match token.value.as_str() {
        "var" => {
            let var_token = consume(tokens, pos, Expected::String(String::from("var")));
            let identifier = consume(tokens, pos, Expected::Token(TokenType::Identifier));
            let var_name = identifier.value.clone(); // Inoptimal?

            let mut typ = Type::Unk;
            if peek(tokens, *pos).value.as_str() == ":" {
                consume(tokens, pos, Expected::String(String::from(":")));
                let typ_token = consume(tokens, pos, Expected::Token(TokenType::Identifier));
                let location = typ_token.location;
                let typ_str = typ_token.value;

                typ = match typ_str.as_str() {
                    "Int" => Type::Int,
                    "Bool" => Type::Bool,
                    _ => panic!("{location:?}: Unknown type {typ_str}"),
                }
            }

            consume(tokens, pos, Expected::String(String::from("=")));
            let value = parse_expression(tokens, pos, 0);

            Ast::Var {
                name: var_name,
                value: Box::new(value),
                typ,
                loc: var_token.location,
            }
        }
        "while" => {
            let while_token = consume(tokens, pos, Expected::String(String::from("while")));
            let cond = parse_expression(tokens, pos, 0);
            consume(tokens, pos, Expected::String(String::from("do")));
            let brace_token = consume(tokens, pos, Expected::String(String::from("{")));
            let then = parse_block(
                tokens,
                pos,
                Ast::Block {
                    stats: vec![],
                    typ: Type::Unk,
                    loc: brace_token.location,
                },
            );
            Ast::While {
                cond: Box::new(cond),
                then: Box::new(then),
                typ: Type::Unk,
                loc: while_token.location,
            }
        }
        _ => parse_expression(tokens, pos, 0),
    }
}

// Sorted in precedence order
const BIN_OP: [&[&str]; 7] = [
    &[],                     // 0, Special case for "="
    &["or"],                 // 1
    &["and"],                // 2
    &["==", "!="],           // 3
    &["<", "<=", ">", ">="], // 4
    &["+", "-"],             // 5
    &["*", "/", "%"],        // 6
                             // 7, -, not
                             // 8, literal, identifier, if, block, parenthesis, function call
];

// Parse something that returns a value
fn parse_expression(tokens: &Vec<Token>, pos: &mut usize, level: usize) -> Ast {
    let mut left = if level < 7 {
        parse_expression(tokens, pos, level + 1)
    } else {
        parse_term(tokens, pos) // Contains levels 7 and 8
    };
    let mut token = peek(tokens, *pos);
    let location = token.location;

    // Special case for equality
    if token.value == *"=" && level == 0 {
        if !matches!(
            left,
            Ast::Identifier {
                name: _,
                typ: _,
                loc: _
            }
        ) {
            panic!("{location:?}: Expected variable")
        }

        let op = consume(tokens, pos, Expected::String(String::from("=")));

        // Allow same level on purpose because of right-associativity
        let right = parse_expression(tokens, pos, 0);

        return Ast::BinaryOp {
            left: Box::new(left),
            op: String::from("="),
            right: Box::new(right),
            typ: Type::Unk,
            loc: op.location,
        };
    }

    // Other binary ops
    for (op_lvl, ops) in BIN_OP.iter().enumerate().skip(level) {
        while ops.contains(&token.value.as_str()) {
            let op = consume(tokens, pos, Expected::None);
            let right = parse_expression(tokens, pos, op_lvl + 1);
            left = Ast::BinaryOp {
                left: Box::new(left),
                op: op.value,
                right: Box::new(right),
                typ: Type::Unk,
                loc: op.location,
            };

            token = peek(tokens, *pos);
        }
    }
    left
}

// Could be added to parse_expression
fn parse_term(tokens: &Vec<Token>, pos: &mut usize) -> Ast {
    let token = peek(tokens, *pos);
    let location = token.location;
    match token.value.as_str() {
        "if" => {
            let if_token = consume(tokens, pos, Expected::String(String::from("if")));
            let cond = parse_expression(tokens, pos, 0);
            consume(tokens, pos, Expected::String(String::from("then")));
            let then = parse_expression(tokens, pos, 0);

            let mut els = None;
            if peek(tokens, *pos).value == *"else" {
                consume(tokens, pos, Expected::String(String::from("else")));
                els = Some(parse_expression(tokens, pos, 0));
            }
            return Ast::If {
                cond: Box::new(cond),
                then: Box::new(then),
                els: Box::new(els),
                typ: Type::Unk,
                loc: if_token.location,
            };
        }
        "-" => {
            let token = consume(tokens, pos, Expected::String(String::from("-")));
            let expression = parse_term(tokens, pos);
            return Ast::Minus {
                stat: Box::new(expression),
                typ: Type::Unk,
                loc: token.location,
            };
        }
        "not" => {
            let token = consume(tokens, pos, Expected::String(String::from("not")));
            let expression = parse_term(tokens, pos);
            return Ast::Negate {
                stat: Box::new(expression),
                typ: Type::Unk,
                loc: token.location,
            };
        }
        "(" => {
            consume(tokens, pos, Expected::String(String::from("(")));
            let expression = parse_expression(tokens, pos, 0);
            consume(tokens, pos, Expected::String(String::from(")")));
            return expression;
        }
        "{" => {
            let token = consume(tokens, pos, Expected::String(String::from("{")));
            return parse_block(
                tokens,
                pos,
                Ast::Block {
                    stats: vec![],
                    typ: Type::Unk,
                    loc: token.location,
                },
            );
        }
        _ => {}
    }
    match token.token_type {
        TokenType::Integer => return parse_int_literal(tokens, pos),
        TokenType::Identifier => {
            let id_token = consume(tokens, pos, Expected::Token(TokenType::Identifier));
            let name = id_token.value.clone(); // Inoptimal?

            let identifier = match id_token.value.as_str() {
                "true" => Ast::BoolLiteral {
                    val: true,
                    typ: Type::Unk,
                    loc: id_token.location,
                },
                "false" => Ast::BoolLiteral {
                    val: false,
                    typ: Type::Unk,
                    loc: id_token.location,
                },
                _ => Ast::Identifier {
                    name: id_token.value,
                    typ: Type::Unk,
                    loc: id_token.location,
                },
            };

            // If boolean return;
            if matches!(
                identifier,
                Ast::BoolLiteral {
                    val: _,
                    typ: _,
                    loc: _
                }
            ) {
                return identifier;
            }

            // Check if function call or variable;
            let next_token = peek(tokens, *pos);
            if next_token.value != "(" {
                return identifier; // Variable
            };

            // Function call
            consume(tokens, pos, Expected::String(String::from("(")));
            let mut params = vec![];
            while peek(tokens, *pos).token_type != TokenType::End {
                if peek(tokens, *pos).value != ")" {
                    let expression = parse_expression(tokens, pos, 0);
                    params.push(expression);
                }
                if peek(tokens, *pos).value == ")" {
                    consume(tokens, pos, Expected::String(String::from(")")));
                    return Ast::Function {
                        name,
                        params,
                        typ: Type::Unk,
                        loc: id_token.location,
                    };
                }
                consume(tokens, pos, Expected::String(String::from(",")));
            }
            panic!("Missing ')' for function at {location:?}");
        }
        _ => {}
    }
    panic!("{location:?}: Expected expression");
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::tokenize;

    use super::*;

    // Macro rules to make creating asts manually easier
    // Root
    macro_rules! rast {
        () => (
            Ast::Root{stats: Vec::new(),typ: Type::Unk, loc: loc!()}
        );
        ($($x:expr),+ $(,)?) => (
            Ast::Root{stats: vec![$($x),+], typ: Type::Unk,loc: loc!()}
        );
    }
    // Block
    macro_rules! blast {
            () => (
                Ast::Block(Vec::new())
            );
            ($($x:expr),+ $(,)?) => (
                Ast::Block{stats: vec![$($x),+],typ: Type::Unk, loc: loc!()}
            );
        }
    // Function
    macro_rules! fast {
        () => (
            Ast::Function{params: Vec::new,typ: Type::Unk, loc: loc!()}
        );
        ($($x:expr),+ $(,)?) => (
            Ast::Function{name: String::from("f"), params: vec![$($x),+], typ: Type::Unk,loc: loc!()}
        );
    }
    // BinaryOp
    macro_rules! bast {
        ($left: expr, $op: expr, $right: expr) => {
            Ast::BinaryOp {
                left: Box::new($left),
                op: $op.to_owned(),
                right: Box::new($right),
                typ: Type::Unk,
                loc: loc!(),
            }
        };
    }
    // Integer
    macro_rules! iast {
        ($value: expr) => {
            Ast::IntLiteral {
                val: $value,
                typ: Type::Unk,
                loc: loc!(),
            }
        };
    }
    // None
    macro_rules! nast {
        () => {
            Ast::NoneLiteral {
                typ: Type::Unk,
                loc: loc!(),
            }
        };
    }
    // Identifier
    macro_rules! idast {
        ($name: expr) => {
            Ast::Identifier {
                name: $name.to_owned(),
                typ: Type::Unk,
                loc: loc!(),
            }
        };
    }
    // Boolean
    macro_rules! boast {
        ($value: expr) => {
            Ast::BoolLiteral {
                val: $value,
                typ: Type::Unk,
                loc: loc!(),
            }
        };
    }
    // Var
    macro_rules! vast {
        ($name: expr, $ast: expr) => {
            Ast::Var {
                name: $name.to_owned(),
                value: Box::new($ast),
                typ: Type::Unk,
                loc: loc!(),
            }
        };
    }
    // While
    macro_rules! wast {
        ($left: expr, $right: expr) => {
            Ast::While {
                cond: Box::new($left),
                then: Box::new($right),
                typ: Type::Unk,
                loc: loc!(),
            }
        };
    }
    // If
    macro_rules! ifast {
        ($cond: expr, $then: expr) => {
            Ast::If {
                cond: Box::new($cond),
                then: Box::new($then),
                els: Box::new(None),
                typ: Type::Unk,
                loc: loc!(),
            }
        };
        ($cond: expr, $then: expr, $els: expr) => {
            Ast::If {
                cond: Box::new($cond),
                then: Box::new($then),
                els: Box::new(Some($els)),
                typ: Type::Unk,
                loc: loc!(),
            }
        };
    }

    // Tests
    #[test]
    fn test_simple_addition() {
        let tokens = tokenize(String::from("1+1;"));
        let expected = rast![bast![iast!(1), "+", iast!(1)]];
        assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn test_simple_addition_with_identifiers() {
        let tokens = tokenize(String::from("x-50;"));
        let expected = rast![bast![idast!("x"), "-", iast!(50)]];
        assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn test_multiple_additions() {
        let tokens = tokenize(String::from("2-3+4;"));
        let expected = rast![bast![bast![iast!(2), "-", iast!(3)], "+", iast!(4)]];
        assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn test_multiplication() {
        let tokens = tokenize(String::from("1*2/3;"));
        let expected = rast![bast![bast![iast!(1), "*", iast!(2)], "/", iast!(3)]];
        assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn test_multiplication_and_addition() {
        let tokens = tokenize(String::from("1+2*3;"));
        let expected = rast![bast![iast!(1), "+", bast![iast!(2), "*", iast!(3)]]];
        assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn test_parenthesis() {
        let tokens = tokenize(String::from("(1+2)*3;"));
        let expected = rast![bast![bast![iast!(1), "+", iast!(2)], "*", iast!(3)]];
        assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn test_empty() {
        let tokens = tokenize(String::from(""));
        let expected = rast![];
        assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn test_var() {
        let tokens = tokenize(String::from("var x = 2;"));
        let expected = rast![vast!("x", iast!(2))];
        assert_eq!(parse(tokens), expected);
    }
    #[test]
    fn test_while() {
        let tokens = tokenize(String::from("while x do { var y = 2; };"));
        let expected = rast![wast!(idast!("x"), blast![vast!("y", iast!(2)), nast!()])];
        assert_eq!(parse(tokens), expected);
    }
    #[test]
    fn test_equality() {
        let tokens = tokenize(String::from("a = b = c;"));
        let expected = rast![bast![
            idast!("a"),
            "=",
            bast![idast!("b"), "=", idast!("c")]
        ]];
        assert_eq!(parse(tokens), expected);
    }
    #[test]
    fn test_function_call() {
        let tokens = tokenize(String::from("f(a, b, 1+c);"));
        let expected = rast![fast![
            idast!("a"),
            idast!("b"),
            bast![iast!(1), "+", idast!("c")]
        ]];
        assert_eq!(parse(tokens), expected);
    }
    #[test]
    fn test_returning_from_block() {
        let tokens = tokenize(String::from("a = {f(a); x=y; f(x)};"));
        let expected = rast![bast![
            idast!("a"),
            "=",
            blast![
                fast![idast!("a")],
                bast![idast!("x"), "=", idast!("y")],
                fast![idast!("x")]
            ]
        ]];
        assert_eq!(parse(tokens), expected);
    }
    #[test]
    fn test_if() {
        let tokens = tokenize(String::from("a = if b then c else d;"));
        let expected = rast![bast![
            idast!("a"),
            "=",
            ifast!(idast!("b"), idast!("c"), idast!("d"))
        ]];
        assert_eq!(parse(tokens), expected);
    }

    // Panicking tests
    #[test]
    #[should_panic]
    fn test_missing_number() {
        let tokens = tokenize(String::from("1+1+;"));
        parse(tokens);
    }

    #[test]
    #[should_panic]
    fn test_extra_number() {
        let tokens = tokenize(String::from("1+1 1;"));
        parse(tokens);
    }

    #[test]
    fn test_binary_ops() {
        let tokens = tokenize(String::from("n / 2 == 0;"));
        let expected = rast![bast!(bast!(idast!("n"), "/", iast!(2)), "==", iast!(0))];
        assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn test_binary_ops_in_if() {
        let tokens = tokenize(String::from("if n / 2 == 0 then 1;"));
        let expected = rast![ifast!(
            bast!(bast!(idast!("n"), "/", iast!(2)), "==", iast!(0)),
            iast!(1)
        )];
        assert_eq!(parse(tokens), expected);
    }

    mod block_tests {
        use crate::tokenizer::tokenize;

        use super::*;

        #[test]
        fn test_blocks() {
            let tokens = tokenize(String::from("{ { a } { b } }"));
            let expected = rast![blast![blast![idast!("a")], blast![idast!("b")]]];
            assert_eq!(parse(tokens), expected);
        }
        #[test]
        fn test_blocks_2() {
            let tokens = tokenize(String::from("{ if true then { a } b }"));
            let expected = rast![blast![
                ifast!(boast!(true), blast![idast!("a")]),
                idast!("b")
            ]];
            assert_eq!(parse(tokens), expected);
        }
        #[test]
        fn test_blocks_3() {
            let tokens = tokenize(String::from("{ if true then { a }; b }"));
            let expected = rast![blast![
                ifast!(boast!(true), blast![idast!("a")]),
                idast!("b")
            ]];
            assert_eq!(parse(tokens), expected);
        }
        #[test]
        fn test_blocks_4() {
            let tokens = tokenize(String::from("{ if true then { a }; b; c }"));
            let expected = rast![blast![
                ifast!(boast!(true), blast![idast!("a")]),
                idast!("b"),
                idast!("c")
            ]];
            assert_eq!(parse(tokens), expected);
        }
        #[test]
        fn test_blocks_5() {
            let tokens = tokenize(String::from("{ if true then { a } else { b } 3 }"));
            let expected = rast![blast![
                ifast!(boast!(true), blast![idast!("a")], blast![idast!("b")]),
                iast!(3)
            ]];
            assert_eq!(parse(tokens), expected);
        }
        #[test]
        fn test_blocks_6() {
            let tokens = tokenize(String::from("x = { { f(a) } { b } }"));
            let expected = rast![bast!(
                idast!("x"),
                "=",
                blast![blast![fast![idast!("a")]], blast![idast!("b")]]
            )];
            assert_eq!(parse(tokens), expected);
        }

        #[test]
        #[should_panic]
        fn test_panic_blocks() {
            let tokens = tokenize(String::from("{ a b }"));
            parse(tokens);
        }
        #[test]
        #[should_panic]
        fn test_panic_blocks_2() {
            let tokens = tokenize(String::from("{ if true then { a } b c }"));
            parse(tokens);
        }
    }
}
