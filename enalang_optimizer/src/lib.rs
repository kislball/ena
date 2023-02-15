use std::{error::Error, fmt::Debug};

use optimizations::inline::InlineOptimization;

pub mod optimizations;

#[derive(Clone, Default)]
pub struct OptimizationContext {
    pub ir: enalang_ir::IR,
}

pub trait Optimization {
    fn optimize(
        &mut self,
        ctx: OptimizationContext,
    ) -> Result<enalang_ir::IR, Box<dyn OptimizationError>>;
}

pub trait OptimizationError: Debug + Error {
    fn from(&self) -> Option<String>;
}

impl<T: OptimizationError + 'static> From<Box<T>> for Box<dyn OptimizationError> {
    fn from(value: Box<T>) -> Self {
        value
    }
}

#[derive(Default)]
pub struct Optimizer {
    pub optimizations: Vec<Box<dyn Optimization>>,
}

impl Optimizer {
    pub fn new() -> Self {
        let mut opt = Self::default();

        opt.optimizations.push(Box::new(InlineOptimization::new()));

        opt
    }

    pub fn optimize(
        &mut self,
        mut code: enalang_ir::IR,
    ) -> Result<enalang_ir::IR, Box<dyn OptimizationError>> {
        for optimization in &mut self.optimizations {
            code = optimization.optimize(OptimizationContext { ir: code.clone() })?;
        }
        Ok(code)
    }
}
