use crate::checker::{Check, CheckContext, CheckError};
use enalang_compiler::ir::{Block, IRCode, Value};
use enalang_vm::{
    blocks::VMBlock,
    machine::{ScopeManager, VMError},
};
use flexstr::LocalStr;

#[derive(Debug, thiserror::Error)]
pub enum BlocksCheckerError {
    #[error("unknown block `{0}` in `{1}`")]
    UnknownBlock(LocalStr, LocalStr),
    #[error("cannot shadow `{0}` in `{1}`")]
    CannotShadowBlocksInLocalScope(LocalStr, LocalStr),
    #[error("vm error - `{0}`")]
    VM(VMError),
}

impl CheckError for BlocksCheckerError {
    fn from(&self) -> Option<String> {
        match self {
            Self::UnknownBlock(_, b) => Some(b.to_string()),
            Self::CannotShadowBlocksInLocalScope(_, b) => Some(b.to_string()),
            Self::VM(_) => None,
        }
    }
}

pub struct BlocksChecker {}

impl Default for BlocksChecker {
    fn default() -> Self {
        Self {}
    }
}

impl BlocksChecker {
    pub fn new() -> Self {
        Self {}
    }

    fn check_block(
        &self,
        name: LocalStr,
        block: &Block,
        scope_manager: &mut ScopeManager,
    ) -> Result<(), Vec<Box<dyn CheckError>>> {
        let mut errs: Vec<Box<dyn CheckError>> = Default::default();
        for op in &block.code {
            if let IRCode::LocalBlock(sub_name, typ, data) = op {
                let r = scope_manager.add_local(sub_name.clone()).map_err(|_| {
                    Box::new(BlocksCheckerError::CannotShadowBlocksInLocalScope(
                        sub_name.clone(),
                        name.clone(),
                    ))
                });

                if let Err(e) = r {
                    errs.push(e);
                    continue;
                }

                let block = Block {
                    code: data.clone(),
                    global: false,
                    run_type: *typ,
                };
                let e = scope_manager
                    .blocks_mut()
                    .add_block(sub_name.clone(), VMBlock::IR(block.clone()))
                    .map_err(|_| {
                        Box::new(BlocksCheckerError::CannotShadowBlocksInLocalScope(
                            sub_name.clone(),
                            name.clone(),
                        ))
                    });
                if let Err(err) = e {
                    errs.push(err);
                    continue;
                }
                if let Err(mut err) = self.check_block(sub_name.clone(), &block, scope_manager) {
                    errs.append(&mut err);
                }
                continue;
            }

            let sub = match op {
                IRCode::Call(i) => i,
                IRCode::If(i) => i,
                IRCode::PutValue(Value::Block(i)) => i,
                IRCode::While(i) => i,
                _ => {
                    continue;
                }
            };

            let block_sub = scope_manager.blocks().get_block(sub);
            if block_sub.is_none() {
                errs.push(Box::new(BlocksCheckerError::UnknownBlock(
                    sub.clone(),
                    name.clone(),
                )));
                continue;
            }

            let block_sub = block_sub.cloned().unwrap();
            let block_sub = match block_sub {
                VMBlock::IR(b) => b,
                _ => {
                    continue;
                }
            };

            if let Err(mut e) = self.check_block(sub.clone(), &block_sub, scope_manager) {
                errs.append(&mut e);
            }
        }
        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs)
        }
    }
}

impl Check for BlocksChecker {
    fn check(&self, mut ctx: CheckContext) -> Result<(), Vec<Box<dyn CheckError>>> {
        let mut errs: Vec<Box<dyn CheckError>> = vec![];
        for (name, block) in &ctx.blocks.blocks {
            if !block.is_global() {
                continue;
            }

            if let VMBlock::NativeHandler(_) = block {
                continue;
            }

            let err = ctx
                .scope_manager
                .root(ctx.blocks.clone(), name.clone())
                .map_err(|x| Box::new(BlocksCheckerError::VM(x)));
            if let Err(e) = err {
                errs.push(e);
                continue;
            }

            let block = match block {
                VMBlock::NativeHandler(_) => panic!("unreachable"),
                VMBlock::IR(ir) => ir,
            };

            if let Err(mut e) = self.check_block(name.clone(), block, &mut ctx.scope_manager) {
                errs.append(&mut e);
            }
        }

        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs)
        }
    }

    fn is_independent(&self) -> bool {
        false
    }
}
