use crate::tokenizer;

pub fn compile(source_code: String) -> String {
    let tokens = tokenizer::tokenize(source_code);

    // Debug print
    for token in tokens {
        let text = token.value;
        print!("{text} ");
    }
    println!();

    unimplemented!()
}
