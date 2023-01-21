use std::process;

use enalang;

fn main() {
    let vec: Vec<String> = std::env::args().skip(1).collect();

    if vec.len() < 1 {
        eprintln!("err: specify files");
        process::exit(1);
    }

    let err = enalang::run(vec);
    match err {
        Err(e) => println!("{:?}", e),
        _ => {}
    }
}
