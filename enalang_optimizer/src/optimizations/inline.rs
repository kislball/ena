use crate::{Optimization, OptimizationContext};
use enalang_ir::{Block, BlockRunType, IRCode};
use flexstr::{LocalStr, ToLocalStr};

pub struct InlineOptimization {
    ctx: OptimizationContext,
}

impl InlineOptimization {
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
                IRCode::If(if_block) => {
                    if !self.can_inline(&if_block) {
                        return false;
                    }
                }
                IRCode::While(while_block) => {
                    if !self.can_inline(&while_block) {
                        return false;
                    }
                }
                _ => {}
            };
        }

        true
    }
}

impl Optimization for InlineOptimization {
    fn optimize(
        &mut self,
        ctx: OptimizationContext,
    ) -> Result<enalang_ir::IR, Box<dyn crate::OptimizationError>> {
        let mut new_ir = enalang_ir::IR::new();
        let mut scope_manager = enalang_vm::machine::ScopeManager::new();
        new_ir.annotations = ctx.ir.annotations.clone();
        self.ctx = ctx;

        for block in &self.ctx.ir.blocks {
            // TODO: implement inlining
        }

        Ok(new_ir)
    }
}
