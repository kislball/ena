use clap::{Parser, CommandFactory};
use enalang;

#[derive(Parser)]
#[command(about = "Interpreter for the Ena programming language", version)]
struct Args {
    /// Files to interpret
    files: Vec<String>,
    /// Word to start execution from
    #[arg(short, long)]
    main_word: Option<String>,
    /// Stage to stop execution on.
    #[arg(value_enum, short, long)]
    stage: Option<enalang::Stage>
}

fn main() {
    let args = Args::parse();
    let main = match args.main_word {
        Some(i) => i,
        None => "main".to_string(),
    };

    if args.files.len() < 1 {
        Args::command()
            .error(clap::error::ErrorKind::TooFewValues, "specify files")
            .exit();
    }

    let options = enalang::RunOptions {
        file_names: args.files,
        stage: args.stage.unwrap_or(enalang::Stage::Run),
        main: main,
    };

    let err = enalang::run(&options);
    match err {
        Err(e) => println!("{:?}", e),
        _ => {}
    }
}
