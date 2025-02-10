use std::process::exit;

use crate::{
    as_generator::run_as_gen, assembler::assemble, interpreter::run_interpret,
    ir_generator::run_ir_gen, parser, tokenizer, typechecker::run_typecheck,
};

const INTERPRET: bool = false;

pub fn compile(source_code: String) -> Vec<u8> {
    let tokens = tokenizer::tokenize(source_code);

    println!(); // Extra newline to make reading output easier

    // Tokenize
    println!("\x1b[1mTokenizer output\x1b[0;48;2;30;30;30m");
    for token in &tokens {
        let text = &token.value;
        print!("{text} ");
    }
    println!("\x1b[0m\n");

    // Generate AST
    let mut ast = parser::parse(tokens);
    // println!("{ast:#?}");

    // Interpret
    if INTERPRET {
        println!();
        run_interpret(&ast);
        exit(0);
    }

    // Type check
    run_typecheck(&mut ast);

    // IR generator
    let ir = run_ir_gen(ast);

    println!("\x1b[1mIR generator output\x1b[0;48;2;30;30;30m");
    for ins in ir.iter() {
        println!("{ins}");
    }
    println!("\x1b[0m\n");

    // Assembly generator
    let assembly = run_as_gen(&ir);

    println!("\x1b[1mAssembly generator output\x1b[0;48;2;30;30;30m");
    println!("{assembly}");
    println!("\x1b[0m");

    println!(); // Extra newline to make reading output easier
    assemble(assembly)
}
