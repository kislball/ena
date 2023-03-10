use flexstr::{shared_str, SharedStr};
use optimizations::inline::InlineOptimization;
use std::{error::Error, fmt::Debug};

pub mod optimizations;

#[derive(Clone)]
pub struct OptimizationContext {
    pub ir: enalang_ir::IR,
    pub main: SharedStr,
}

impl Default for OptimizationContext {
    fn default() -> Self {
        Self {
            ir: Default::default(),
            main: shared_str!("main"),
        }
    }
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

pub struct Optimizer {
    pub optimizations: Vec<Box<dyn Optimization>>,
}

impl Default for Optimizer {
    fn default() -> Self {
        let mut opt = Self::new();

        opt.optimizations.push(Box::new(InlineOptimization::new()));

        opt
    }
}

impl Optimizer {
    pub fn new() -> Self {
        Self {
            optimizations: Vec::new(),
        }
    }

    pub fn optimize(
        &mut self,
        mut code: enalang_ir::IR,
        main: &SharedStr,
    ) -> Result<enalang_ir::IR, Box<dyn OptimizationError>> {
        for optimization in &mut self.optimizations {
            code = optimization.optimize(OptimizationContext {
                ir: code.clone(),
                main: main.clone(),
            })?;
        }
        Ok(code)
    }
}
