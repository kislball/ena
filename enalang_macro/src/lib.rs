use enalang_compiler::tok::{Token, TokenInner};
use flexstr::{local_str, LocalStr, ToLocalStr};
use macros::{class::ClassMacro, id::IdMacro, log::LogMacro, define::DefineMacro};
use std::{cell::RefCell, collections::HashMap, error::Error, rc::Rc};

pub mod macros;

#[derive(Debug, thiserror::Error)]
pub enum MacroError {
    #[error("unknown macro {name} at {at}")]
    UnknownMacro { name: LocalStr, at: usize },
    #[error("{name} error: {error} at {at}")]
    InternalMacroError {
        name: LocalStr,
        error: Box<dyn Error>,
        at: usize,
    },
}

impl MacroError {
    pub fn get_pos(&self) -> usize {
        match self {
            MacroError::UnknownMacro { at, .. } => *at,
            MacroError::InternalMacroError { at, .. } => *at,
        }
    }
}

pub struct MacroUnwrapper {
    macros: HashMap<LocalStr, Rc<RefCell<dyn Macro>>>,
}

impl Default for MacroUnwrapper {
    fn default() -> Self {
        let mut s = Self::new();
        s.add_macro(local_str!("log"), LogMacro);
        s.add_macro(local_str!("class"), ClassMacro);
        s.add_macro(local_str!("define"), DefineMacro);
        s.add_macro(local_str!("id"), IdMacro);

        return s;
    }
}

impl MacroUnwrapper {
    pub fn new() -> Self {
        Self {
            macros: HashMap::default(),
        }
    }

    pub fn add_macro(&mut self, name: LocalStr, m: impl Macro + 'static) {
        self.macros.insert(name, Rc::new(RefCell::new(m)));
    }

    pub fn add_rc_macro(&mut self, name: LocalStr, m: Rc<RefCell<dyn Macro>>) {
        self.macros.insert(name, m);
    }

    pub fn clear(&self) {
        for v in self.macros.values() {
            v.borrow_mut().clear();
        }
    }

    pub fn unwrap_macros(&mut self, tokens: &[Token]) -> Result<Vec<Token>, MacroError> {
        let mut out = Vec::new();
        for token in tokens {
            if let Token(n, TokenInner::Comment(text)) = token.clone() {
                let macro_name = text.split(' ').nth(0).unwrap().strip_prefix('#');
                if let Some(macro_name) = macro_name {
                    let unwrapper = self.macros.get(&macro_name.to_local_str()).ok_or(
                        MacroError::UnknownMacro {
                            name: macro_name.to_local_str(),
                            at: n,
                        },
                    )?;
                    let without_name: String = text
                        .split(' ')
                        .skip(1)
                        .fold(String::new(), |a, b| format!("{a} {b}"));
                    let macroed = unwrapper
                        .clone()
                        .borrow_mut()
                        .unwrap(without_name.trim(), self)
                        .map_err(|x| match x {
                            MacroError::InternalMacroError { name, error, at } => {
                                MacroError::InternalMacroError {
                                    name,
                                    error,
                                    at: at + n,
                                }
                            }
                            x => x,
                        })?;
                    out.extend(macroed);
                } else {
                    out.push(token.clone());
                }
            } else {
                out.push(token.clone());
            }
        }

        return Ok(out);
    }
}

pub trait Macro {
    fn unwrap(
        &mut self,
        tokens: &str,
        unwrapper: &mut MacroUnwrapper,
    ) -> Result<Vec<Token>, MacroError>;
    fn clear(&mut self) {}
}
