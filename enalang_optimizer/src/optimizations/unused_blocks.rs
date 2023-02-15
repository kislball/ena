use crate::{Optimization, OptimizationContext, OptimizationError};

#[derive(Debug, thiserror::Error)]
pub enum UnusedBlocksOptimizationError {}

impl OptimizationError for UnusedBlocksOptimizationError {
    fn from(&self) -> Option<String> {
        Some(String::from("unused blocks"))
    }
}

pub struct UnusedBlocksOptimization {
    ctx: OptimizationContext,
}

impl UnusedBlocksOptimization {}

impl Optimization for UnusedBlocksOptimization {
    fn optimize(
        &mut self,
        ctx: crate::OptimizationContext,
    ) -> Result<enalang_ir::IR, Box<dyn crate::OptimizationError>> {
        Ok(ctx.ir)
    }
}
