use std::collections::HashMap;

use crate::{
    as_intrinsics::parse_intrinsics,
    ir_generator::{IRVar, Instruction},
    parser::Type,
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
            Instruction::Return { .. } => (),
        }
    }

    locals
}

pub fn emit(lines: &mut Vec<String>, line: String) {
    lines.push(line);
}

pub fn run_as_gen(instructions: HashMap<String, (Vec<IRVar>, Vec<Instruction>, Type)>) -> String {
    let mut output = String::from(
        r#"
    .extern print_int
    .extern print_bool
    .extern read_int
    .global main
    .type main, @function

    .section .text
        "#,
    );
    for (name, ins) in instructions.iter() {
        let fn_output = run_as_gen_function(name, &ins.0, &ins.1, &ins.2);
        output.push_str(&fn_output);
        output.push('\n');
    }
    output
}

fn run_as_gen_function(
    name: &String,
    params: &[IRVar],
    instructions: &[Instruction],
    typ: &Type,
) -> String {
    let mut locals = init_locals(instructions);
    for param in params {
        add_var(&mut locals, param.clone());
    }
    let space = locals.stack_used;

    // lines = []
    let mut lines = svec![
        format!("{name}:"),
        "    pushq %rbp",
        "    movq %rsp, %rbp",
        format!("    subq ${space}, %rsp")
    ];

    let fn_regs = ["%rdi", "%rsi", "%rdx", "%rcx", "%r8", "%r9"];
    for (i, param) in params.iter().enumerate() {
        let reg = fn_regs[i];
        let param_ref = &locals.var_to_location[param];
        emit(&mut lines, format!("movq {reg}, {param_ref}"));
    }

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
                    emit(&mut lines, format!("movq %rax, {dest_ref}"));
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
                    for (i, arg) in args.iter().enumerate() {
                        let source_ref = &locals.var_to_location[arg];
                        let reg = fn_regs[i];
                        emit(&mut lines, format!("movq {source_ref}, {reg}"));
                    }

                    let dest_ref = &locals.var_to_location[dest];
                    let name = &fun.0;
                    emit(&mut lines, format!("callq {name}"));
                    emit(&mut lines, format!("movq %rax, {dest_ref}"));
                }
            }

            Instruction::Return { value, .. } => {
                let value_ref = &locals.var_to_location[value];

                lines.extend(svec![
                    format!("movq {value_ref}, %rax"),
                    "movq %rbp, %rsp",
                    "popq %rbp",
                    "ret",
                ]);
            }
        }
    }

    if name.as_str() == "main" || matches!(typ, Type::Unit) {
        lines.extend(svec![
            "movq $0, %rax",
            "movq %rbp, %rsp",
            "popq %rbp",
            "ret",
        ]);
    } else {
        // TODO: Add assembly that panics
        lines.push(String::from("# TODO: Panic"));
        lines.push(String::from(""));
    }

    lines.join("\n")
}
