use std::{collections::HashMap};
use crate::vm::ir;

#[derive(Debug)]
pub enum VMError {
    UnknownBlock(String),
    NoIR,
    StackEnded(String),
    ExpectedBoolean(String),
    ExpectedString(String),
}

pub struct VM<'a> {
    pub stack: Vec<ir::Value<'a>>,
    single_eval_blocks: HashMap<&'a str, ir::Value<'a>>,
    current_block: Option<&'a str>,
}

impl<'a> VM<'a> {
    pub fn new() -> Self {
        VM {
            stack: vec![],
            current_block: None,
            single_eval_blocks: HashMap::new(),
        }
    }

    pub fn clean(&mut self) {
        self.stack = vec![];
        self.current_block = None;
        self.single_eval_blocks = HashMap::new();
    }

    pub fn run(&mut self, ir: &ir::IR<'a>, main: &'a str) -> Result<(), VMError> {
        self.clean();
        self.run_block(main, ir)
    }

    pub fn run_main(&mut self, ir: &ir::IR<'a>) -> Result<(), VMError> {
        self.run(ir, "main")
    }

    pub fn pop(&mut self) -> Result<ir::Value<'a>, VMError> {
        match self.stack.pop() {
            Some(i) => Ok(i),
            None => Err(VMError::StackEnded(self.block_name().to_string())),
        }
    }

    fn block_name(&self) -> &'a str {
        &self.current_block.unwrap_or("unknown block")
    }

    pub fn run_block(&mut self, block_name: &'a str, ir: &ir::IR<'a>) -> Result<(), VMError> {
        self.current_block = Some(block_name);
        let block = match ir.get_block(block_name) {
            Some(b) => b,
            None => return Err(VMError::UnknownBlock(block_name.to_string())),
        };
        let single_eval;
        
        if let ir::Block::IR(ir::BlockRunType::Once, _) = block {
            match self.single_eval_blocks.get(block_name) {
                Some(i) => {
                    self.stack.push(*i);
                    return Ok(());
                },
                None => {},
            }
            single_eval = true;
        } else {
            single_eval = false;
        }

        match block {
            ir::Block::IR(_, code) => {
                for c in code {
                    match c {
                        ir::IRCode::PutValue(v) => {
                            self.stack.push(*v);
                        }
                        ir::IRCode::Call(b) => {
                            self.run_block(b, ir)?;
                        }
                        ir::IRCode::While(b) => loop {
                            let top = self.pop()?;
                            if let ir::Value::Boolean(true) = top {
                                self.run_block(b, ir)?;
                                self.current_block = Some(block_name);
                            } else {
                                break;
                            }
                        },
                        ir::IRCode::If(b) => {
                            let top = self.pop()?;
                            if let ir::Value::Boolean(bo) = top {
                                if !bo {
                                    break;
                                } else {
                                    let v = self.run_block(b, ir);
                                    self.current_block = Some(block_name);
                                    return v;
                                }
                            } else {
                                return Err(VMError::ExpectedBoolean(
                                    self.block_name().to_string(),
                                ));
                            }
                        }
                        ir::IRCode::Return => {
                            if single_eval {
                                match self.stack.last() {
                                    Some(i) => {
                                        self.single_eval_blocks.insert(block_name, *i);
                                    },
                                    None => {},
                                };
                            }
                            return Ok(());
                        }
                    }
                }
                if single_eval {
                    match self.stack.last() {
                        Some(i) => {
                            self.single_eval_blocks.insert(block_name, *i);
                        },
                        None => {},
                    };
                }
                Ok(())
            }
            ir::Block::Native(f) => f(self, ir),
        }
    }
}
