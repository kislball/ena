use crate::{Macro, MacroError, MacroUnwrapper};
use colored::Colorize;
use enalang_compiler::tok::Token;

pub struct LogMacro;

impl Macro for LogMacro {
    fn unwrap(&mut self, tokens: &str, _: &mut MacroUnwrapper) -> Result<Vec<Token>, MacroError> {
        println!("{}: {tokens}", "log".bold().bright_blue());
        Ok(vec![])
    }
}
