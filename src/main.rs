use clap::{Args, CommandFactory, Parser, Subcommand};
use enalang::{
    vm::{self, ir},
    EnaError,
};
use glob::glob;
use std::fs;
use std::io::Write;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile given files for future execution
    Compile(Compile),
    /// Runs IR files
    Run(Run),
    /// Combines several IR files into one
    Link(Link),
}

#[derive(Args)]
struct Link {
    /// Files to merge
    files: Vec<String>,
    /// Output file
    #[arg(short, long)]
    output: Option<String>,
}

#[derive(Args)]
struct Compile {
    /// Files to compile
    files: Vec<String>,
    /// Output file
    #[arg(short, long)]
    output: Option<String>,
}

#[derive(Args)]
struct Run {
    /// Files to run
    files: Vec<String>,
    /// Word to start execution from
    #[arg(short, long)]
    main_word: Option<String>,
    /// Enable stack debugging
    #[arg(long, default_value_t = false)]
    debug_stack: bool,
    /// Enable or disable GC
    #[arg(long, default_value_t = true)]
    gc: bool,
    /// Whether to debug gc
    #[arg(long, default_value_t = false)]
    debug_gc: bool,
}

fn save_ir<'a>(output: &String, i: &'a ir::IR<'a>) {
    let v = match i.into_serializable().into_vec() {
        Ok(i) => i,
        Err(_) => report_error(clap::error::ErrorKind::InvalidValue, "failed to serialize"),
    };
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(output)
        .unwrap();
    if let Err(e) = file.write_all(&v) {
        report_error(clap::error::ErrorKind::Io, e)
    }
}

fn report_error(kind: clap::error::ErrorKind, message: impl std::fmt::Display) -> ! {
    Cli::command().error(kind, message).exit()
}

fn into_paths(strs: &Vec<String>) -> Vec<String> {
    let mut files: Vec<String> = Vec::new();

    for str in strs {
        let res = match glob(str) {
            Err(e) => report_error(clap::error::ErrorKind::InvalidValue, e),
            Ok(i) => i,
        };

        for pattern in res {
            match pattern {
                Ok(path) => files.push(path.display().to_string()),
                Err(e) => {
                    report_error(clap::error::ErrorKind::ValueValidation, e);
                }
            }
        }
    }

    files
}

fn link(opts: Link) {
    let paths = into_paths(&opts.files);
    let mut file_contents: Vec<Vec<u8>> = Vec::new();

    if paths.is_empty() {
        report_error(clap::error::ErrorKind::Io, "no files were given")
    }

    for path in paths {
        match fs::read(&path) {
            Ok(i) => file_contents.push(i),
            Err(_) => report_error(clap::error::ErrorKind::Io, format!("failed to read {path}")),
        };
    }
    let mut irs: Vec<ir::IR> = Vec::new();

    for (i, _) in file_contents.iter().enumerate() {
        let i = match ir::from_vec(&file_contents[i]).map_err(EnaError::SerializationError) {
            Ok(i) => i,
            Err(e) => report_error(clap::error::ErrorKind::ValueValidation, format!("{e:?}")),
        };
        let i = match i.into_ir() {
            Ok(i) => i,
            Err(e) => report_error(clap::error::ErrorKind::ValueValidation, format!("{e:?}")),
        };
        irs.push(i.clone());
    }

    let mut ir = ir::IR::new();

    for sub_ir in irs {
        if let Err(e) = ir.add(&sub_ir) {
            report_error(clap::error::ErrorKind::ValueValidation, format!("{e:?}"));
        }
    }

    save_ir(&opts.output.unwrap_or("output.enair".to_string()), &ir);
}

fn run(opts: Run) {
    let paths = into_paths(&opts.files);
    let mut file_contents: Vec<Vec<u8>> = Vec::new();

    if paths.is_empty() {
        report_error(clap::error::ErrorKind::Io, "no files were given")
    }

    for path in paths {
        match fs::read(&path) {
            Ok(i) => file_contents.push(i),
            Err(_) => report_error(clap::error::ErrorKind::Io, format!("failed to read {path}")),
        };
    }
    let mut irs: Vec<ir::IR> = Vec::new();

    for (i, _) in file_contents.iter().enumerate() {
        let i = match ir::from_vec(&file_contents[i]).map_err(EnaError::SerializationError) {
            Ok(i) => i,
            Err(e) => report_error(clap::error::ErrorKind::ValueValidation, format!("{e:?}")),
        };
        let i = match i.into_ir() {
            Ok(i) => i,
            Err(e) => report_error(clap::error::ErrorKind::ValueValidation, format!("{e:?}")),
        };
        irs.push(i.clone());
    }

    let mut ir = ir::IR::new();

    for sub_ir in irs {
        if let Err(e) = ir.add(&sub_ir) {
            report_error(clap::error::ErrorKind::ValueValidation, format!("{e:?}"))
        }
    }

    let group = vm::native::group();
    if let Err(e) = group.apply(&mut ir) {
        report_error(clap::error::ErrorKind::ValueValidation, format!("{e:?}"))
    }

    let mut virt = vm::machine::VM::new(opts.gc, opts.debug_gc);
    virt.debug_stack = opts.debug_stack;

    if let Err(e) = virt.run(ir, &opts.main_word.unwrap_or("main".to_string())) {
        eprintln!(
            "ran into unhandled exception: {e:?}\nstack: {:?}",
            virt.call_stack
        )
    }
}

fn compile(opts: Compile) {
    let paths = into_paths(&opts.files);
    let mut file_contents: Vec<String> = Vec::new();

    if paths.is_empty() {
        report_error(clap::error::ErrorKind::Io, "no files were given")
    }

    for path in paths {
        let data = match fs::read_to_string(path) {
            Ok(i) => i,
            Err(e) => report_error(clap::error::ErrorKind::Io, e),
        };
        file_contents.push(data);
    }

    let compiled = enalang::compile_many(&file_contents);
    match compiled {
        Ok(i) => save_ir(&opts.output.unwrap_or("output.enair".to_string()), &i),
        Err(e) => report_error(
            clap::error::ErrorKind::Io,
            format!("failed to write file {e:?}"),
        ),
    };
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Compile(c) => compile(c),
        Commands::Run(r) => run(r),
        Commands::Link(l) => link(l),
    }
}
