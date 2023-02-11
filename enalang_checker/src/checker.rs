use enalang_vm::{
    blocks::{self, Blocks},
    machine::{self, ScopeManager},
};
use std::fmt::Debug;

pub struct Checker {
    pub checks: Vec<Box<dyn Check>>,
    pub blocks: Option<blocks::Blocks>,
}

impl Checker {
    pub fn new(blocks: blocks::Blocks) -> Self {
        Self {
            blocks: Some(blocks),
            checks: Vec::new(),
        }
    }

    fn create_check_context(&self) -> CheckContext {
        CheckContext {
            scope_manager: ScopeManager::new(),
            blocks: self.blocks.as_ref().unwrap().clone(),
        }
    }

    pub fn set_blocks(&mut self, blocks: blocks::Blocks) {
        self.blocks = Some(blocks);
    }

    pub fn run_checks(&self, independent: bool) -> Vec<Box<dyn CheckError>> {
        let mut errs = Vec::new();

        for check in &self.checks {
            if independent && !check.is_independent() {
                continue;
            }
            if let Err(err) = check.check(self.create_check_context()) {
                errs.push(err);
            }
        }

        errs
    }

    pub fn add_check(&mut self, check: Box<impl Check + 'static>) {
        self.checks.push(check)
    }
}

impl Default for Checker {
    fn default() -> Self {
        Self {
            blocks: None,
            checks: Vec::new(),
        }
    }
}

pub struct CheckContext {
    pub scope_manager: machine::ScopeManager,
    pub blocks: Blocks,
}

impl Default for CheckContext {
    fn default() -> Self {
        Self {
            scope_manager: machine::ScopeManager::default(),
            blocks: Blocks::default(),
        }
    }
}

pub trait Check {
    fn check(&self, ctx: CheckContext) -> Result<(), Box<dyn CheckError>>;
    fn is_independent(&self) -> bool;
}

pub trait CheckError: Debug {
    fn explain(&self) -> String;
}

impl<T: CheckError + 'static> From<Box<T>> for Box<dyn CheckError> {
    fn from(value: Box<T>) -> Self {
        value
    }
}
