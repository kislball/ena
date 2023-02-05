use colored::Colorize;
use flexstr::ToLocalStr;
use glob::glob;
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
    process,
};

pub mod ast;
pub mod compiler;
pub mod ir;
pub mod tok;
pub mod util;
pub mod vm;

#[derive(Debug)]
pub enum EnaError {
    TokenizerError(String, tok::TokenizerError),
    ASTError(String, ast::ASTError),
    CompilerError(String, compiler::CompilerError),
    IRError(ir::IRError),
    SerializationError(ir::SerializationError),
    VMError(vm::machine::VMError),
    FailedToReadGlobPattern(String),
    FSError(String),
    NotYetParsed(String),
    NotLinked,
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

pub struct Ena {
    tokenizer: tok::Tokenizer,
    ast: ast::ASTBuilder,
    compiler: compiler::Compiler,
    vm: vm::machine::VM,
    files: HashMap<String, String>,
    astified_files: HashMap<String, ast::ASTNode>,
    compiled_files: HashMap<String, ir::IR>,
    pub ir: Option<ir::IR>,
}

impl Ena {
    pub fn new(options: EnaOptions) -> Self {
        Self {
            tokenizer: tok::Tokenizer::new(),
            ast: ast::ASTBuilder::new(),
            compiler: compiler::Compiler::new(),
            vm: vm::machine::VM::new(vm::machine::VMOptions {
                debug_gc: options.debug_gc,
                enable_gc: options.gc,
                debug_stack: options.debug_stack,
                debug_calls: options.debug_calls,
            }),
            files: HashMap::new(),
            astified_files: HashMap::new(),
            compiled_files: HashMap::new(),
            ir: None,
        }
    }

    pub fn report_error(&self, err: EnaError) {
        match err {
            EnaError::TokenizerError(file, data) => {
                let file_data = self.files.get(&file).unwrap();
                let (line, col) = util::get_line(file_data, data.0);
                eprintln!(
                    "{} in {}:{}:{}: {:?}",
                    "error".red().bold(),
                    file,
                    line,
                    col,
                    data.1,
                );
                eprintln!(
                    "\t {} {}",
                    format!("{line} |").dimmed(),
                    Self::highlight_char_in_string(
                        file_data.lines().nth(line - 1).unwrap(),
                        col - 1
                    )
                );
            }
            EnaError::ASTError(file, data) => {
                let file_data = self.files.get(&file).unwrap();
                let token = self.tokenizer.tokens.get(data.0).unwrap();
                let (line, col) = util::get_line(file_data, token.0);
                eprintln!(
                    "{} in {}:{}:{}: {:?}",
                    "error".red().bold(),
                    file,
                    line,
                    col,
                    data.1,
                );
                eprintln!(
                    "\t {} {}",
                    format!("{line} |").dimmed(),
                    Self::highlight_char_in_string(
                        file_data.lines().nth(line - 1).unwrap(),
                        col - 1
                    )
                );
            }
            EnaError::CompilerError(file, data) => {
                let file_data = self.files.get(&file).unwrap();
                let (line, col) = util::get_line(file_data, data.0 .0);
                eprintln!(
                    "{} in {}:{}:{}: {:?}",
                    "error".red().bold(),
                    file,
                    line,
                    col,
                    data.1,
                );
                eprintln!(
                    "\t {} {}",
                    format!("{line} |").dimmed(),
                    Self::highlight_char_in_string(
                        file_data.lines().nth(line - 1).unwrap(),
                        col - 1
                    )
                );
            }
            EnaError::VMError(err) => {
                eprintln!("{}: {:?}\n\tcall stack:", "error".red().bold(), err);
                for call in &self.vm.call_stack {
                    eprintln!("{}", format!("\t\t- {call}").dimmed());
                }
            }
            other => eprintln!("{}: {other:?}", "error".red().bold()),
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

        self.tokenizer
            .parse(file)
            .map_err(|x| EnaError::TokenizerError(name.clone(), x))?;
        let ast = self
            .ast
            .parse(&self.tokenizer.tokens)
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
            .map_err(|x| EnaError::CompilerError(name.clone(), x))?;

        self.compiled_files.insert(name.clone(), ir);

        Ok(())
    }

    pub fn link_files(&mut self) -> Result<(), EnaError> {
        let mut ir =ir::IR::new();

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

    pub fn run(&mut self, main: &str) -> Result<(), EnaError> {
        let ir = match self.ir {
            Some(ref mut i) => {
                vm::native::group().apply(i).unwrap();
                i
            }
            None => {
                return Err(EnaError::NotLinked);
            }
        };
        self.vm
            .run(&main.to_local_str(), ir.clone())
            .map_err(EnaError::VMError)
            .map(|_| ())
    }

    pub fn run_main(&mut self) -> Result<(), EnaError> {
        self.run("main")
    }

    pub fn clean(&mut self) {
        self.tokenizer.clean();
        self.ast.clean();
        self.vm.clean();
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
        let ch = String::from(initial.chars().nth(at).unwrap());

        format!("{}{}{}", start, ch.bold().red(), end)
    }

    fn get_keys(&self) -> Vec<String> {
        // some cloning and evil shenanigans are needed to trick borrow checker
        // todo: redo this
        self.files.clone().into_keys().collect()
    }
}
