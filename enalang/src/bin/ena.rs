use clap::{Args, Parser, Subcommand};
use enalang::{Ena, EnaError};
use enalang_vm::machine::VMOptions;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile given files for future execution
    Compile(Compile),
    /// Runs IR files
    Run(Run),
    /// Combines several IR files into one
    Link(Link),
    /// Checks linked IR
    Check(Check),
    /// Optimizes IR
    Optimize(Optimize),
    /// Generates documentation
    Doc(Doc),
    /// Print the JSON structure of IR
    JSON(JSON),
    /// Use ENA interactively,
    REPL,
}

#[derive(Args)]
struct JSON {
    /// Print pretty json
    #[arg(short, long, default_value_t = false)]
    pretty: bool,
    /// File to display
    file: String,
}

#[derive(Args)]
struct Doc {
    /// Files to generate docs from
    files: Vec<String>,
    /// Renderer to use
    #[arg(short, long, default_value_t = enalang::DocGen::JSON)]
    generator: enalang::DocGen,
}

#[derive(Args)]
struct Optimize {
    /// Main
    #[arg(short, long)]
    main: Option<String>,
    /// Files to optimize
    files: Vec<String>,
    /// Prints IR before exit
    #[arg(short, long, default_value_t = false)]
    print_ir: bool,
}

#[derive(Args)]
struct Link {
    /// Whether to optimize the resulting IR
    #[arg(long, default_value_t = true)]
    optimize: bool,
    /// Files to merge
    files: Vec<String>,
    /// Output file
    #[arg(short, long)]
    output: Option<String>,
    /// Main
    #[arg(short, long)]
    main: Option<String>,
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

fn doc(d: Doc, ena: &mut enalang::Ena) -> Result<(), EnaError> {
    ena.load_irs(&d.files[..])?;
    let result = ena.generate_doc(d.generator)?;
    println!("{result}");
    Ok(())
}

fn compile(c: Compile, ena: &mut enalang::Ena) -> Result<(), EnaError> {
    ena.read_files(&c.files[..])?;
    ena.parse_files()?;
    ena.compile_files()?;
    ena.link_files()?;
    ena.save(&c.output.unwrap_or("output.enair".to_string()))?;

    if c.print_ir {
        println!("{:#?}", ena.ir.as_ref().unwrap());
    }
    Ok(())
}

fn check(l: Check, ena: &mut enalang::Ena) -> Result<(), EnaError> {
    ena.load_irs(&l.files[..])?;
    ena.check()?;
    Ok(())
}

fn optimize(o: Optimize, ena: &mut enalang::Ena) -> Result<(), EnaError> {
    let main = o.main.unwrap_or(String::from("main"));
    ena.load_irs(&o.files[..])?;
    ena.optimize(&main)?;
    if o.print_ir {
        println!("{:#?}", ena.ir.as_ref().unwrap());
    }
    Ok(())
}

fn link(l: Link, ena: &mut enalang::Ena) -> Result<(), EnaError> {
    let main = l.main.unwrap_or(String::from("main"));
    ena.load_irs(&l.files[..])?;
    ena.check()?;
    if l.optimize {
        ena.optimize(&main)?;
    }
    ena.save(&l.output.unwrap_or("output.enair".to_string()))?;
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

fn json(j: JSON, ena: &mut enalang::Ena) -> Result<(), EnaError> {
    match ena.load_ir(&j.file) {
        Err(e) => {
            ena.report_error_and_exit(e);
        }
        Ok(e) => {
            ena.ir = Some(e);
        }
    }
    ena.display_json(j.pretty)
}

pub fn repl(e: &mut Ena) -> Result<(), EnaError> {
    e.run_repl(VMOptions::default())
}

fn main() {
    let args = Cli::parse();
    let mut ena = enalang::Ena::new();

    let res = match args.command {
        Some(Commands::Compile(c)) => compile(c, &mut ena),
        Some(Commands::Link(l)) => link(l, &mut ena),
        Some(Commands::Check(c)) => check(c, &mut ena),
        Some(Commands::Run(r)) => run(r, &mut ena),
        Some(Commands::Optimize(o)) => optimize(o, &mut ena),
        Some(Commands::Doc(d)) => doc(d, &mut ena),
        Some(Commands::JSON(j)) => json(j, &mut ena),
        Some(Commands::REPL) | None => repl(&mut ena),
    };

    if let Err(e) = res {
        ena.report_error_and_exit(e);
    }
}
