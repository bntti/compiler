use std::{collections::HashMap, fmt::Display};

use crate::{
    loc,
    parser::{Ast, Type},
    util::Location,
};

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct IRVar(pub String);

#[derive(Debug, Clone)]
#[expect(dead_code)] // Some loc fields
pub enum Instruction {
    LoadBoolConst {
        loc: Location,
        value: bool,
        dest: IRVar,
    },
    LoadIntConst {
        loc: Location,
        value: i64,
        dest: IRVar,
    },
    Copy {
        loc: Location,
        source: IRVar,
        dest: IRVar,
    },
    Call {
        loc: Location,
        fun: IRVar,
        args: Vec<IRVar>,
        dest: IRVar,
    },
    Jump {
        loc: Location,
        label: Box<Instruction>,
    },
    CondJump {
        loc: Location,
        cond: IRVar,
        then_label: Box<Instruction>,
        else_label: Box<Instruction>,
    },
    Label {
        loc: Location,
        name: String,
    },
}

impl Display for IRVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = &self.0;
        write!(f, "{name}")
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::LoadBoolConst { value, dest, .. } => {
                write!(f, "LoadBoolConst({value}, {dest})")
            }
            Instruction::LoadIntConst { value, dest, .. } => {
                write!(f, "LoadIntConst({value}, {dest})")
            }
            Instruction::Copy { source, dest, .. } => {
                write!(f, "Copy({source}, {dest})")
            }
            Instruction::Call {
                fun, args, dest, ..
            } => {
                let args = args
                    .iter()
                    .map(|arg| format!("{arg}"))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "Call({fun}, [{args}], {dest})")
            }
            Instruction::Jump { label, .. } => {
                write!(f, "Jump({label})")
            }
            Instruction::CondJump {
                cond,
                then_label,
                else_label,
                ..
            } => {
                write!(f, "CondJump({cond}, {then_label}, {else_label})")
            }
            Instruction::Label { name, .. } => {
                write!(f, "Label({name})")
            }
        }
    }
}

pub fn run_ir_gen(root_ast: Ast) -> Vec<Instruction> {
    // The type is actually not Unit, but we do not need the type for now
    let root_types = HashMap::from([
        (IRVar(String::from("<")), Type::Unit),
        (IRVar(String::from("<=")), Type::Unit),
        (IRVar(String::from(">")), Type::Unit),
        (IRVar(String::from(">=")), Type::Unit),
        (IRVar(String::from("==")), Type::Unit),
        (IRVar(String::from("!=")), Type::Unit),
        (IRVar(String::from("+")), Type::Unit),
        (IRVar(String::from("-")), Type::Unit),
        (IRVar(String::from("*")), Type::Unit),
        (IRVar(String::from("/")), Type::Unit),
        (IRVar(String::from("%")), Type::Unit),
        (IRVar(String::from("unary_minus")), Type::Unit),
        (IRVar(String::from("unary_not")), Type::Unit),
        (IRVar(String::from("read_int")), Type::Unit),
        (IRVar(String::from("print_int")), Type::Unit),
        (IRVar(String::from("print_bool")), Type::Unit),
    ]);
    generate_ir(root_types, root_ast)
}

fn generate_ir(root_types: HashMap<IRVar, Type>, root_ast: Ast) -> Vec<Instruction> {
    let mut var_types = root_types.clone();
    let var_unit = IRVar(String::from("unit"));
    var_types.insert(var_unit.clone(), Type::Unit);

    let mut free = 0;
    let mut ins = vec![];
    let mut root_symbol_table = vec![HashMap::new()];
    for (v, _) in root_types.iter() {
        root_symbol_table[0].insert(v.0.clone(), v.clone());
    }

    ins.push(Instruction::Label {
        loc: loc!(0, 0),
        name: String::from("start"),
    });
    let var_final_result = visit(
        &mut ins,
        &mut var_types,
        &mut root_symbol_table,
        &mut free,
        &root_ast,
    );

    // Print return value of root block if not unit
    match var_types[&var_final_result] {
        Type::Bool => {
            let fun = root_symbol_table[0][&String::from("print_bool")].clone();
            ins.push(Instruction::Call {
                loc: loc!(usize::MAX, usize::MAX),
                fun,
                args: vec![var_final_result],
                dest: var_unit,
            });
        }
        Type::Int => {
            let fun = root_symbol_table[0][&String::from("print_int")].clone();
            ins.push(Instruction::Call {
                loc: loc!(usize::MAX, usize::MAX),
                fun,
                args: vec![var_final_result],
                dest: var_unit,
            });
        }
        _ => (),
    }

    ins
}

fn new_var(var_types: &mut HashMap<IRVar, Type>, free: &mut usize, t: Type) -> IRVar {
    if matches!(t, Type::Unit) {
        return IRVar(String::from("unit"));
    }

    let var = IRVar(String::from("x") + &free.to_string());
    *free += 1;
    var_types.insert(var.clone(), t);
    var
}

fn new_label(loc: &Location, free: &mut usize) -> Instruction {
    let label = Instruction::Label {
        loc: *loc,
        name: String::from("L") + &free.to_string(),
    };
    *free += 1;

    label
}

fn visit(
    ins: &mut Vec<Instruction>,
    var_types: &mut HashMap<IRVar, Type>,
    sym_table: &mut Vec<HashMap<String, IRVar>>,
    free: &mut usize,
    ast: &Ast,
) -> IRVar {
    match ast {
        Ast::NoneLiteral { .. } => IRVar(String::from("unit")),
        Ast::BoolLiteral { val, loc, .. } => {
            let var = new_var(var_types, free, Type::Bool);
            ins.push(Instruction::LoadBoolConst {
                loc: *loc,
                value: *val,
                dest: var.clone(),
            });
            var
        }
        Ast::IntLiteral { val, loc, .. } => {
            let var = new_var(var_types, free, Type::Int);
            ins.push(Instruction::LoadIntConst {
                loc: *loc,
                value: (*val)
                    .try_into()
                    .unwrap_or_else(|_| panic!("{loc:?}: Invalid integer")),
                dest: var.clone(),
            });
            var
        }
        Ast::Identifier { name, .. } => {
            for map in sym_table.iter().rev() {
                if map.contains_key(name) {
                    return map[name].clone();
                }
            }
            unreachable!();
        }
        Ast::BinaryOp {
            left,
            op,
            right,
            typ,
            loc,
        } => {
            match op.as_str() {
                "=" => {
                    let var_left = visit(ins, var_types, sym_table, free, left);
                    let var_right = visit(ins, var_types, sym_table, free, right);
                    ins.push(Instruction::Copy {
                        loc: *loc,
                        source: var_right.clone(),
                        dest: var_left,
                    });
                    var_right
                }
                "or" => {
                    let short_circuit_label = new_label(loc, free);
                    let other_label = new_label(loc, free);
                    let end_label = new_label(loc, free);
                    let result = new_var(var_types, free, Type::Bool);

                    let var_left = visit(ins, var_types, sym_table, free, left);
                    ins.push(Instruction::CondJump {
                        loc: *loc,
                        cond: var_left,
                        then_label: Box::new(short_circuit_label.clone()),
                        else_label: Box::new(other_label.clone()),
                    });

                    // Short circuit
                    ins.push(short_circuit_label);
                    ins.push(Instruction::LoadBoolConst {
                        loc: *loc,
                        value: true,
                        dest: result.clone(),
                    });
                    ins.push(Instruction::Jump {
                        loc: *loc,
                        label: Box::new(end_label.clone()),
                    });

                    // No short circuit; parse other term.
                    ins.push(other_label);
                    let var_right = visit(ins, var_types, sym_table, free, right);
                    ins.push(Instruction::Copy {
                        loc: *loc,
                        source: var_right,
                        dest: result.clone(),
                    });

                    ins.push(end_label);

                    result
                }
                "and" => {
                    let short_circuit_label = new_label(loc, free);
                    let other_label = new_label(loc, free);
                    let end_label = new_label(loc, free);
                    let result = new_var(var_types, free, Type::Bool);

                    let var_left = visit(ins, var_types, sym_table, free, left);
                    ins.push(Instruction::CondJump {
                        loc: *loc,
                        cond: var_left,
                        then_label: Box::new(other_label.clone()),
                        else_label: Box::new(short_circuit_label.clone()),
                    });

                    // Short circuit
                    ins.push(short_circuit_label);
                    ins.push(Instruction::LoadBoolConst {
                        loc: *loc,
                        value: false,
                        dest: result.clone(),
                    });
                    ins.push(Instruction::Jump {
                        loc: *loc,
                        label: Box::new(end_label.clone()),
                    });

                    // No short circuit; parse other term.
                    ins.push(other_label);
                    let var_right = visit(ins, var_types, sym_table, free, right);
                    ins.push(Instruction::Copy {
                        loc: *loc,
                        source: var_right,
                        dest: result.clone(),
                    });

                    ins.push(end_label);

                    result
                }
                _ => {
                    let var_op = sym_table[0][op].clone();
                    let var_left = visit(ins, var_types, sym_table, free, left);
                    let var_right = visit(ins, var_types, sym_table, free, right);
                    let var_result = new_var(var_types, free, typ.clone());
                    ins.push(Instruction::Call {
                        loc: *loc,
                        fun: var_op,
                        args: vec![var_left, var_right],
                        dest: var_result.clone(),
                    });
                    var_result
                }
            }
        }
        Ast::Minus { stat, typ, loc } => {
            // Special case for negative integers because -i64::MIN > i64::MAX
            if let Ast::IntLiteral { val, loc, .. } = &**stat {
                // If val == -i64::MIN
                let signed_val = if *val == 9_223_372_036_854_775_808_u64 {
                    i64::MIN
                } else {
                    -<u64 as std::convert::TryInto<i64>>::try_into(*val)
                        .unwrap_or_else(|_| panic!("{loc:?}: Invalid integer"))
                };

                let var = new_var(var_types, free, Type::Int);
                ins.push(Instruction::LoadIntConst {
                    loc: *loc,
                    value: signed_val,
                    dest: var.clone(),
                });
                return var;
            }

            let var_op = sym_table[0][&String::from("unary_minus")].clone();
            let var_value = visit(ins, var_types, sym_table, free, stat);
            let var_result = new_var(var_types, free, typ.clone());
            ins.push(Instruction::Call {
                loc: *loc,
                fun: var_op,
                args: vec![var_value],
                dest: var_result.clone(),
            });
            var_result
        }
        Ast::Negate { stat, typ, loc } => {
            let var_op = sym_table[0][&String::from("unary_not")].clone();
            let var_value = visit(ins, var_types, sym_table, free, stat);
            let var_result = new_var(var_types, free, typ.clone());
            ins.push(Instruction::Call {
                loc: *loc,
                fun: var_op,
                args: vec![var_value],
                dest: var_result.clone(),
            });
            var_result
        }

        Ast::Root { stats, .. } => {
            sym_table.push(HashMap::new());
            let mut block_var = IRVar(String::from("unit"));
            for stat in stats {
                block_var = visit(ins, var_types, sym_table, free, stat);
            }
            sym_table.pop();
            block_var
        }
        Ast::Block { stats, .. } => {
            sym_table.push(HashMap::new());
            let mut block_var = IRVar(String::from("unit"));
            for stat in stats {
                block_var = visit(ins, var_types, sym_table, free, stat);
            }
            sym_table.pop();
            block_var
        }

        Ast::Var {
            name,
            value: stat,
            typ,
            loc,
        } => {
            // Would be nice to keep the name, but duplicate names would be a problem :(
            let new_var = new_var(var_types, free, typ.clone());
            var_types.insert(new_var.clone(), typ.clone());
            sym_table
                .last_mut()
                .unwrap()
                .insert(name.clone(), new_var.clone());

            let value = visit(ins, var_types, sym_table, free, stat);
            ins.push(Instruction::Copy {
                loc: *loc,
                source: value,
                dest: new_var,
            });

            IRVar(String::from("unit"))
        }

        Ast::Function {
            name,
            params,
            typ,
            loc,
        } => {
            let mut params_vars = vec![];
            for param in params.iter() {
                params_vars.push(visit(ins, var_types, sym_table, free, param));
            }

            let mut fun = None;
            for map in sym_table.iter().rev() {
                if map.contains_key(name) {
                    fun = Some(map[name].clone());
                }
            }

            let output = new_var(var_types, free, typ.clone());
            ins.push(Instruction::Call {
                loc: *loc,
                fun: fun.unwrap(),
                args: params_vars,
                dest: output.clone(),
            });

            output
        }

        Ast::If {
            cond,
            then,
            els,
            typ,
            loc,
        } => {
            let then_label = new_label(loc, free);
            let else_label = new_label(loc, free);

            let cond_var = visit(ins, var_types, sym_table, free, cond);
            ins.push(Instruction::CondJump {
                loc: *loc,
                cond: cond_var,
                then_label: Box::new(then_label.clone()),
                else_label: Box::new(else_label.clone()),
            });

            match &**els {
                None => {
                    ins.push(then_label);
                    visit(ins, var_types, sym_table, free, then);

                    ins.push(else_label); // Use else label as end label.

                    IRVar(String::from("unit"))
                }
                Some(els) => {
                    let end_label = new_label(loc, free); // Create separate end label
                    let result = new_var(var_types, free, typ.clone());

                    ins.push(then_label);
                    let then_value = visit(ins, var_types, sym_table, free, then);
                    ins.push(Instruction::Copy {
                        loc: *loc,
                        source: then_value,
                        dest: result.clone(),
                    });
                    ins.push(Instruction::Jump {
                        loc: *loc,
                        label: Box::new(end_label.clone()),
                    });

                    ins.push(else_label);
                    let else_value = visit(ins, var_types, sym_table, free, els);
                    ins.push(Instruction::Copy {
                        loc: *loc,
                        source: else_value,
                        dest: result.clone(),
                    });

                    ins.push(end_label);
                    result
                }
            }
        }

        Ast::While {
            cond, then, loc, ..
        } => {
            let cond_label = new_label(loc, free);
            let do_label = new_label(loc, free);
            let end_label = new_label(loc, free);

            // Condition check
            ins.push(cond_label.clone());
            let cond_var = visit(ins, var_types, sym_table, free, cond);
            ins.push(Instruction::CondJump {
                loc: *loc,
                cond: cond_var,
                then_label: Box::new(do_label.clone()),
                else_label: Box::new(end_label.clone()),
            });

            // Loop
            ins.push(do_label.clone());
            visit(ins, var_types, sym_table, free, then);
            ins.push(Instruction::Jump {
                loc: *loc,
                label: Box::new(cond_label),
            });

            ins.push(end_label);

            IRVar(String::from("unit"))
        }
    }
}
