use crate::{interpreter::interpret, parser, tokenizer};

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

    interpret(&ast);

    println!(); // Extra newline to make reading output easier
    todo!("Implement the rest of the compile process")
}
