use crate::vm::ir;
use std::collections::HashMap;

#[derive(Debug)]
pub enum VMError {
    UnknownBlock(String),
    NoIR,
    StackEnded(String),
    ExpectedBoolean(String),
    ExpectedString(String),
    ExpectedNumber(String),
    CannotCompare(String),
    ExpectedBlock(String),
}

pub struct VM<'a> {
    pub stack: Vec<ir::Value<'a>>,
    pub debug_stack: bool,
    pub call_stack: Vec<&'a str>,
    single_eval_blocks: HashMap<&'a str, ir::Value<'a>>,
}

impl<'a> VM<'a> {
    pub fn new() -> Self {
        VM {
            stack: vec![],
            call_stack: vec![],
            single_eval_blocks: HashMap::new(),
            debug_stack: false,
        }
    }

    pub fn clean(&mut self) {
        self.stack = vec![];
        self.call_stack = vec![];
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
        self.call_stack.last().unwrap_or(&"unknown block")
    }

    pub fn print_call_stack(&self) {
        println!("vm stack trace: {:#?}", self.call_stack);
    }

    pub fn run_block(&mut self, block_name: &'a str, ir: &ir::IR<'a>) -> Result<(), VMError> {
        self.call_stack.push(block_name);
        let block= match ir.get_block(block_name) {
            Some(b) => b,
            None => return Err(VMError::UnknownBlock(block_name.to_string())),
        };
        let single_eval;

        if let ir::Block::IR(ir::BlockRunType::Once, _) = block {
            match self.single_eval_blocks.get(block_name) {
                Some(i) => {
                    self.stack.push(*i);
                    return Ok(());
                }
                None => {}
            }
            single_eval = true;
        } else {
            single_eval = false;
        }

        let v = match block {
            ir::Block::IR(_, code) => {
                for c in code {
                    if self.debug_stack {
                        println!("\n\n=== stack debug ===\n\n{:?}", self.stack);
                    }
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
                            } else {
                                break;
                            }
                        },
                        ir::IRCode::If(b) => {
                            let top = self.pop()?;
                            if let ir::Value::Boolean(bo) = top {
                                if !bo {
                                    continue;
                                } else {
                                    let v = self.run_block(b, ir);
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
                                    }
                                    None => {}
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
                        }
                        None => {}
                    };
                }
                Ok(())
            }
            ir::Block::Native(f) => f(self, ir),
        };

        self.call_stack.pop();

        v
    }
}
