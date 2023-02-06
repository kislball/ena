use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use std::time;

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
    /// Prints ir before exit
    #[arg(short, long, default_value_t = false)]
    print_ir: bool,
}

#[derive(Args)]
struct Run {
    /// File to run
    file: String,
    /// Block to start execution from
    #[arg(short, long)]
    main_word: Option<String>,
    /// Enable stack debugging
    #[arg(long, default_value_t = false)]
    debug_stack: bool,
    /// Enable or disable GC
    #[arg(long, default_value_t = true)]
    gc: bool,
    /// Whether to debug GC
    #[arg(long, default_value_t = false)]
    debug_gc: bool,
    /// Whether to debug calls
    #[arg(long, default_value_t = false)]
    debug_calls: bool,
}

fn compile(c: Compile) {
    let mut ena = enalang::Ena::new(enalang::EnaOptions::default());
    if let Err(e) = ena.read_files(&c.files[..]) {
        ena.report_error_and_exit(e);
    }
    if let Err(e) = ena.parse_files() {
        ena.report_error_and_exit(e);
    }
    if let Err(e) = ena.compile_files() {
        ena.report_error_and_exit(e);
    }
    if let Err(e) = ena.link_files() {
        ena.report_error_and_exit(e);
    }
    if let Err(e) = ena.save(&c.output.unwrap_or("output.enair".to_string())) {
        ena.report_error_and_exit(e);
    }

    if c.print_ir {
        println!("{:#?}", ena.ir.unwrap());
    }
}

fn link(l: Link) {
    let mut ena = enalang::Ena::new(enalang::EnaOptions::default());
    if let Err(e) = ena.load_irs(&l.files[..]) {
        ena.report_error_and_exit(e);
    }
    // if let Err(e) = ena.link_files() {
    //     ena.report_error_and_exit(e);
    // }
    if let Err(e) = ena.save(&l.output.unwrap_or("output.enair".to_string())) {
        ena.report_error_and_exit(e);
    }
}

fn run(r: Run) {
    let mut ena = enalang::Ena::new(enalang::EnaOptions {
        debug_gc: r.debug_gc,
        gc: r.gc,
        debug_stack: r.debug_stack,
        debug_calls: r.debug_calls,
    });
    match ena.load_ir(&r.file) {
        Err(e) => {
            ena.report_error_and_exit(e);
        }
        Ok(e) => {
            ena.ir = Some(e);
        }
    }
    if let Err(e) = ena.run(&r.main_word.unwrap_or("main".to_string())) {
        ena.report_error_and_exit(e);
    }
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Compile(c) => {
            let begin = time::Instant::now();
            compile(c);
            println!(
                "Compilation {} in {:?}",
                "successful".bold().green(),
                begin.elapsed()
            );
        }
        Commands::Link(l) => {
            let begin = time::Instant::now();
            link(l);
            println!(
                "Linking {} in {:?}",
                "successful".bold().green(),
                begin.elapsed()
            );
        }
        Commands::Run(r) => run(r),
    };
}
