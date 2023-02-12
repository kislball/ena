use enalang_vm::{
    blocks::{self, Blocks},
    machine::{self, ScopeManager},
};
use std::{fmt::Debug, error::Error};

use crate::checks::blocks::BlocksChecker;

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

        errs.into_iter().flatten().collect()
    }

    pub fn add_check(&mut self, check: Box<impl Check + 'static>) {
        self.checks.push(check)
    }
}

impl Default for Checker {
    fn default() -> Self {
        Self {
            blocks: Some(blocks::Blocks::default()),
            checks: vec![Box::new(BlocksChecker::new())],
        }
    }
}

#[derive(Default)]
pub struct CheckContext {
    pub scope_manager: machine::ScopeManager,
    pub blocks: Blocks,
}

pub trait Check {
    fn check(&self, ctx: CheckContext) -> Result<(), Vec<Box<dyn CheckError>>>;
    fn is_independent(&self) -> bool;
}

pub trait CheckError: Debug + Error {
    fn from(&self) -> Option<String>;
}

impl<T: CheckError + 'static> From<Box<T>> for Box<dyn CheckError> {
    fn from(value: Box<T>) -> Self {
        value
    }
}
