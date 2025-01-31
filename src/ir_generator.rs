use std::collections::HashMap;

use crate::{parser, util::Location};

#[derive(Clone)]
enum Type {
    Int,
    Bool,
    Unit,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct IRVar(String);

#[derive(Debug)]
#[expect(dead_code)]
pub struct Label {
    loc: Location,
    name: String,
}

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
        label: Label,
    },
    CondJump {
        loc: Location,
        cond: IRVar,
        then_label: Label,
        else_label: Label,
    },
    // Label {
    //     name:String,
    // }
}

pub fn run_ir_gen(root_ast: parser::Ast) -> Vec<Instruction> {
    let root_types = HashMap::new();
    generate_ir(root_types, root_ast)
}

fn generate_ir(root_types: HashMap<IRVar, Type>, root_ast: parser::Ast) -> Vec<Instruction> {
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
        root_ast,
    );

    match var_types[&var_final_result] {
        Type::Bool => todo!("Emit call to print_bool"),
        Type::Int => todo!("Emit call to print_int"),
        _ => (),
    }

    ins
}

fn new_var(var_types: &mut HashMap<IRVar, Type>, free: &mut usize, t: Type) -> IRVar {
    let var = IRVar(free.to_string());
    *free += 1;
    var_types.insert(var.clone(), t);
    var
}

fn visit(
    ins: &mut Vec<Instruction>,
    var_types: &mut HashMap<IRVar, Type>,
    sym_table: &mut Vec<HashMap<String, IRVar>>,
    free: &mut usize,
    ast: parser::Ast,
) -> IRVar {
    #[expect(unused_variables)]
    match ast {
        parser::Ast::NoneLiteral { typ, loc } => IRVar(String::from("unit")),
        parser::Ast::BoolLiteral { val, typ, loc } => {
            let var = new_var(var_types, free, Type::Bool);
            ins.push(Instruction::LoadBoolConst {
                loc,
                value: val,
                dest: var.clone(),
            });
            var
        }
        parser::Ast::IntLiteral { val, typ, loc } => {
            let var = new_var(var_types, free, Type::Int);
            ins.push(Instruction::LoadIntConst {
                loc,
                value: val,
                dest: var.clone(),
            });
            var
        }
        parser::Ast::Identifier { name, typ, loc } => {
            for map in sym_table.iter().rev() {
                if map.contains_key(&name) {
                    return map[&name].clone();
                }
            }
            unreachable!();
        }
        parser::Ast::BinaryOp {
            left,
            op,
            right,
            typ,
            loc,
        } => todo!(),
        parser::Ast::Block { stats, typ, loc } => todo!(),
        parser::Ast::Function {
            name,
            params,
            typ,
            loc,
        } => todo!(),
        parser::Ast::If {
            cond,
            then,
            els,
            typ,
            loc,
        } => todo!(),
        parser::Ast::Minus { stat, typ, loc } => todo!(),
        parser::Ast::Negate { stat, typ, loc } => todo!(),
        parser::Ast::Root { stats, typ, loc } => todo!(),
        parser::Ast::Var {
            name,
            value,
            typ,
            loc,
        } => todo!(),
        parser::Ast::While {
            cond,
            then,
            typ,
            loc,
        } => todo!(),
    }
}
