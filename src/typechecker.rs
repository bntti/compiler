use std::collections::HashMap;

use crate::{
    parser::{Ast, Type},
    util::ast_loc,
};

pub fn run_typecheck(ast: &mut Ast) {
    let bool_bin_fn = Type::Function {
        params: vec![Type::Bool, Type::Bool],
        ret: Box::new(Type::Bool),
    };
    let int_bool_bin_fn = Type::Function {
        params: vec![Type::Int, Type::Int],
        ret: Box::new(Type::Bool),
    };
    let int_bin_fn = Type::Function {
        params: vec![Type::Int, Type::Int],
        ret: Box::new(Type::Int),
    };

    let mut globals: HashMap<String, Type> = HashMap::from([
        (String::from("or"), bool_bin_fn.clone()),
        (String::from("and"), bool_bin_fn.clone()),
        (String::from("<"), int_bool_bin_fn.clone()),
        (String::from("<="), int_bool_bin_fn.clone()),
        (String::from(">"), int_bool_bin_fn.clone()),
        (String::from(">="), int_bool_bin_fn.clone()),
        (String::from("+"), int_bin_fn.clone()),
        (String::from("-"), int_bin_fn.clone()),
        (String::from("*"), int_bin_fn.clone()),
        (String::from("/"), int_bin_fn.clone()),
        (String::from("%"), int_bin_fn.clone()),
        (
            String::from("print_int"),
            Type::Function {
                params: vec![Type::Int],
                ret: Box::new(Type::Unit),
            },
        ),
        (
            String::from("print_bool"),
            Type::Function {
                params: vec![Type::Bool],
                ret: Box::new(Type::Unit),
            },
        ),
        (
            String::from("read_int"),
            Type::Function {
                params: vec![],
                ret: Box::new(Type::Int),
            },
        ),
    ]);

    // Add functions to globals
    let Ast::Module { stats, .. } = ast else {
        unreachable!()
    };
    for stat in stats {
        if let Ast::Fn {
            name,
            param_types,
            typ,
            ..
        } = stat
        {
            globals.insert(
                name.clone(),
                Type::Function {
                    params: param_types.iter_mut().map(|p| p.1.clone()).collect(),
                    ret: Box::new(typ.clone()),
                },
            );
        }
    }

    let mut variables: Vec<HashMap<String, Type>> = vec![globals];
    variables.push(HashMap::new());

    typecheck(ast, &mut variables, &None);
}

fn typecheck(
    node: &mut Ast,
    variables: &mut Vec<HashMap<String, Type>>,
    ret_type: &Option<Type>,
) -> Type {
    match node {
        Ast::NoneLiteral { typ, .. } => {
            *typ = Type::Unit;
            Type::Unit
        }
        Ast::IntLiteral { typ, .. } => {
            *typ = Type::Int;
            Type::Int
        }
        Ast::BoolLiteral { typ, .. } => {
            *typ = Type::Bool;
            Type::Bool
        }
        Ast::UnaryMinus { stat, typ, .. } => {
            typecheck_int(stat, variables, ret_type);
            *typ = Type::Int;
            Type::Int
        }
        Ast::UnaryNot { stat, typ, .. } => {
            typecheck_bool(stat, variables, ret_type);
            *typ = Type::Bool;
            Type::Bool
        }
        Ast::Continue { typ, .. } => {
            *typ = Type::Unit;
            Type::Unit
        }
        Ast::Break { typ, .. } => {
            *typ = Type::Unit;
            Type::Unit
        }

        Ast::Fn {
            param_types,
            block,
            typ,
            ..
        } => {
            variables.push(HashMap::new());
            for param in param_types {
                variables
                    .last_mut()
                    .unwrap()
                    .insert(param.0.clone(), param.1.clone());
            }
            typecheck(block, variables, &Some(typ.clone()));
            variables.pop();

            Type::Unit
        }

        Ast::Return { stat, typ, loc } => match ret_type {
            Some(expected) => {
                let ret_type = typecheck(stat, variables, ret_type);
                if ret_type != *expected {
                    panic!("{loc:?}: Expected {typ:?}")
                };
                Type::Unit
            }
            None => panic!("{loc:?}: Cannot return outside of a function"),
        },

        Ast::FnCall {
            name,
            params,
            typ,
            loc,
        } => {
            let mut fn_type = Type::Unk;
            for map in variables.iter().rev() {
                if map.contains_key(name) {
                    fn_type = map[name].clone();
                    break;
                }
            }
            match fn_type {
                Type::Unk => panic!("{loc:?}: Unknown function \"{name}\""),
                Type::Function {
                    params: param_types,
                    ret,
                } => {
                    if params.len() != params.len() {
                        panic!("{loc:?}: Wrong number of parameters for \"{name}\"");
                    }
                    for i in 0..params.len() {
                        let location = ast_loc(&params[i]);
                        let expected = param_types[i].clone();
                        let param_type = typecheck(&mut params[i], variables, ret_type);
                        if param_type != expected {
                            panic!("{location:?}: Wrong parameter type, expected {expected:?}");
                        }
                    }

                    *typ = *ret.clone();
                    *ret
                }
                _ => panic!("{loc:?}: Not a function \"{name}\""),
            }
        }

        Ast::Var {
            name,
            value,
            typ,
            loc,
        } => {
            let value_type = typecheck_val(value, variables, ret_type);
            if !matches!(typ, Type::Unk) && value_type != *typ {
                panic!("{loc:?}: Wrong type, expected {typ:?}");
            }

            if variables.last_mut().unwrap().contains_key(name) {
                panic!("{loc:?}: Variable {name} already in scope");
            }

            variables
                .last_mut()
                .unwrap()
                .insert(name.clone(), value_type.clone());

            *typ = value_type.clone();
            value_type
        }

        // TODO: Check that identifier is not a function?
        Ast::Identifier { name, typ, loc } => {
            for map in variables.iter().rev() {
                if map.contains_key(name) {
                    let identifier_type = map.get(name).unwrap().clone();
                    *typ = identifier_type.clone();
                    return identifier_type;
                }
            }
            panic!("{loc:?}: Unknown variable \"{name}\"");
        }

        Ast::If {
            cond,
            then,
            els,
            loc,
            typ,
        } => {
            typecheck_bool(cond, variables, ret_type);

            let then_type = typecheck(then, variables, ret_type);

            match &mut **els {
                Some(node) => {
                    let else_type = typecheck(&mut *node, variables, ret_type);
                    if then_type != else_type {
                        panic!("{loc:?}: Mismatching types for then and else branches");
                    }
                    *typ = then_type.clone();
                    then_type
                }
                None => {
                    *typ = Type::Unit;
                    Type::Unit
                }
            }
        }

        Ast::While {
            cond, then, typ, ..
        } => {
            typecheck_bool(cond, variables, ret_type);
            typecheck(then, variables, ret_type);
            *typ = Type::Unit;
            Type::Unit
        }

        // Identical to block code
        Ast::Module { stats, typ, .. } => {
            variables.push(HashMap::new());
            let mut block_type = Type::Unit;
            for stat in stats {
                block_type = typecheck(&mut *stat, variables, ret_type);
            }
            variables.pop();
            *typ = block_type.clone();
            block_type
        }
        Ast::Block { stats, typ, .. } => {
            variables.push(HashMap::new());
            let mut block_type = Type::Unit;
            for stat in stats {
                block_type = typecheck(&mut *stat, variables, ret_type);
            }
            variables.pop();
            *typ = block_type.clone();
            block_type
        }

        Ast::BinaryOp {
            left,
            op,
            right,
            typ,
            loc,
        } => match op.as_str() {
            "=" => {
                if !matches!(&**left, Ast::Identifier { .. }) {
                    panic!("{loc:?}: Expected left side of = to be an identifier");
                }

                let left_type = typecheck(left, variables, ret_type);
                let value_type = typecheck_val(right, variables, ret_type);

                if left_type != value_type {
                    panic!("{loc:?}: Invalid type, expected {left_type:?}");
                }
                *typ = value_type.clone();
                value_type
            }
            "==" => {
                let left_type = typecheck(left, variables, ret_type);
                let right_type = typecheck(right, variables, ret_type);
                if left_type != right_type {
                    panic!("{loc:?}: Mismatching types");
                }
                *typ = Type::Bool;
                Type::Bool
            }
            "!=" => {
                let left_type = typecheck(left, variables, ret_type);
                let right_type = typecheck(right, variables, ret_type);
                if left_type != right_type {
                    panic!("{loc:?}: Mismatching types");
                }
                *typ = Type::Bool;
                Type::Bool
            }

            op_str => {
                let mut fn_type = Type::Unk;
                if variables[0].contains_key(op_str) {
                    fn_type = variables[0][op_str].clone();
                }

                match fn_type {
                    Type::Function {
                        params: param_types,
                        ret,
                    } => {
                        let left_type = typecheck(left, variables, ret_type);
                        let right_type = typecheck(right, variables, ret_type);
                        let left_loc = ast_loc(left);
                        let right_loc = ast_loc(right);

                        if left_type != param_types[0] {
                            panic!("{left_loc:?}: Invalid type for left operand of {op_str}");
                        }

                        if right_type != param_types[1] {
                            panic!("{right_loc:?}: Invalid type for right operand of {op_str}");
                        }

                        *typ = *ret.clone();
                        *ret
                    }
                    _ => unreachable!(),
                }
            }
        },
    }
}

fn typecheck_bool(
    node: &mut Ast,
    variables: &mut Vec<HashMap<String, Type>>,
    ret_type: &Option<Type>,
) -> Type {
    let loc = ast_loc(node);
    match typecheck(node, variables, ret_type) {
        Type::Bool => Type::Bool,
        _ => panic!("{loc:?}: Expected boolean"),
    }
}

fn typecheck_int(
    node: &mut Ast,
    variables: &mut Vec<HashMap<String, Type>>,
    ret_type: &Option<Type>,
) -> Type {
    let loc = ast_loc(node);
    match typecheck(node, variables, ret_type) {
        Type::Int => Type::Int,
        _ => panic!("{loc:?}: Expected integer"),
    }
}

fn typecheck_val(
    node: &mut Ast,
    variables: &mut Vec<HashMap<String, Type>>,
    ret_type: &Option<Type>,
) -> Type {
    let loc = ast_loc(node);
    match typecheck(node, variables, ret_type) {
        Type::Bool => Type::Bool,
        Type::Int => Type::Int,
        _ => panic!("{loc:?}: Expected integer"),
    }
}
