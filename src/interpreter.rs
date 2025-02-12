use std::{collections::HashMap, io};

use crate::{parser::Ast, util::ast_loc};

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    None,
    Int(i64),
    Bool(bool),
}

pub fn run_interpret(ast: &Ast) -> Value {
    let mut variables: Vec<HashMap<String, Value>> = Vec::new();
    interpret(ast, &mut variables)
}

fn interpret(node: &Ast, variables: &mut Vec<HashMap<String, Value>>) -> Value {
    match node {
        Ast::NoneLiteral { .. } => Value::None,
        // Fails with integers equal to i64::MIN
        Ast::IntLiteral { val, .. } => Value::Int((*val).try_into().unwrap()),
        Ast::BoolLiteral { val, .. } => Value::Bool(*val),
        Ast::Minus { stat, .. } => Value::Int(-interpret_int(stat, variables)),
        Ast::Negate { stat, .. } => Value::Bool(!interpret_bool(stat, variables)),
        Ast::Root { stats, typ, loc } => interpret(
            &Ast::Block {
                stats: stats.clone(), // Inoptimal, too lazy to make duplicate code
                typ: typ.clone(),
                loc: *loc,
            },
            variables,
        ), // Identical code

        Ast::Function {
            name, params, loc, ..
        } => match name.as_str() {
            "print_int" => {
                assert!(params.len() == 1);
                let value = interpret_int(&params[0], variables);
                println!("{value}");
                Value::None
            }
            "print_bool" => {
                assert!(params.len() == 1);
                let value = interpret_bool(&params[0], variables);
                println!("{value}");
                Value::None
            }
            "read_int" => {
                let mut input_line = String::new();
                io::stdin()
                    .read_line(&mut input_line)
                    .expect("Failed to read line");
                Value::Int(
                    input_line
                        .trim()
                        .parse::<i64>()
                        .expect("Input not an integer"),
                )
            }
            _ => panic!("{loc:?}: Unknown function \"{name}\""),
        },

        Ast::Var { name, value, .. } => {
            let value = interpret_val(value, variables);
            variables.last_mut().unwrap().insert(name.clone(), value);
            Value::None
        }

        Ast::Identifier { name, loc, .. } => {
            for map in variables.iter().rev() {
                if map.contains_key(name) {
                    return map.get(name).unwrap().clone();
                }
            }
            panic!("{loc:?}: Unknown variable \"{name}\"");
        }

        Ast::If {
            cond,
            then,
            els,
            loc,
            ..
        } => {
            let cond_val = match interpret(cond, variables) {
                Value::Bool(val) => val,
                _ => panic!("{loc:?}: Expected boolean"),
            };
            if cond_val {
                return interpret(then, variables);
            }
            match &**els {
                Some(node) => interpret(node, variables),
                None => Value::None,
            }
        }

        Ast::While { cond, then, .. } => {
            while interpret_bool(cond, variables) {
                interpret(then, variables);
            }
            Value::None
        }

        Ast::Block { stats, .. } => {
            variables.push(HashMap::new());
            let mut ret_value = Value::None;
            for stat in stats {
                ret_value = interpret(stat, variables);
            }
            variables.pop();
            ret_value
        }

        Ast::BinaryOp {
            left,
            op,
            right,
            loc,
            ..
        } => match op.as_str() {
            "=" => {
                let name = match &**left {
                    Ast::Identifier { name, .. } => name,
                    _ => panic!("{loc:?}: Expected left side of = to be an identifier"),
                };
                let value = interpret_val(right, variables);
                for map in variables.iter_mut().rev() {
                    if map.contains_key(name) {
                        map.insert(name.clone(), value.clone());
                        return value;
                    }
                }
                panic!("{loc:?}: Unknown variable \"{name}\"");
            }
            "or" => {
                Value::Bool(interpret_bool(left, variables) || interpret_bool(right, variables))
            }
            "and" => {
                Value::Bool(interpret_bool(left, variables) && interpret_bool(right, variables))
            }
            "==" => Value::Bool(interpret_val(left, variables) == interpret_val(right, variables)),
            "!=" => Value::Bool(interpret_val(left, variables) != interpret_val(right, variables)),
            "<" => Value::Bool(interpret_int(left, variables) < interpret_int(right, variables)),
            "<=" => Value::Bool(interpret_int(left, variables) <= interpret_int(right, variables)),
            ">" => Value::Bool(interpret_int(left, variables) > interpret_int(right, variables)),
            ">=" => Value::Bool(interpret_int(left, variables) >= interpret_int(right, variables)),
            "+" => Value::Int(interpret_int(left, variables) + interpret_int(right, variables)),
            "-" => Value::Int(interpret_int(left, variables) - interpret_int(right, variables)),
            "*" => Value::Int(interpret_int(left, variables) * interpret_int(right, variables)),
            "/" => Value::Int(interpret_int(left, variables) / interpret_int(right, variables)),
            "%" => Value::Int(interpret_int(left, variables) % interpret_int(right, variables)),
            _ => panic!("{loc:?}: Invalid operator {op}"),
        },
    }
}

fn interpret_bool(node: &Ast, variables: &mut Vec<HashMap<String, Value>>) -> bool {
    let loc = ast_loc(node);
    match interpret(node, variables) {
        Value::Bool(val) => val,
        _ => panic!("{loc:?}: Expected boolean"),
    }
}

fn interpret_int(node: &Ast, variables: &mut Vec<HashMap<String, Value>>) -> i64 {
    let loc = ast_loc(node);
    match interpret(node, variables) {
        Value::Int(val) => val,
        _ => panic!("{loc:?}: Expected integer"),
    }
}

fn interpret_val(node: &Ast, variables: &mut Vec<HashMap<String, Value>>) -> Value {
    let loc = ast_loc(node);
    let value = interpret(node, variables);
    if matches!(value, Value::None) {
        panic!("{loc:?}: Expected boolean or integer");
    }
    value
}
