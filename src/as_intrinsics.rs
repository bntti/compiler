#![expect(clippy::useless_format)]

use std::collections::{HashMap, HashSet};

use crate::{
    as_generator::{emit, Locals},
    ir_generator::Instruction,
};

pub fn parse_intrinsics(locals: &Locals, lines: &mut Vec<String>, ins: &Instruction) -> bool {
    let Instruction::Call {
        fun, args, dest, ..
    } = ins
    else {
        panic!()
    };

    let operators = HashSet::from([
        "minus", "not", "+", "-", "*", "/", "%", "==", "!=", "<", "<=", ">", ">=",
    ]);
    if !operators.contains(fun.0.as_str()) {
        return false;
    }

    let integer_comparisons = HashMap::from([
        ("==", "sete"),
        ("!=", "setne"),
        ("<", "setl"),
        ("<=", "setle"),
        (">", "setg"),
        (">=", "setge"),
    ]);

    let source_ref1 = &locals.var_to_location[&args[0]];
    let result_reg = "%rax";
    match fun.0.as_str() {
        // unary_minus
        "minus" => {
            emit(lines, format!("movq {source_ref1}, {result_reg}"));
            emit(lines, format!("negq {result_reg}"));
        }

        // Unary_not
        "not" => {
            emit(lines, format!("movq {source_ref1}, {result_reg}"));
            emit(lines, format!("xorq $1, {result_reg}"));
        }

        "+" => {
            let source_ref2 = &locals.var_to_location[&args[1]];
            if source_ref1 != result_reg {
                emit(lines, format!("movq {source_ref1}, {result_reg}"));
            }
            emit(lines, format!("addq {source_ref2}, {result_reg}"));
        }

        "-" => {
            let source_ref2 = &locals.var_to_location[&args[1]];
            if source_ref1 != result_reg {
                emit(lines, format!("movq {source_ref1}, {result_reg}"));
            }
            emit(lines, format!("subq {source_ref2}, {result_reg}"));
        }

        "*" => {
            let source_ref2 = &locals.var_to_location[&args[1]];
            if source_ref1 != result_reg {
                emit(lines, format!("movq {source_ref1}, {result_reg}"));
            }
            emit(lines, format!("imulq {source_ref2}, {result_reg}"));
        }

        "/" => {
            let source_ref2 = &locals.var_to_location[&args[1]];
            emit(lines, format!("movq {source_ref1}, %rax"));
            emit(lines, format!("cqto"));
            emit(lines, format!("idivq {source_ref2}"));
            if result_reg != "%rax" {
                emit(lines, format!("movq %rax, {result_reg}"));
            }
        }

        "%" => {
            let source_ref2 = &locals.var_to_location[&args[1]];
            emit(lines, format!("movq {source_ref1}, %rax"));
            emit(lines, format!("cqto"));
            emit(lines, format!("idivq {source_ref2}"));
            if result_reg != "%rdx" {
                emit(lines, format!("movq %rdx, {result_reg}"));
            }
        }

        // Check integer operations
        _ => {
            let op = integer_comparisons[fun.0.as_str()];

            let source_ref2 = &locals.var_to_location[&args[1]];
            emit(lines, format!("xor %rax, %rax"));
            emit(lines, format!("movq {source_ref1}, %rdx"));
            emit(lines, format!("cmpq {source_ref2}, %rdx"));
            emit(lines, format!("{op} %al"));
            if result_reg != "%rax" {
                emit(lines, format!("movq %rax, {result_reg}"));
            }
        }
    }

    let result_ref = &locals.var_to_location[dest];
    emit(lines, format!("movq %rax, {result_ref}"));

    true
}
