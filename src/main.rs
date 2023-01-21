use enalang;

fn main() {
    let vec: Vec<&'static str> = vec!["./test.ena"];
    let err = enalang::run(&vec);
    match err {
        Err(e) => println!("{:?}", e),
        _ => {}
    }
}
