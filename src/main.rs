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

fn report_error(kind: clap::error::ErrorKind, message: impl std::fmt::Display) -> ! {
    Args::command()
        .error(kind, message)
        .exit()
}

fn main() {
    let args = Args::parse();
    let main = match args.main_word {
        Some(i) => i,
        None => "main".to_string(),
    };
    let mut files: Vec<String> = vec![];

    for file in args.files {
        let res = match glob::glob(&file) {
            Ok(i) => i,
            Err(e) => {
                report_error(clap::error::ErrorKind::InvalidValue, e);
            },
        };

        for pattern in res {
            match pattern {
                Ok(path) => files.push(path.display().to_string()),
                Err(e) => {
                    report_error(clap::error::ErrorKind::ValueValidation, e);
                },
            }
        }
    }

    if files.len() < 1 {
        report_error(clap::error::ErrorKind::TooFewValues, "specify files");
    }

    let options = enalang::RunOptions {
        file_names: files,
        stage: args.stage.unwrap_or(enalang::Stage::Run),
        main: main,
    };

    let err = enalang::run(&options);
    match err {
        Err(e) => println!("{:?}", e),
        _ => {}
    }
}
