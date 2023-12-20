use crate::{Macro, MacroError, MacroUnwrapper};
use enalang_compiler::tok::{Token, TokenInner};
use flexstr::local_str;

#[derive(Debug, thiserror::Error)]
pub enum ClassMacroError {
    #[error("wanted {wanted} arguments, got - {got}")]
    TooFewArguments { wanted: usize, got: usize },
}

pub struct ClassMacro;

impl Macro for ClassMacro {
    fn unwrap(&mut self, tokens: &str, _: &mut MacroUnwrapper) -> Result<Vec<Token>, MacroError> {
        let tokens: Vec<&str> = tokens.split(' ').collect();
        let mut res = Vec::<TokenInner>::new();
        if tokens.len() < 2 {
            return Err(MacroError::InternalMacroError {
                name: local_str!("class"),
                error: Box::new(ClassMacroError::TooFewArguments {
                    wanted: 2,
                    got: tokens.len(),
                }),
                at: 0,
            });
        }
        let name = tokens[0];
        let fields = &tokens[1..];

        res.extend(vec![
            TokenInner::Identifier(format!("{name}.allocate")),
            TokenInner::UniqueOpen,
            TokenInner::Number(fields.len() as f64),
            TokenInner::Identifier("alloc".into()),
            TokenInner::UniqueClose,
        ]);

        for (n, field) in fields.iter().enumerate() {
            res.extend(vec![
                TokenInner::Identifier(format!("{name}.{field}")),
                TokenInner::UniqueOpen,
                TokenInner::Number(n as f64),
                TokenInner::Identifier("+".into()),
                TokenInner::Identifier("@".into()),
                TokenInner::UniqueClose,
            ]);

            res.extend(vec![
                TokenInner::Identifier(format!("{name}.{field}=")),
                TokenInner::UniqueOpen,
                TokenInner::Identifier("swap".into()),
                TokenInner::Number(n as f64),
                TokenInner::Identifier("+".into()),
                TokenInner::Identifier("=".into()),
                TokenInner::UniqueClose,
            ]);
        }

        Ok(res
            .into_iter()
            .enumerate()
            .map(|(n, i)| Token(n, i))
            .collect())
    }
}
