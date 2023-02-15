use crate::{Optimization, OptimizationContext};
use enalang_ir::{Block, BlockRunType, IRCode, IRError};
use enalang_vm::{
    blocks::{Blocks, BlocksError, VMBlock},
    machine::{ScopeManager, VMError},
    native,
};
use flexstr::{local_str, LocalStr, ToLocalStr};

pub struct InlineOptimization {
    ctx: OptimizationContext,
    optimized: Vec<LocalStr>,
    scope_manager: ScopeManager,
}

impl InlineOptimization {
    pub fn new() -> Self {
        Self {
            ctx: OptimizationContext::default(),
            optimized: Vec::new(),
            scope_manager: ScopeManager::new(),
        }
    }

    fn can_inline(&self, name: &LocalStr) -> bool {
        if self
            .ctx
            .ir
            .has_directive(&name.clone(), &"@unsafe(inline)".to_local_str())
        {
            return true;
        }

        if self
            .ctx
            .ir
            .has_directive(&name.clone(), &"@no-inline".to_local_str())
        {
            return false;
        }

        let block = self.ctx.ir.get_block(name);
        let block = match block {
            Some(i) => i,
            None => {
                return false;
            }
        };

        if let BlockRunType::Once = block.run_type {
            return false;
        }

        for code in &block.code {
            match code {
                IRCode::LocalBlock(_, _, _) | IRCode::ReturnLocal | IRCode::Return => {
                    return false;
                }
                _ => {}
            };
        }

        true
    }

    fn optimize_block(
        &mut self,
        name: &LocalStr,
        block: &Block,
    ) -> Result<Block, Box<dyn crate::OptimizationError>> {
        let mut new_block = Block {
            global: block.global,
            run_type: block.run_type,
            code: Vec::new(),
        };

        if block.global {
            self.scope_manager
                .parent(name.clone())
                .map_err(|x| Box::new(InlineOptimizationError::VM(x)))?;
        } else {
            self.scope_manager
                .child(name.clone())
                .map_err(|x| Box::new(InlineOptimizationError::VM(x)))?;
        }

        for code in &block.code {
            match code {
                IRCode::PutValue(_)
                | IRCode::While(_)
                | IRCode::If(_)
                | IRCode::Return
                | IRCode::ReturnLocal => {
                    new_block.code.push(code.clone());
                }
                IRCode::LocalBlock(name, _, _) => {
                    self.scope_manager
                        .add_local(name.clone())
                        .map_err(|x| Box::new(InlineOptimizationError::VM(x)))?;
                    new_block.code.push(code.clone());
                }
                IRCode::Call(block_name) => {
                    if self.can_inline(block_name) {
                        let block_to_be_inlined = self.scope_manager.blocks().get_block(block_name);
                        let block_to_be_inlined = match block_to_be_inlined {
                            Some(i) => i,
                            None => {
                                return Err(Box::new(InlineOptimizationError::UnknownBlock(
                                    block_name.clone(),
                                )));
                            }
                        };

                        if let VMBlock::IR(ir_block) = block_to_be_inlined {
                            for sub_code in &ir_block.code {
                                new_block.code.push(sub_code.clone());
                            }
                        }
                    }
                }
            };
        }

        self.scope_manager
            .pop_scope()
            .map_err(|x| Box::new(InlineOptimizationError::VM(x)))?;
        Ok(new_block)
    }
}

impl Optimization for InlineOptimization {
    fn optimize(
        &mut self,
        ctx: OptimizationContext,
    ) -> Result<enalang_ir::IR, Box<dyn crate::OptimizationError>> {
        self.ctx = ctx;
        self.optimized = vec![];
        self.scope_manager = ScopeManager::new();
        self.scope_manager
            .root(
                Blocks::new(native::group(), self.ctx.ir.clone())
                    .map_err(|x| Box::new(InlineOptimizationError::Blocks(x)))?,
                local_str!("root"),
            )
            .map_err(|x| Box::new(InlineOptimizationError::VM(x)))?;

        let mut new_ir = enalang_ir::IR::new();
        for block in &self.ctx.clone().ir.blocks {
            if self.optimized.contains(&block.0.clone()) {
                continue;
            }
            let opt_block = self.optimize_block(block.0, block.1)?;
            self.optimized.push(block.0.clone());
            new_ir
                .add_block(block.0.clone(), opt_block, true)
                .map_err(|x| Box::new(InlineOptimizationError::IR(x)))?;
        }

        Ok(new_ir)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InlineOptimizationError {
    #[error("blocks error - `{0}`")]
    Blocks(BlocksError),
    #[error("vm error - `{0}`")]
    VM(VMError),
    #[error("ir error - `{0}`")]
    IR(IRError),
    #[error("unknown block `{0}`")]
    UnknownBlock(LocalStr),
}

impl crate::OptimizationError for InlineOptimizationError {
    fn from(&self) -> Option<String> {
        Some(String::from("inline"))
    }
}
