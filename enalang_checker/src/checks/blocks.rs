use crate::checker::{Check, CheckContext, CheckError};
use enalang_compiler::ir::{Block, IRCode, Value};
use enalang_vm::{
    blocks::VMBlock,
    machine::{ScopeManager, VMError},
};
use flexstr::LocalStr;

#[derive(Debug)]
pub enum BlocksCheckerError {
    UnknownBlock(LocalStr, LocalStr),
    CannotShadowBlocksInLocalScope(LocalStr, LocalStr),
    VM(VMError),
}

impl CheckError for BlocksCheckerError {
    fn explain(&self) -> String {
        match self {
            Self::UnknownBlock(a, b) => format!("unknown block {a} in {b}"),
            Self::CannotShadowBlocksInLocalScope(a, b) => format!("cannot shadow {a} in {b}"),
            Self::VM(e) => format!("{e:?}"),
        }
    }
}

pub struct BlocksChecker {}

impl BlocksChecker {
    pub fn new() -> Self {
        Self {}
    }

    fn check_block(
        &self,
        name: LocalStr,
        block: &Block,
        scope_manager: &mut ScopeManager,
    ) -> Result<(), Box<dyn CheckError>> {
        for op in &block.code {
            if let IRCode::LocalBlock(sub_name, typ, data) = op {
                scope_manager.add_local(sub_name.clone()).map_err(|_| {
                    Box::new(BlocksCheckerError::CannotShadowBlocksInLocalScope(
                        sub_name.clone(),
                        name.clone(),
                    ))
                })?;
                let block = Block {
                    code: data.clone(),
                    global: false,
                    run_type: *typ,
                };
                scope_manager
                    .blocks_mut()
                    .add_block(
                        sub_name.clone(),
                        VMBlock::IR(block.clone()),
                    )
                    .map_err(|_| {
                        Box::new(BlocksCheckerError::CannotShadowBlocksInLocalScope(
                            sub_name.clone(),
                            name.clone(),
                        ))
                    })?;
                self.check_block(sub_name.clone(), &block, scope_manager)?;
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
            if let None = scope_manager.blocks().get_block(sub) {
                return Err(Box::new(BlocksCheckerError::UnknownBlock(
                    sub.clone(),
                    name.clone(),
                )));
            }
        }
        Ok(())
    }
}

impl Check for BlocksChecker {
    fn check(&self, mut ctx: CheckContext) -> Result<(), Box<dyn CheckError>> {
        for (name, block) in &ctx.blocks.blocks {
            if !block.is_global() {
                continue;
            }

            if let VMBlock::NativeHandler(_) = block {
                continue;
            }

            ctx.scope_manager
                .root(ctx.blocks.clone(), name.clone())
                .map_err(|x| Box::new(BlocksCheckerError::VM(x)))?;

            let block = match block {
                VMBlock::NativeHandler(_) => panic!("unreachable"),
                VMBlock::IR(ir) => ir,
            };

            self.check_block(name.clone(), block, &mut ctx.scope_manager)?;
        }

        Ok(())
    }

    fn is_independent(&self) -> bool {
        false
    }
}
