use crate::{interpreter::run_interpret, parser, tokenizer};

const INTERPRET: bool = true;

pub fn compile(source_code: String) -> String {
    let tokens = tokenizer::tokenize(source_code);

    println!(); // Extra newline to make reading output easier

    // Debug print
    for token in &tokens {
        let text = &token.value;
        print!("{text} ");
    }
    println!();

    let ast = parser::parse(tokens);
    println!("{ast:?}");

    if INTERPRET {
        println!();
        run_interpret(&ast);
    }

    println!(); // Extra newline to make reading output easier
    todo!("Implement the rest of the compile process")
}
