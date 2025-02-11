use std::collections::HashMap;

use crate::{
    as_intrinsics::parse_intrinsics,
    ir_generator::{IRVar, Instruction},
    svec,
    util::label_name,
};

pub struct Locals {
    pub var_to_location: HashMap<IRVar, String>,
    stack_used: i32,
}

fn add_var(locals: &mut Locals, var: IRVar) {
    if let std::collections::hash_map::Entry::Vacant(e) = locals.var_to_location.entry(var) {
        locals.stack_used += 8;
        let stack_used = locals.stack_used;
        e.insert(format!("-{stack_used}(%rbp)"));
    }
}

fn init_locals(instructions: &[Instruction]) -> Locals {
    let mut locals = Locals {
        var_to_location: HashMap::new(),
        stack_used: 0,
    };

    add_var(&mut locals, IRVar(String::from("unit")));
    for ins in instructions {
        match ins {
            Instruction::Call { dest, .. } => add_var(&mut locals, dest.clone()),
            Instruction::Copy { dest, .. } => add_var(&mut locals, dest.clone()),
            Instruction::LoadBoolConst { dest, .. } => add_var(&mut locals, dest.clone()),
            Instruction::LoadIntConst { dest, .. } => add_var(&mut locals, dest.clone()),

            Instruction::Label { .. } => (),
            Instruction::CondJump { .. } => (),
            Instruction::Jump { .. } => (),
        }
    }

    locals
}

pub fn emit(lines: &mut Vec<String>, line: String) {
    lines.push(line);
}

pub fn run_as_gen(instructions: &[Instruction]) -> String {
    let locals = init_locals(instructions);
    let space = locals.stack_used;

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
        "main:",
        "    pushq %rbp",
        "    movq %rsp, %rbp",
        format!("    subq ${space}, %rsp")
    ];

    for ins in instructions {
        emit(&mut lines, format!("# {ins}"));
        match ins {
            Instruction::Label { name, .. } => {
                //  ".L" prefix marks the symbol as "private".
                //  This makes GDB backtraces look nicer too:
                //  https://stackoverflow.com/a/26065570/965979
                emit(&mut lines, format!(".L{name}:"));
            }

            Instruction::LoadIntConst { value, dest, .. } => {
                let dest_ref = &locals.var_to_location[dest];
                if -2_i64.pow(31) <= *value && *value < 2_i64.pow(31) {
                    emit(&mut lines, format!("movq ${value}, {dest_ref}"));
                } else {
                    //Due to a quirk of x86-64, we must use
                    //a different instruction for large integers.
                    //It can only write to a register,
                    //not a memory location, so we use %rax
                    //as a temporary.
                    emit(&mut lines, format!("movabsq ${value}, %rax"));
                    emit(&mut lines, format!("movq %rax, %rax {dest_ref}"));
                }
            }

            Instruction::Jump { label, .. } => {
                let name = label_name(label);
                emit(&mut lines, format!("jmp .L{name}"));
            }

            Instruction::LoadBoolConst { value, dest, .. } => {
                let dest_ref = &locals.var_to_location[dest];
                let bin_value = if *value { 1 } else { 0 };
                emit(&mut lines, format!("movq ${bin_value}, {dest_ref}"));
            }

            Instruction::Copy { source, dest, .. } => {
                let source_ref = &locals.var_to_location[source];
                let dest_ref = &locals.var_to_location[dest];

                emit(&mut lines, format!("movq {source_ref}, %rax"));
                emit(&mut lines, format!("movq %rax, {dest_ref}"));
            }

            Instruction::CondJump {
                cond,
                then_label,
                else_label,
                ..
            } => {
                let cond_ref = &locals.var_to_location[cond];
                let then_label = label_name(then_label);
                let else_label = label_name(else_label);

                emit(&mut lines, format!("cmpq $0, {cond_ref}"));
                emit(&mut lines, format!("jne .L{then_label}"));
                emit(&mut lines, format!("jmp .L{else_label}"));
            }

            Instruction::Call {
                fun, args, dest, ..
            } => {
                if !parse_intrinsics(&locals, &mut lines, ins) {
                    let regs = ["%rdi", "%rsi", "%rdx", "%rcx", "%r8", "%r9"];
                    for (i, arg) in args.iter().enumerate() {
                        let source_ref = &locals.var_to_location[arg];
                        let reg = regs[i];
                        emit(&mut lines, format!("movq {source_ref}, {reg}"));
                    }

                    let dest_ref = &locals.var_to_location[dest];
                    let name = &fun.0;
                    emit(&mut lines, format!("callq {name}"));
                    emit(&mut lines, format!("movq %rax, {dest_ref}"));
                }
            }
        }
    }

    lines.extend(svec![
        "movq $0, %rax",
        "movq %rbp, %rsp",
        "popq %rbp",
        "ret",
    ]);

    lines.join("\n")
}
