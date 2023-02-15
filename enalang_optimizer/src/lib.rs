use std::{error::Error, fmt::Debug};

pub mod optimizations;

#[derive(Clone)]
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
        Self::default()
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
