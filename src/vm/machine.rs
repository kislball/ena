use flexstr::{local_str, LocalStr, ToLocalStr};

use crate::vm::{heap, ir};
use std::collections::HashMap;

#[derive(Debug)]
pub enum VMError {
    UnknownBlock(String),
    NoIR,
    StackEnded(String),
    ExpectedBoolean(String),
    ExpectedString(String),
    ExpectedNumber(String),
    ExpectedInteger(String),
    CannotCompare(String),
    ExpectedBlock(String),
    ExpectedPointer(String),
    HeapError(heap::HeapError),
    BadPointer(String),
    ExpectedTwo(&'static str),
}

pub struct VM {
    pub stack: Vec<ir::Value>,
    pub debug_stack: bool,
    pub call_stack: Vec<LocalStr>,
    pub heap: heap::Heap,
    single_eval_blocks: HashMap<LocalStr, ir::Value>,
}

impl<'a> VM {
    pub fn new(gc: bool, debug_gc: bool) -> Self {
        VM {
            stack: vec![],
            call_stack: vec![],
            single_eval_blocks: HashMap::new(),
            debug_stack: false,
            heap: heap::Heap::new(gc, debug_gc),
        }
    }

    pub fn clean(&mut self) {
        self.stack = vec![];
        self.call_stack = vec![];
        self.single_eval_blocks = HashMap::new();
    }

    pub fn run(&mut self, ir: &ir::IR<'a>, main: &'a str) -> Result<(), VMError> {
        self.clean();
        self.run_block(main.to_local_str(), ir)
    }

    pub fn run_main(&mut self, ir: &ir::IR<'a>) -> Result<(), VMError> {
        self.run(ir, "main")
    }

    pub fn push(&mut self, value: ir::Value) -> Result<(), VMError> {
        if let ir::Value::Pointer(pointer) = value {
            self.heap
                .rc_plus(pointer)
                .map_err(VMError::HeapError)?;
        }

        self.stack.push(value);
        Ok(())
    }

    pub fn pop(&mut self) -> Result<ir::Value, VMError> {
        match self.stack.pop() {
            Some(i) => {
                if let ir::Value::Pointer(pointer) = i {
                    self.heap
                        .rc_minus(pointer)
                        .map_err(VMError::HeapError)?
                }
                Ok(i)
            }
            None => Err(VMError::StackEnded(self.block_name().to_string())),
        }
    }

    pub fn pop_usize(&mut self) -> Result<usize, VMError> {
        let val = self.pop()?;
        if let ir::Value::Number(num) = val {
            let int = num as usize;

            if int as f64 != num {
                return Err(VMError::ExpectedInteger(self.block_name().to_string()));
            }

            Ok(int)
        } else {
            Err(VMError::ExpectedInteger(self.block_name().to_string()))
        }
    }

    fn block_name(&self) -> LocalStr {
        match self.call_stack.last() {
            Some(i) => i.clone(),
            None => local_str!("unknown block"),
        }
    }

    pub fn print_call_stack(&self) {
        println!("vm stack trace: {:#?}", self.call_stack);
    }

    pub fn run_block(&mut self, block_name: LocalStr, ir: &ir::IR<'a>) -> Result<(), VMError> {
        self.call_stack.push(block_name.clone());
        let block = match ir.get_block(block_name.clone()) {
            Some(b) => b,
            None => return Err(VMError::UnknownBlock(block_name.to_string())),
        };
        let single_eval;

        if let ir::Block::IR(ir::BlockRunType::Once, _) = block {
            if let Some(i) = self.single_eval_blocks.get(&block_name) {
                              self.stack.push(i.clone());
                             return Ok(());
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
                            self.stack.push(v.clone());
                        }
                        ir::IRCode::Call(b) => {
                            self.run_block(b.to_local_str(), ir)?;
                        }
                        ir::IRCode::While(b) => loop {
                            let top = self.pop()?;
                            if let ir::Value::Boolean(true) = top {
                                self.run_block(b.to_local_str(), ir)?;
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
                                    let v = self.run_block(b.to_local_str(), ir);
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
                                if let Some(i) = self.stack.last() {
                                                                       self.single_eval_blocks.insert(block_name, i.clone());
                                                                  };
                            }
                            return Ok(());
                        }
                    }
                }
                if single_eval {
                    if let Some(i) = self.stack.last() {
                                              self.single_eval_blocks.insert(block_name, i.clone());
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
