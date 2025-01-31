use std::process::exit;

use crate::{
    interpreter::run_interpret, ir_generator::run_ir_gen, parser, tokenizer,
    typechecker::run_typecheck,
};

const INTERPRET: bool = false;

pub fn compile(source_code: String) -> String {
    let tokens = tokenizer::tokenize(source_code);

    println!(); // Extra newline to make reading output easier

    println!("Tokenizer output:");
    for token in &tokens {
        let text = &token.value;
        print!("{text} ");
    }
    println!();
    println!();

    let mut ast = parser::parse(tokens);
    // println!("{ast:?}");

    if INTERPRET {
        println!();
        run_interpret(&ast);
        exit(0);
    }

    run_typecheck(&mut ast);
    let ir = run_ir_gen(ast);

    println!("IR generator output:");
    for ins in ir.iter() {
        println!("{ins}");
    }

    println!(); // Extra newline to make reading output easier
    todo!("Implement the rest of the compile process")
}
