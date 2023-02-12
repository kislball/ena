use clap::{Args, Parser, Subcommand};
use enalang::EnaError;
use enalang_vm::machine::VMOptions;

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
    /// Checks **linked** IR
    Check(Check),
}

#[derive(Args)]
struct Link {
    /// Files to merge
    files: Vec<String>,
    /// Output file
    #[arg(short, long)]
    output: Option<String>,
    /// Whether to save source map
    #[arg(short, long, default_value_t = true)]
    source_map: bool,
}

#[derive(Args)]
struct Check {
    /// Files to check
    files: Vec<String>,
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
    /// Whether to save source map
    #[arg(short, long, default_value_t = true)]
    source_map: bool,
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

fn compile(c: Compile, ena: &mut enalang::Ena) -> Result<(), EnaError> {
    ena.read_files(&c.files[..])?;
    ena.parse_files()?;
    ena.compile_files()?;
    ena.link_files()?;
    ena.save(
        &c.output.unwrap_or("output.enair".to_string()),
        c.source_map,
    )?;

    if c.print_ir {
        println!("{:#?}", ena.ir.as_ref().unwrap());
    }
    Ok(())
}

fn check(l: Check, ena: &mut enalang::Ena) -> Result<(), EnaError> {
    ena.load_irs(&l.files[..])?;
    ena.check()?;
    // ena.save(&l.output.unwrap_or("output.enair".to_string()), l.source_map)?;
    Ok(())
}

fn link(l: Link, ena: &mut enalang::Ena) -> Result<(), EnaError> {
    ena.load_irs(&l.files[..])?;
    ena.check()?;
    ena.save(
        &l.output.unwrap_or("output.enair".to_string()),
        l.source_map,
    )?;
    Ok(())
}

fn run(r: Run, ena: &mut enalang::Ena) -> Result<(), EnaError> {
    match ena.load_ir(&r.file) {
        Err(e) => {
            ena.report_error_and_exit(e);
        }
        Ok(e) => {
            ena.ir = Some(e);
        }
    }
    ena.run(
        &r.main_word.unwrap_or("main".to_string()),
        VMOptions {
            debug_stack: r.debug_stack,
            enable_gc: r.gc,
            debug_gc: r.debug_gc,
            debug_calls: r.debug_calls,
        },
    )?;
    Ok(())
}

fn main() {
    let args = Cli::parse();
    let mut ena = enalang::Ena::new();

    let res = match args.command {
        Commands::Compile(c) => compile(c, &mut ena),
        Commands::Link(l) => link(l, &mut ena),
        Commands::Check(c) => check(c, &mut ena),
        Commands::Run(r) => run(r, &mut ena),
    };

    if let Err(e) = res {
        ena.report_error_and_exit(e);
    }
}
