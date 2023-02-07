use std::fmt::Debug;
use enalang_vm::{blocks,machine::{self, ScopeManager}};

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
        }
    }

    pub fn set_blocks(&mut self, blocks: blocks::Blocks) {
        self.blocks = Some(blocks);
    }

    pub fn run_checks(&self, independent: bool) -> Vec<Box<dyn CheckError>> {
        let mut errs = Vec::<Box<dyn CheckError>>::new();

        for check in &self.checks {
            if independent && !check.is_independent() {
                continue;
            }
            if let Some(err) = check.check(self.create_check_context()) {
                errs.push(err);
            }
        }

        errs
    }

    pub fn add_check(&mut self, check: Box<impl Check + 'static>)  {
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
}

impl Default for CheckContext {
    fn default() -> Self {
        Self {
            scope_manager: machine::ScopeManager::default(),
        }
    }
}

pub trait Check {
    fn check(&self, ctx: CheckContext) -> Option<Box<dyn CheckError>>;
    fn is_independent(&self) -> bool;
}

pub trait CheckError: Debug {
    fn explain(&self) -> String;
}