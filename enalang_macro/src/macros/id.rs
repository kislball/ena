use crate::{Macro, MacroError, MacroUnwrapper};
use enalang_compiler::tok::{Token, TokenInner};

pub struct IdMacro;

impl Macro for IdMacro {
    fn unwrap(&mut self, tokens: &str, _: &mut MacroUnwrapper) -> Result<Vec<Token>, MacroError> {
        Ok(vec![Token(0, TokenInner::Identifier(tokens.into()))])
    }
}
