use enalang_compiler::{
    ast::{ASTBuilder, ASTError},
    irgen::{IRGen, IRGenError},
    tok::{Tokenizer, TokenizerError},
};
use enalang_ir::IR;
use enalang_vm::{
    blocks::{self, BlocksError},
    machine::{VMError, VM},
    native,
};
use flexstr::{local_fmt, local_str, IntoLocalStr};
use rand::distributions::{Alphanumeric, DistString};
use std::{
    io::{self, stdout, Write},
    process,
};

pub const DEFAULT_OUTPUT_LENGTH: usize = 10;
const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

pub fn get_startup_message() -> String {
    format!(
        "ENA {}\nExit using :exit or ctrl+c, type \":help\" for help.",
        VERSION.unwrap_or("unknown")
    )
}

pub fn get_repl_help() -> String {
    format!(
        r#"REPL help:
:help, :h - print help
:history <n>, :hist <n> - print <n> records from history. default - {DEFAULT_OUTPUT_LENGTH}
:stack <n>, :s <n> - print top <n> entries in stack. default - {DEFAULT_OUTPUT_LENGTH}
:exit, :e - exit

If you do not enter a command, the input will be compiled to Ena bytecode and interpreted."#
    )
}

pub struct Repl {
    pub vm: VM,
    pub history: Vec<String>,
}

impl Default for Repl {
    fn default() -> Self {
        Self::new(VM::default())
    }
}

#[derive(Debug, Clone)]
pub enum ReplCommand {
    ShowStack(usize),
    ShowHistory(usize),
    RunBlock(String),
    ShowHelp,
    Exit,
}

#[derive(Debug, thiserror::Error)]
pub enum CommandParseError {
    #[error("unknown command `{0}`")]
    UnknownCommand(String),
    #[error("expected number at argument no. {0}")]
    ExpectedNumber(u32),
}

#[derive(Debug, thiserror::Error)]
pub enum ReplError {
    #[error("vm - {0}")]
    VMError(VMError),
    #[error("ast - {0}")]
    ASTError(ASTError),
    #[error("tokenizer - {0}")]
    TokenizerError(TokenizerError),
    #[error("ir - {0}")]
    IRGenError(IRGenError),
    #[error("block error - {0}")]
    BlocksError(BlocksError),
    #[error("io - {0}")]
    IOError(io::Error),
}

impl Repl {
    pub fn new(vm: VM) -> Self {
        Self {
            vm,
            history: Vec::new(),
        }
    }

    pub fn run_interactive(&mut self) -> ! {
        println!("{}", get_startup_message());
        self.vm
            .run(
                &local_str!("nop"),
                blocks::Blocks::new(native::group(), IR::default()).unwrap(),
            )
            .unwrap();
        let mut s = String::new();
        loop {
            print!(">>> ");
            stdout().flush().unwrap();
            s.clear();
            let r = io::stdin().read_line(&mut s);
            s = s.replace("\n", "");
            self.history.push(s.clone());
            if let Err(e) = r {
                println!("io error - {e}");
                continue;
            }
            let cmd = match Self::parse_command(&s) {
                Ok(e) => e,
                Err(err) => {
                    println!("{err}");
                    continue;
                }
            };

            if let Err(e) = self.run_command(&cmd) {
                println!("{e}");
            }
        }
    }

    pub fn run_command(&mut self, cmd: &ReplCommand) -> Result<(), ReplError> {
        match cmd {
            ReplCommand::ShowStack(i) => {
                if self.vm.stack.len() == 0 {
                    println!("<stack empty>");
                } else {
                    for (n, i) in self.vm.stack.iter().rev().take(*i).enumerate() {
                        println!("{n}. {i:?}", n = n + 1);
                    }
                }
            }
            ReplCommand::Exit => process::exit(0),
            ReplCommand::ShowHistory(i) => {
                if self.history.len() == 0 {
                    println!("<history empty>");
                } else {
                    for (n, i) in self.history.iter().take(*i).enumerate() {
                        println!("{n}. {i}", n = n + 1);
                    }
                }
            }
            ReplCommand::RunBlock(code) => {
                let rand = Alphanumeric
                    .sample_string(&mut rand::thread_rng(), 12)
                    .into_local_str();
                let block_name = local_fmt!("repl_{rand}");
                let code = format!("{block_name} {{ {code} }}");

                let mut tokenizer = Tokenizer::new();
                let tokens = tokenizer.parse(&code).map_err(ReplError::TokenizerError)?;
                let mut ast = ASTBuilder::new();
                let tree = ast.parse(tokens).map_err(ReplError::ASTError)?;
                let mut compiler = IRGen::new();
                let ir = compiler.compile(&tree).map_err(ReplError::IRGenError)?;

                self.vm
                    .scope_manager
                    .blocks_mut()
                    .add_ir(ir)
                    .map_err(ReplError::BlocksError)?;

                return self
                    .vm
                    .run_block(&block_name)
                    .map(|_| ())
                    .map_err(ReplError::VMError);
            }
            ReplCommand::ShowHelp => {
                println!("{}", get_repl_help())
            }
        }

        Ok(())
    }

    pub fn parse_command(cmd: &str) -> Result<ReplCommand, CommandParseError> {
        if let Some(cmd_name) = cmd.replace("\n", "").strip_prefix(':') {
            let arg = cmd_name
                .split(' ')
                .nth(1)
                .map(|x| x.parse::<usize>().unwrap_or(DEFAULT_OUTPUT_LENGTH))
                .unwrap_or(DEFAULT_OUTPUT_LENGTH);
            match cmd_name {
                "hist" | "history" => Ok(ReplCommand::ShowHistory(arg)),
                "h" | "help" => Ok(ReplCommand::ShowHelp),
                "s" | "stack" => Ok(ReplCommand::ShowStack(arg)),
                "e" | "exit" => Ok(ReplCommand::Exit),
                _ => Err(CommandParseError::UnknownCommand(cmd_name.to_string())),
            }
        } else {
            Ok(ReplCommand::RunBlock(cmd.to_string()))
        }
    }
}
