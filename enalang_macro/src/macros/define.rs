use crate::{Macro, MacroError};
use enalang_compiler::tok::{Token, TokenInner, Tokenizer};
use flexstr::{local_str, LocalStr};

pub struct DefineMacro;

#[derive(thiserror::Error, Debug)]
pub enum DefineMacroError {
    #[error("wanted at least {wanted} arguments, got - {got}")]
    ArgumentCount { wanted: usize, got: usize },
    #[error("wanted {wanted} at argument {at}")]
    ArgumentTypeError { wanted: LocalStr, at: usize },
}

impl Macro for DefineMacro {
    fn unwrap(
        &mut self,
        tokens: &str,
        unwrapper: &mut crate::MacroUnwrapper,
    ) -> Result<Vec<Token>, MacroError> {
        let mut tokenizer = Tokenizer::default();
        let mut tokens =
            tokenizer
                .parse(tokens)
                .cloned()
                .map_err(|x| MacroError::InternalMacroError {
                    name: local_str!("define"),
                    error: Box::new(x.1),
                    at: x.0,
                })?;
        if tokens.len() < 3 {
            return Err(MacroError::InternalMacroError {
                name: local_str!(""),
                error: Box::new(DefineMacroError::ArgumentCount {
                    wanted: 3,
                    got: tokens.len(),
                }),
                at: 0,
            });
        };
        let name = if let Token(_, TokenInner::Identifier(id)) = tokens.remove(0) {
            id
        } else {
            return Err(MacroError::InternalMacroError {
                name: local_str!("define"),
                error: Box::new(DefineMacroError::ArgumentTypeError {
                    wanted: local_str!("string"),
                    at: 0,
                }),
                at: 0,
            });
        };
        let arg_count = if let Token(_, TokenInner::Number(id)) = tokens.remove(0) {
            id as usize
        } else {
            return Err(MacroError::InternalMacroError {
                name: local_str!("define"),
                error: Box::new(DefineMacroError::ArgumentTypeError {
                    wanted: local_str!("number"),
                    at: 0,
                }),
                at: 0,
            });
        };

        unwrapper.add_macro(
            name.clone().into(),
            UserMacro {
                arg_count,
                name,
                tokens,
            },
        );

        Ok(vec![])
    }
}

#[derive(Debug)]
pub struct UserMacro {
    pub arg_count: usize,
    pub tokens: Vec<Token>,
    pub name: String,
}

#[derive(thiserror::Error, Debug)]
pub enum UserMacroError {
    #[error("wanted {wanted} arguments, got - {got}")]
    ArgumentCount { wanted: usize, got: usize },
}

impl Macro for UserMacro {
    fn unwrap(
        &mut self,
        tokens: &str,
        _: &mut crate::MacroUnwrapper,
    ) -> Result<Vec<Token>, crate::MacroError> {
        let mut tokenizer = Tokenizer::default();
        let tokens = tokenizer
            .parse(tokens)
            .map_err(|x| MacroError::InternalMacroError {
                name: self.name.clone().into(),
                error: Box::new(x.1),
                at: x.0,
            })
            .cloned()?;
        if tokens.len() != self.arg_count {
            return Err(MacroError::InternalMacroError {
                name: self.name.clone().into(),
                error: Box::new(UserMacroError::ArgumentCount {
                    wanted: self.arg_count,
                    got: tokens.len(),
                }),
                at: 0,
            });
        }

        let replaced = self
            .tokens
            .iter()
            .cloned()
            .map(|x| match x {
                Token(at, TokenInner::Identifier(id)) => {
                    if let Some(prefix) = id.strip_prefix('%') {
                        if let Ok(i) = prefix.parse::<usize>() {
                            if let Some(token) = tokens.get(i).cloned() {
                                token
                            } else {
                                Token(at, TokenInner::Identifier(id))
                            }
                        } else {
                            Token(at, TokenInner::Identifier(id))
                        }
                    } else {
                        Token(at, TokenInner::Identifier(id))
                    }
                }
                other => other,
            })
            .collect();

        Ok(replaced)
    }
}
