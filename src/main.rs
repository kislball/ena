use std::{fs, process};
use enalang::{ast,tok};

fn main() {
    let mut tokenizer = tok::Tokenizer::new();
    let file_content = fs::read_to_string("test.ena").expect("fuck you");
    let tokens = tokenizer.parse(file_content);

    let tokens = match tokens {
        Ok(vec) => vec,
        Err(e) => {
            println!("{:?}", e);
            process::exit(-1);
        },
    };

    let mut builder = ast::ASTBuilder::new();
    let nodes = match builder.parse(tokens) {
        Ok(vec) => vec,
        Err(e) => {
            println!("{:?}", e);
            process::exit(-1);
        },
    };

    println!("{:#?}", nodes);
}