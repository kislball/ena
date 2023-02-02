use flexstr::{LocalStr, ToLocalStr};
use serde::{Deserialize, Serialize};

use crate::vm::{heap, ir};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VMError {
    UnknownBlock(LocalStr),
    NoIR,
    StackEnded,
    ExpectedBoolean,
    ExpectedString,
    ExpectedNumber,
    ExpectedInteger,
    ExpectedBlock,
    ExpectedPointer,
    ExpectedValue,
    ExpectedException,
    CannotShadowBlocksInLocalScope,
    CannotCompare(ir::Value, ir::Value),
    CannotConvert(ir::Value),
    HeapError(heap::HeapError),
    BadPointer(usize),
    RuntimeException(ir::Value),
}

pub struct VM {
    pub stack: Vec<ir::Value>,
    pub debug_stack: bool,
    pub call_stack: Vec<LocalStr>,
    pub heap: heap::Heap,
}

impl VM {
    pub fn new(gc: bool, debug_gc: bool) -> Self {
        VM {
            stack: vec![],
            call_stack: vec![],
            debug_stack: false,
            heap: heap::Heap::new(gc, debug_gc),
        }
    }

    pub fn clean(&mut self) {
        self.stack = vec![];
        self.call_stack = vec![];
    }

    pub fn run(&mut self, ir: ir::IR, main: &str) -> Result<(), VMError> {
        self.clean();
        self.run_block(main.to_local_str(), &ir, &mut HashMap::new())
    }

    pub fn run_main(&mut self, ir: ir::IR) -> Result<(), VMError> {
        self.run(ir, "main")
    }

    pub fn handle_plus(&mut self, value: ir::Value) -> Result<(), VMError> {
        if let ir::Value::Pointer(pointer) = value {
            self.heap.rc_plus(pointer).map_err(VMError::HeapError)
        } else {
            Ok(())
        }
    }

    pub fn handle_minus(&mut self, value: ir::Value) -> Result<(), VMError> {
        if let ir::Value::Pointer(pointer) = value {
            self.heap.rc_minus(pointer).map_err(VMError::HeapError)
        } else {
            Ok(())
        }
    }

    pub fn push(&mut self, value: ir::Value) -> Result<(), VMError> {
        self.handle_plus(value.clone())?;
        self.stack.push(value);
        Ok(())
    }

    pub fn pop(&mut self) -> Result<ir::Value, VMError> {
        match self.stack.pop() {
            Some(i) => {
                self.handle_minus(i.clone())?;
                Ok(i)
            }
            None => Err(VMError::StackEnded),
        }
    }

    pub fn pop_usize(&mut self) -> Result<usize, VMError> {
        let val = self.pop()?;
        if let ir::Value::Number(num) = val {
            let int = num as usize;

            if int as f64 != num {
                return Err(VMError::ExpectedInteger);
            }

            Ok(int)
        } else {
            Err(VMError::ExpectedInteger)
        }
    }

    pub fn print_call_stack(&self) {
        println!("vm stack trace: {:#?}", self.call_stack);
    }

    pub fn run_block(
        &mut self,
        block_name: LocalStr,
        ir: &ir::IR,
        single_evals: &mut HashMap<LocalStr, ir::Value>,
    ) -> Result<(), VMError> {
        let mut locals: Vec<LocalStr> = Vec::new();

        let mut local_ir = ir::IR::new();
        local_ir.add(ir).unwrap();
        self.call_stack.push(block_name.clone());

        let binding = (&ir).get_block(block_name.clone());
        let block = match binding {
            Some(b) => b,
            None => return Err(VMError::UnknownBlock(block_name)),
        };
        let single_eval;

        if let ir::Block::IR(ir::BlockRunType::Once, _) = block {
            if let Some(i) = single_evals.get(&block_name) {
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
                        ir::IRCode::LocalBlock(n, t, code) => {
                            let err = local_ir
                                .add_block(n.to_local_str(), ir::Block::IR(*t, code.to_vec()));
                            if let Err(_) = err {
                                return Err(VMError::CannotShadowBlocksInLocalScope);
                            } else {
                                locals.push(n.to_local_str());
                            }
                        }
                        ir::IRCode::PutValue(v) => {
                            self.stack.push(v.clone());
                        }
                        ir::IRCode::Call(b) => {
                            self.run_block(b.to_local_str(), &local_ir, single_evals)?;
                        }
                        ir::IRCode::While(b) => loop {
                            let top = self.pop()?;
                            if let ir::Value::Boolean(true) = top {
                                self.run_block(b.to_local_str(), &local_ir, single_evals)?;
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
                                    self.run_block(b.to_local_str(), &local_ir, single_evals)?;
                                }
                            } else {
                                return Err(VMError::ExpectedBoolean);
                            }
                        }
                        ir::IRCode::Return => {
                            if single_eval {
                                if let Some(i) = self.stack.last() {
                                    single_evals.insert(block_name, i.clone());
                                };
                            }
                            return Ok(());
                        }
                    }
                }
                if single_eval {
                    let v = self.stack.last();
                    if let Some(i) = v {
                        single_evals.insert(block_name, i.clone());
                        self.handle_plus(i.clone())?;
                    };
                }
                Ok(())
            }
            ir::Block::Native(f) => { 
                f(ir::NativeHandlerCtx {
                    ir: &ir,
                    locals: &mut locals,
                    single_evals: single_evals,
                    vm: self,
                })
            },
        };

        self.call_stack.pop();

        for k in &locals {
            if single_evals.contains_key(k) {
                self.handle_minus(single_evals.remove(k).unwrap())?;
            }
        }

        v
    }
}
