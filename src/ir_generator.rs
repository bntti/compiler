use std::{collections::HashMap, fmt::Display};

use crate::{
    parser::{Ast, Type},
    util::Location,
};

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct IRVar(String);

#[derive(Debug)]
#[expect(dead_code)]
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
        #[expect(unused_variables)]
        match self {
            Instruction::LoadBoolConst { loc, value, dest } => {
                write!(f, "LoadBoolConst({value}, {dest})")
            }
            Instruction::LoadIntConst { loc, value, dest } => {
                write!(f, "LoadIntConst({value}, {dest})")
            }
            Instruction::Copy { loc, source, dest } => {
                write!(f, "Copy({source}, {dest})")
            }
            Instruction::Call {
                loc,
                fun,
                args,
                dest,
            } => {
                let args = args
                    .iter()
                    .map(|arg| format!("{arg}"))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "Call({fun}, [{args}], {dest})")
            }
            Instruction::Jump { loc, label } => {
                write!(f, "Jump({label})")
            }
            Instruction::CondJump {
                loc,
                cond,
                then_label,
                else_label,
            } => {
                write!(f, "CondJump({cond}, {then_label}, {else_label})")
            }
            Instruction::Label { loc, name } => {
                write!(f, "{name}")
            }
        }
    }
}

pub fn run_ir_gen(root_ast: Ast) -> Vec<Instruction> {
    // The type is actually not Unit, but we do not need the type for now
    let root_types = HashMap::from([
        (IRVar(String::from("or")), Type::Unit),
        (IRVar(String::from("and")), Type::Unit),
        (IRVar(String::from("<")), Type::Unit),
        (IRVar(String::from("<=")), Type::Unit),
        (IRVar(String::from(">")), Type::Unit),
        (IRVar(String::from(">=")), Type::Unit),
        (IRVar(String::from("+")), Type::Unit),
        (IRVar(String::from("-")), Type::Unit),
        (IRVar(String::from("*")), Type::Unit),
        (IRVar(String::from("/")), Type::Unit),
        (IRVar(String::from("%")), Type::Unit),
    ]);
    generate_ir(root_types, root_ast)
}

fn generate_ir(root_types: HashMap<IRVar, Type>, root_ast: Ast) -> Vec<Instruction> {
    let mut var_types = root_types.clone();
    let var_unit = IRVar(String::from("unit"));
    var_types.insert(var_unit, Type::Unit);

    let mut free = 0;
    let mut ins = vec![];
    let mut root_symbol_table = vec![HashMap::new()];
    for (v, _) in root_types.iter() {
        root_symbol_table[0].insert(v.0.clone(), v.clone());
    }

    let var_final_result = visit(
        &mut ins,
        &mut var_types,
        &mut root_symbol_table,
        &mut free,
        &root_ast,
    );

    match var_types[&var_final_result] {
        Type::Bool => todo!("Emit call to print_bool"),
        Type::Int => todo!("Emit call to print_int"),
        _ => (),
    }

    ins
}

fn new_var(var_types: &mut HashMap<IRVar, Type>, free: &mut usize, t: Type) -> IRVar {
    let var = IRVar(String::from("x") + &free.to_string());
    *free += 1;
    var_types.insert(var.clone(), t);
    var
}

fn visit(
    ins: &mut Vec<Instruction>,
    var_types: &mut HashMap<IRVar, Type>,
    sym_table: &mut Vec<HashMap<String, IRVar>>,
    free: &mut usize,
    ast: &Ast,
) -> IRVar {
    #[expect(unused_variables)]
    match ast {
        Ast::NoneLiteral { typ, loc } => IRVar(String::from("unit")),
        Ast::BoolLiteral { val, typ, loc } => {
            let var = new_var(var_types, free, Type::Bool);
            ins.push(Instruction::LoadBoolConst {
                loc: *loc,
                value: *val,
                dest: var.clone(),
            });
            var
        }
        Ast::IntLiteral { val, typ, loc } => {
            let var = new_var(var_types, free, Type::Int);
            ins.push(Instruction::LoadIntConst {
                loc: *loc,
                value: *val,
                dest: var.clone(),
            });
            var
        }
        Ast::Identifier { name, typ, loc } => {
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
        Ast::Minus { stat, typ, loc } => {
            let var_op = sym_table[0][&String::from("minus")].clone();
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
            let var_op = sym_table[0][&String::from("neg")].clone();
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

        Ast::Root { stats, typ, loc } => {
            visit(
                ins,
                var_types,
                sym_table,
                free,
                &Ast::Block {
                    stats: stats.clone(), // Inoptimal, too lazy to make duplicate code
                    typ: typ.clone(),
                    loc: *loc,
                },
            )
        }
        Ast::Block { stats, typ, loc } => {
            let len = &stats.len();
            sym_table.push(HashMap::new());
            for (i, stat) in stats.iter().enumerate() {
                if i == len - 1 {
                    return visit(ins, var_types, sym_table, free, stat);
                }
                visit(ins, var_types, sym_table, free, stat);
            }
            IRVar(String::from("unit")) // Empty block
        }

        // Unimplemented
        Ast::Function {
            name,
            params,
            typ,
            loc,
        } => todo!(),
        Ast::If {
            cond,
            then,
            els,
            typ,
            loc,
        } => todo!(),
        Ast::Var {
            name,
            value,
            typ,
            loc,
        } => todo!(),
        Ast::While {
            cond,
            then,
            typ,
            loc,
        } => todo!(),
    }
}
