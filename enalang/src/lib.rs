use clap::ValueEnum;
use colored::Colorize;
use enalang_checker::checker::{CheckError, Checker};
use enalang_macro::{MacroError, MacroUnwrapper};
use enalang_repl::{Repl, ReplError};
use flexstr::ToLocalStr;
use glob::glob;
use std::{
    collections::HashMap,
    fmt::Display,
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
    process,
};
use vm::{
    blocks::{Blocks, BlocksError},
    machine::{VMOptions, VM},
    native,
};

pub use enalang_checker as checker;
pub use enalang_compiler as compiler;
pub use enalang_docgen as docgen;
pub use enalang_ir as ir;
pub use enalang_optimizer as optimizer;
pub use enalang_vm as vm;

pub mod util;

#[derive(Debug, thiserror::Error)]
pub enum EnaError {
    #[error("tokenizer error in file `{0}` - `{1}`")]
    TokenizerError(String, compiler::tok::TokenizerError),
    #[error("ast error in file `{0}` - `{1}`")]
    ASTError(String, compiler::ast::ASTError),
    #[error("irgen error in file `{0}` - `{1}`")]
    IRGenError(String, compiler::irgen::IRGenError),
    #[error("ir error - `{0}`")]
    IRError(ir::IRError),
    #[error("serialization error - `{0}`")]
    SerializationError(ir::SerializationError),
    #[error("VM error - `{0}`")]
    VMError(vm::machine::VMError),
    #[error("checker error - `{0}`")]
    CheckerError(Box<dyn CheckError>),
    #[error("checker errors(output not supported)")]
    CheckerErrors(Vec<Box<dyn CheckError>>),
    #[error("failed to read glob pattern `{0}`")]
    FailedToReadGlobPattern(String),
    #[error("fs error `{0}`")]
    FSError(String),
    #[error("blocks error - `{0}`")]
    BlocksError(BlocksError),
    #[error("not yet parsed `{0}`")]
    NotYetParsed(String),
    #[error("optimizer error - `{0}`")]
    OptimizerError(Box<dyn optimizer::OptimizationError>),
    #[error("files have not been linked")]
    NotLinked,
    #[error("no ir was provided")]
    NoIR,
    #[error("repl error - `{0}`")]
    ReplError(ReplError),
    #[error("macro error in file {0} - `{1}`")]
    MacroError(String, MacroError),
}

#[derive(Copy, Clone)]
pub struct EnaOptions {
    pub debug_gc: bool,
    pub gc: bool,
    pub debug_calls: bool,
    pub debug_stack: bool,
}

impl Default for EnaOptions {
    fn default() -> Self {
        Self {
            debug_gc: false,
            gc: true,
            debug_calls: false,
            debug_stack: false,
        }
    }
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum DocGen {
    JSON,
    Markdown,
    HTML,
}

impl Display for DocGen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocGen::JSON => write!(f, "json"),
            DocGen::Markdown => write!(f, "markdown"),
            DocGen::HTML => write!(f, "html"),
        }
    }
}

pub struct Ena {
    pub tokenizer: compiler::tok::Tokenizer,
    pub ast: compiler::ast::ASTBuilder,
    pub compiler: compiler::irgen::IRGen,
    pub vm: Option<vm::machine::VM>,
    pub files: HashMap<String, String>,
    pub astified_files: HashMap<String, compiler::ast::ASTNode>,
    pub compiled_files: HashMap<String, ir::IR>,
    pub checker: Checker,
    pub optimizer: optimizer::Optimizer,
    pub ir: Option<ir::IR>,
    pub macro_unwrapper: MacroUnwrapper,
}

impl Default for Ena {
    fn default() -> Self {
        Self::new()
    }
}

impl Ena {
    pub fn new() -> Self {
        Self {
            optimizer: optimizer::Optimizer::default(),
            tokenizer: compiler::tok::Tokenizer::new(),
            checker: Checker::default(),
            ast: compiler::ast::ASTBuilder::new(),
            compiler: compiler::irgen::IRGen::new(),
            vm: None,
            files: HashMap::new(),
            astified_files: HashMap::new(),
            compiled_files: HashMap::new(),
            ir: None,
            macro_unwrapper: MacroUnwrapper::default(),
        }
    }

    pub fn generate_doc(&self, generator: DocGen) -> Result<String, EnaError> {
        let gen: Box<dyn docgen::DocRenderer> = match generator {
            DocGen::JSON => Box::new(docgen::renderers::json::JsonRenderer),
            DocGen::Markdown => Box::new(docgen::renderers::md::MarkdownRenderer),
            DocGen::HTML => Box::new(docgen::renderers::html::HtmlRenderer),
        };
        let documentation = docgen::Documentation::from_ir(self.ir.as_ref().unwrap().clone());
        Ok(gen.render(documentation))
    }

    pub fn optimize(&mut self, main: &str) -> Result<(), EnaError> {
        let ir = match &self.ir {
            Some(i) => i,
            None => return Err(EnaError::NoIR),
        };
        self.ir = Some(
            self.optimizer
                .optimize(ir.clone(), &main.to_local_str())
                .map_err(EnaError::OptimizerError)?,
        );
        Ok(())
    }

    pub fn check(&mut self) -> Result<(), EnaError> {
        let blocks = Blocks::new(native::group(), self.ir.as_ref().unwrap().clone())
            .map_err(EnaError::BlocksError)?;
        self.checker.blocks = Some(blocks);
        let vec = self.checker.run_checks(false);
        if !vec.is_empty() {
            Err(EnaError::CheckerErrors(vec))
        } else {
            Ok(())
        }
    }

    fn print_error(
        &self,
        data: &str,
        file: &str,
        line: usize,
        col: usize,
        file_data: &str,
        print_line: bool,
    ) {
        eprintln!(
            "{} {} {}",
            "error".red().bold(),
            format!("in {}:{}:{}:", file, line, col).dimmed(),
            data.bold().bright_white(),
        );
        if print_line {
            eprintln!(
                "\t {} {}",
                format!("{line} |").dimmed(),
                Self::highlight_char_in_string(file_data.lines().nth(line - 1).unwrap(), col - 1)
            );
        }
    }

    pub fn report_error(&self, err: EnaError) {
        match err {
            EnaError::TokenizerError(file, data) => {
                let file_data = self.files.get(&file).unwrap();
                let (line, col) = util::get_line(file_data, data.0);
                self.print_error(&format!("{}", data.1), &file, line, col, file_data, true);
            }
            EnaError::MacroError(file, data) => {
                let file_data = self.files.get(&file).unwrap();
                let (line, col) = util::get_line(file_data, data.get_pos());
                self.print_error(&format!("{}", data), &file, line, col, file_data, true);
            }
            EnaError::ASTError(file, data) => {
                let file_data = self.files.get(&file).unwrap();
                let token = self.tokenizer.tokens.get(data.0).unwrap();
                let (line, col) = util::get_line(file_data, token.0);
                self.print_error(&format!("{}", data.1), &file, line, col, file_data, true);
            }
            EnaError::IRGenError(file, data) => {
                let file_data = self.files.get(&file).unwrap();
                let (line, col) = util::get_line(file_data, data.0 .0);
                self.print_error(&format!("{}", data.1), &file, line, col, file_data, true);
            }
            EnaError::VMError(err) => {
                eprintln!("{red}: {err}", red = "error".red().bold());
                for call in self.vm.as_ref().unwrap().call_stack.iter().rev() {
                    eprintln!("{}", format!("\t^ {call}").dimmed());
                }
            }
            EnaError::CheckerErrors(errs) => {
                for err in errs {
                    self.report_error(EnaError::CheckerError(err));
                }
            }
            EnaError::CheckerError(err) => {
                eprintln!("{red}: {err}", red = "error".red().bold());
            }
            other => eprintln!("{}: {other}", "error".red().bold()),
        }
    }

    pub fn report_error_and_exit(&self, err: EnaError) -> ! {
        self.report_error(err);
        process::exit(1);
    }

    pub fn unwrap_error<T>(&self, err: Result<T, EnaError>) -> T {
        match err {
            Ok(i) => i,
            Err(e) => {
                self.report_error_and_exit(e);
            }
        }
    }

    pub fn read_files(&mut self, paths: &[String]) -> Result<(), EnaError> {
        let unwrapped = Self::read_paths(paths)?;

        for path in unwrapped {
            match fs::read_to_string(&path) {
                Ok(i) => self.files.insert(path.display().to_string(), i),
                Err(e) => {
                    return Err(EnaError::FSError(e.to_string()));
                }
            };
        }

        Ok(())
    }

    pub fn parse_files(&mut self) -> Result<(), EnaError> {
        let files = self.get_keys();

        for name in files {
            self.parse_file(&name)?;
        }

        Ok(())
    }

    pub fn parse_file(&mut self, name: &String) -> Result<(), EnaError> {
        let file = self.files.get(name);
        let file = match file {
            Some(i) => i,
            None => {
                return Err(EnaError::FSError(name.clone()));
            }
        };

        let a = self
            .tokenizer
            .parse(file)
            .map_err(|x| EnaError::TokenizerError(name.clone(), x))?;
        let with_unwrapped_macros = self
            .macro_unwrapper
            .unwrap_macros(a)
            .map_err(|x| EnaError::MacroError(name.clone(), x))?;
        let ast = self
            .ast
            .parse(&with_unwrapped_macros)
            .map_err(|x| EnaError::ASTError(name.clone(), x))?;

        self.tokenizer.clean();
        self.ast.clean();

        self.astified_files.insert(name.clone(), ast);
        Ok(())
    }

    pub fn compile_files(&mut self) -> Result<(), EnaError> {
        let files = self
            .astified_files
            .clone()
            .into_keys()
            .collect::<Vec<String>>();

        for name in files {
            self.compile_file(&name)?;
        }

        Ok(())
    }

    pub fn compile_file(&mut self, name: &String) -> Result<(), EnaError> {
        let data = self.astified_files.get(name);
        let data = match data {
            Some(i) => i,
            None => {
                return Err(EnaError::NotYetParsed(name.clone()));
            }
        };

        let ir = self
            .compiler
            .compile(data)
            .map_err(|x| EnaError::IRGenError(name.clone(), x))?;

        self.compiled_files.insert(name.clone(), ir);

        Ok(())
    }

    pub fn link_files(&mut self) -> Result<(), EnaError> {
        let mut ir = ir::IR::new();

        for sub_ir in self
            .compiled_files
            .clone()
            .into_values()
            .collect::<Vec<ir::IR>>()
        {
            ir.add(&sub_ir).map_err(EnaError::IRError)?;
        }

        self.ir = Some(ir);
        Ok(())
    }

    pub fn save(&self, output: &str) -> Result<(), EnaError> {
        match &self.ir {
            Some(i) => {
                let u8vec = i
                    .into_serializable()
                    .into_vec()
                    .map_err(EnaError::SerializationError)?;
                let mut file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .read(false)
                    .open(output)
                    .map_err(|x| EnaError::FSError(x.to_string()))?;
                file.write_all(&u8vec)
                    .map_err(|x| EnaError::FSError(x.to_string()))
            }
            None => Err(EnaError::NotLinked),
        }
    }

    pub fn load_irs(&mut self, paths: &[String]) -> Result<(), EnaError> {
        let paths = Self::read_paths(paths)?;
        let mut ir = ir::IR::new();

        for path in paths {
            let sub_ir = self.load_ir(path.to_str().unwrap())?;
            ir.add(&sub_ir).unwrap();
        }

        self.ir = Some(ir);

        Ok(())
    }

    pub fn load_ir(&mut self, from: &str) -> Result<ir::IR, EnaError> {
        let mut open_opts = OpenOptions::new()
            .read(true)
            .open(from)
            .map_err(|x| EnaError::FSError(x.to_string()))?;
        let mut v = Vec::<u8>::new();
        open_opts
            .read_to_end(&mut v)
            .map_err(|x| EnaError::FSError(x.to_string()))?;

        let serial = ir::from_vec(&v).map_err(EnaError::SerializationError)?;
        serial.into_ir().map_err(EnaError::SerializationError)
    }

    pub fn run_repl(&mut self, options: VMOptions) -> Result<(), EnaError> {
        let mut repl = Repl::new(VM::new(options));

        repl.run_interactive();
    }

    pub fn display_json(&self, pretty: bool) -> Result<(), EnaError> {
        match &self.ir {
            Some(ir) => {
                let res = if pretty {
                    serde_json::to_string_pretty(ir).unwrap()
                } else {
                    serde_json::to_string(ir).unwrap()
                };
                println!("{}", res);
                Ok(())
            }
            None => Err(EnaError::NotLinked),
        }
    }

    pub fn run(&mut self, main: &str, options: vm::machine::VMOptions) -> Result<(), EnaError> {
        self.vm = Some(vm::machine::VM::new(options));
        let ir = match self.ir {
            Some(ref mut i) => i,
            None => {
                return Err(EnaError::NotLinked);
            }
        };

        let blocks = vm::blocks::Blocks::new(vm::native::group(), ir.clone());
        let blocks = match blocks {
            Ok(blocks) => blocks,
            Err(err) => {
                return Err(EnaError::IRError(err.into()));
            }
        };

        self.vm
            .as_mut()
            .unwrap()
            .run(&main.to_local_str(), blocks)
            .map_err(EnaError::VMError)
            .map(|_| ())
    }

    pub fn run_main(&mut self, options: VMOptions) -> Result<(), EnaError> {
        self.run("main", options)
    }

    pub fn clean(&mut self) {
        self.tokenizer.clean();
        self.ast.clean();
        self.vm = None;
        self.files = HashMap::new();
        self.astified_files = HashMap::new();
        self.compiled_files = HashMap::new();
        self.ir = None;
    }

    fn read_paths(paths: &[String]) -> Result<Vec<PathBuf>, EnaError> {
        let mut unwrapped = Vec::<PathBuf>::new();

        for path in paths {
            let resolved_paths = match glob(path) {
                Ok(i) => i,
                Err(e) => {
                    return Err(EnaError::FailedToReadGlobPattern(format!(
                        "{}: {}",
                        e.pos, e.msg
                    )));
                }
            };

            for resolved_path in resolved_paths {
                match resolved_path {
                    Ok(i) => {
                        unwrapped.push(i);
                    }
                    Err(e) => {
                        return Err(EnaError::FailedToReadGlobPattern(format!(
                            "{}: {}",
                            e.path().display(),
                            e.error()
                        )));
                    }
                };
            }
        }

        for u in &unwrapped {
            if !u.as_path().exists() || u.as_path().is_dir() {
                return Err(EnaError::FSError(format!("file {} not found", u.display())));
            }
        }

        if unwrapped.is_empty() {
            return Err(EnaError::FSError("files not found".to_string()));
        }

        Ok(unwrapped)
    }

    fn highlight_char_in_string(initial: &str, at: usize) -> String {
        let start: String = initial.chars().take(at).collect();
        let end: String = initial.chars().skip(at + 1).collect();
        let ch = String::from(initial.chars().nth(at).unwrap_or('\0'));

        format!("{}{}{}", start, ch.bold().red(), end)
    }

    fn get_keys(&self) -> Vec<String> {
        // some cloning and evil shenanigans are needed to trick borrow checker
        // todo: redo this
        self.files.clone().into_keys().collect()
    }
}
