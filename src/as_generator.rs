use std::collections::HashMap;

use crate::{
    ir_generator::{IRVar, Instruction},
    svec,
    util::label_name,
};

struct Locals {
    var_to_location: HashMap<IRVar, String>,
    stack_used: i32,
}

fn add_var(locals: &mut Locals, var: IRVar) {
    if let std::collections::hash_map::Entry::Vacant(e) = locals.var_to_location.entry(var) {
        locals.stack_used += 8;
        let stack_used = locals.stack_used;
        e.insert(format!("-${stack_used}(%rbp)"));
    }
}

fn init_locals(instructions: &[Instruction]) -> Locals {
    let mut locals = Locals {
        var_to_location: HashMap::new(),
        stack_used: 0,
    };

    for ins in instructions {
        #[expect(unused_variables)]
        match ins {
            Instruction::Call {
                loc,
                fun,
                args,
                dest,
            } => add_var(&mut locals, dest.clone()),
            Instruction::Copy { loc, source, dest } => add_var(&mut locals, dest.clone()),
            Instruction::LoadBoolConst { loc, value, dest } => add_var(&mut locals, dest.clone()),
            Instruction::LoadIntConst { loc, value, dest } => add_var(&mut locals, dest.clone()),

            Instruction::Label { loc, name } => (),
            Instruction::CondJump {
                loc,
                cond,
                then_label,
                else_label,
            } => (),
            Instruction::Jump { loc, label } => (),
        }
    }

    locals
}

fn emit(lines: &mut Vec<String>, line: String) {
    lines.push(line);
}

pub fn run_as_gen(instructions: &[Instruction]) -> String {
    let locals = init_locals(instructions);

    // lines = []
    let mut lines = svec![
        "    .extern print_int",
        "    .extern print_bool",
        "    .extern read_int",
        "    .global main",
        "    .type main, @function",
        "",
        "    .section .text",
        "",
        "main:"
    ];

    for ins in instructions {
        emit(&mut lines, format!("# {ins}"));
        match ins {
            Instruction::Label { name, .. } => {
                //  ".L" prefix marks the symbol as "private".
                //  This makes GDB backtraces look nicer too:
                //  https://stackoverflow.com/a/26065570/965979
                emit(&mut lines, format!("L.{name}"));
            }

            Instruction::LoadIntConst { value, dest, .. } => {
                let dest_ref = &locals.var_to_location[dest];
                if -2_i64.pow(31) <= *value && *value < 2_i64.pow(31) {
                    emit(&mut lines, format!("movq {value} {dest_ref}"));
                } else {
                    //Due to a quirk of x86-64, we must use
                    //a different instruction for large integers.
                    //It can only write to a register,
                    //not a memory location, so we use %rax
                    //as a temporary.
                    emit(&mut lines, format!("movabsq {value}, %rax"));
                    emit(&mut lines, format!("movq %rax, %rax {dest_ref}"));
                }
            }

            Instruction::Jump { label, .. } => {
                let name = label_name(label);
                emit(&mut lines, format!("jmp .L{name}"));
            }

            // Unimplemented
            Instruction::Call {
                loc,
                fun,
                args,
                dest,
            } => todo!(),
            Instruction::CondJump {
                loc,
                cond,
                then_label,
                else_label,
            } => todo!(),
            Instruction::Copy { loc, source, dest } => todo!(),
            Instruction::LoadBoolConst { loc, value, dest } => todo!(),
        }
    }
    lines.join("\n")
}
