use crate::{parser::Ast, util::ast_loc};

#[derive(Debug, PartialEq)]
pub enum Value {
    None,
    Int(i64),
    Bool(bool),
}

pub fn interpret(node: &Ast) -> Value {
    #[expect(unused_variables)]
    match node {
        Ast::NoneLiteral { loc } => Value::None,
        Ast::IntLiteral { val, loc } => Value::Int(*val),
        Ast::BoolLiteral { val, loc } => Value::Bool(*val),
        Ast::Minus { stat, loc } => Value::Int(-interpret_int(stat)),
        Ast::Negate { stat, loc } => Value::Bool(!interpret_bool(stat)),
        Ast::Root { stats, loc } => interpret(&Ast::Block {
            stats: stats.to_vec(), // Inoptimal, too lazy to make duplicate code
            loc: *loc,
        }), // Identical code

        Ast::Function { params, loc } => todo!(),

        Ast::Var { stat, loc } => todo!(),
        Ast::Identifier { name, loc } => todo!(),

        Ast::If {
            cond,
            then,
            els,
            loc,
        } => {
            let cond_val = match interpret(cond) {
                Value::Bool(val) => val,
                _ => panic!("{loc:?}: Expected boolean"),
            };
            if cond_val {
                return interpret(then);
            }
            match &**els {
                Some(node) => interpret(node),
                None => Value::None,
            }
        }

        Ast::While { cond, then, loc } => {
            while interpret_bool(cond) {
                interpret(then);
            }
            Value::None
        }

        Ast::Block { stats, loc: _ } => {
            let len = &stats.len();
            for (i, stat) in stats.iter().enumerate() {
                if i == len - 1 {
                    return interpret(stat);
                }
                interpret(stat);
            }
            Value::None // Empty block
        }

        Ast::BinaryOp {
            left,
            op,
            right,
            loc,
        } => match op.as_str() {
            "=" => todo!(),
            "or" => Value::Bool(interpret_bool(left) || interpret_bool(right)),
            "and" => Value::Bool(interpret_bool(left) && interpret_bool(right)),
            "==" => Value::Bool(interpret_value(left) == interpret_value(right)),
            "!=" => Value::Bool(interpret_value(left) != interpret_value(right)),
            "<" => Value::Bool(interpret_int(left) < interpret_int(right)),
            "<=" => Value::Bool(interpret_int(left) <= interpret_int(right)),
            ">" => Value::Bool(interpret_int(left) > interpret_int(right)),
            ">=" => Value::Bool(interpret_int(left) >= interpret_int(right)),
            "+" => Value::Int(interpret_int(left) + interpret_int(right)),
            "-" => Value::Int(interpret_int(left) - interpret_int(right)),
            "*" => Value::Int(interpret_int(left) * interpret_int(right)),
            "/" => Value::Int(interpret_int(left) / interpret_int(right)),
            "%" => Value::Int(interpret_int(left) % interpret_int(right)),
            _ => panic!("{loc:?}: Invalid operator {op}"),
        },
    }
}

fn interpret_bool(node: &Ast) -> bool {
    let loc = ast_loc(node);
    match interpret(node) {
        Value::Bool(val) => val,
        _ => panic!("{loc:?}: Expected boolean"),
    }
}

fn interpret_int(node: &Ast) -> i64 {
    let loc = ast_loc(node);
    match interpret(node) {
        Value::Int(val) => val,
        _ => panic!("{loc:?}: Expected integer"),
    }
}

fn interpret_value(node: &Ast) -> Value {
    let loc = ast_loc(node);
    let value = interpret(node);
    if matches!(value, Value::None) {
        panic!("{loc:?}: Expected boolean or integer");
    }
    value
}

// case ast.Literal():
//     return node.value

// case ast.BinaryOp():
//     a: Any = interpret(node.left)
//     b: Any = interpret(node.right)
//     if node.op == '+':
//         return a + b
//     elif node.op == '<':
//         return a < b
//     else:
//         raise ...

// case ast.IfThenElse():
//     if interpret(node.condition):
//         return interpret(node.then_branch)
//     else:
//         return interpret(node.else_branch)
// ...}
