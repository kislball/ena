use std::collections::HashMap;

use flexstr::{local_str, LocalStr};
use serde::{Deserialize, Serialize};

use crate::vm::{heap, ir};

use super::ir::BlockRunType;

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
    CannotShadowBlocksInLocalScope(LocalStr),
    CannotCompare(ir::Value, ir::Value),
    CannotConvert(ir::Value),
    HeapError(heap::HeapError),
    BadPointer(usize),
    RuntimeException(ir::Value),
    NoScope,
    NoSingleEval,
}

#[derive(Clone, Debug)]
pub struct Scope {
    pub block: LocalStr,
    pub ir: ir::IR,
    pub single_evals: HashMap<LocalStr, ir::Value>,
    pub locals: Vec<LocalStr>,
}

impl Scope {
    pub fn new(ir: ir::IR, block: LocalStr) -> Self {
        Self {
            ir,
            single_evals: HashMap::new(),
            locals: vec![],
            block,
        }
    }

    pub fn add_local(&mut self, local: LocalStr) {
        self.locals.push(local);
    }

    pub fn has_local(&self, local: &LocalStr) -> bool {
        self.locals.contains(&local)
    }

    pub fn add_single_eval(&mut self, local: LocalStr, value: ir::Value) {
        self.single_evals.insert(local, value);
    }

    pub fn has_single_eval(&mut self, local: &LocalStr) -> bool {
        self.single_evals.contains_key(local)
    }
}

pub struct ScopeManager {
    pub scopes: Vec<Scope>,
}

impl ScopeManager {
    pub fn new() -> Self {
        ScopeManager { scopes: vec![] }
    }

    pub fn root(&mut self, ir: ir::IR, block: LocalStr) -> Result<&Scope, VMError> {
        self.scopes = vec![];
        let mut locals: Vec<LocalStr> = Vec::new();

        for (name, block) in &ir.blocks {
            if let ir::Block::IR(true, BlockRunType::Once, _) = block {
                locals.push(name.clone());
            }
        }

        let root = Scope {
            ir,
            single_evals: HashMap::new(),
            locals,
            block,
        };

        self.scopes.push(root);

        Ok(self.scopes.last().unwrap())
    }

    pub fn parent(&mut self, block: LocalStr) -> Result<&Scope, VMError> {
        let root = match self.scopes.first() {
            Some(i) => i,
            None => {
                return Err(VMError::NoScope);
            }
        };

        let new_scope = Scope {
            ir: root.ir.clone(),
            locals: vec![],
            single_evals: HashMap::new(),
            block,
        };
        self.scopes.push(new_scope);

        Ok(self.scopes.last().unwrap())
    }

    pub fn child(&mut self, block: LocalStr) -> Result<&Scope, VMError> {
        let parent = match self.scopes.last() {
            Some(i) => i,
            None => {
                return Err(VMError::NoScope);
            }
        };

        let new_scope = Scope {
            ir: parent.ir.clone(),
            locals: vec![],
            single_evals: HashMap::new(),
            block,
        };
        self.scopes.push(new_scope);

        Ok(self.scopes.last().unwrap())
    }

    pub fn pop_scope(&mut self) -> Result<Vec<ir::Value>, VMError> {
        let scope = self.scopes.pop();
        match scope {
            None => Err(VMError::NoScope),
            Some(scope) => {
                let mut res_vec = Vec::<ir::Value>::new();
                for (_, val) in scope.single_evals {
                    res_vec.push(val);
                }
                Ok(res_vec)
            }
        }
    }

    pub fn remove_single_eval(&mut self, local: LocalStr) -> Result<(), VMError> {
        let owner = match self.lookup_local_owner_mut(&local) {
            Some(i) => i,
            None => {
                return Err(VMError::UnknownBlock(local));
            }
        };
        for (i, name) in owner.locals.clone().iter().enumerate() {
            if name == local {
                owner.locals.remove(i);
                owner.single_evals.remove(name);
            }
        }
        Ok(())
    }

    pub fn add_single_eval(&mut self, local: LocalStr, value: ir::Value) -> Result<(), VMError> {
        let owner = match self.lookup_local_owner_mut(&local) {
            Some(i) => i,
            None => {
                return Err(VMError::UnknownBlock(local));
            }
        };

        owner.add_single_eval(local, value);
        Ok(())
    }

    pub fn lookup_single_eval(&self, local: &LocalStr) -> Result<ir::Value, VMError> {
        let owner = match self.lookup_local_owner(local) {
            Some(i) => i,
            None => {
                return Err(VMError::UnknownBlock(local.clone()));
            }
        };

        match owner.single_evals.get(local) {
            Some(v) => Ok(v.clone()),
            None => Err(VMError::NoSingleEval),
        }
    }

    pub fn add_local(&mut self, local: LocalStr) -> Result<(), VMError> {
        let current_scope = self.scopes.last_mut();
        let current_scope = match current_scope {
            Some(i) => i,
            None => {
                return Err(VMError::UnknownBlock(local));
            }
        };

        current_scope.add_local(local);
        Ok(())
    }

    pub fn lookup_local_owner_mut(&mut self, local: &LocalStr) -> Option<&mut Scope> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.has_local(local) {
                return Some(scope);
            }
        }

        None
    }

    pub fn ir(&self) -> &ir::IR {
        &self.scopes.last().unwrap().ir
    }

    pub fn ir_mut(&mut self) -> &mut ir::IR {
        &mut self.scopes.last_mut().unwrap().ir
    }

    pub fn lookup_local_owner(&self, local: &LocalStr) -> Option<&Scope> {
        for scope in self.scopes.iter().rev() {
            if scope.has_local(local) {
                return Some(scope);
            }
        }

        None
    }
}

pub struct VM {
    pub stack: Vec<ir::Value>,
    pub call_stack: Vec<LocalStr>,
    pub heap: heap::Heap,
    pub options: VMOptions,
    pub scope_manager: ScopeManager,
}

#[derive(Clone, Copy)]
pub struct VMOptions {
    pub debug_stack: bool,
    pub enable_gc: bool,
    pub debug_gc: bool,
}

impl VMOptions {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for VMOptions {
    fn default() -> Self {
        Self {
            debug_stack: false,
            enable_gc: true,
            debug_gc: false,
        }
    }
}

impl VM {
    pub fn new(options: VMOptions) -> Self {
        Self {
            call_stack: Vec::new(),
            heap: heap::Heap::new(options.enable_gc, options.debug_gc),
            options,
            stack: Vec::new(),
            scope_manager: ScopeManager::new(),
        }
    }

    pub fn clean(&mut self) {
        self.call_stack = Vec::new();
        self.heap = heap::Heap::new(self.options.enable_gc, self.options.debug_gc);
        self.stack = Vec::new();
    }

    pub fn handle_plus(&mut self, value: ir::Value) -> Result<(), VMError> {
        if let ir::Value::Pointer(ptr) = value {
            self.heap.rc_plus(ptr).map_err(VMError::HeapError)
        } else {
            Ok(())
        }
    }

    pub fn handle_minus(&mut self, value: ir::Value) -> Result<(), VMError> {
        if let ir::Value::Pointer(ptr) = value {
            self.heap.rc_minus(ptr).map_err(VMError::HeapError)
        } else {
            Ok(())
        }
    }

    pub fn pop_pointer(&mut self) -> Result<usize, VMError> {
        let val = self.pop()?;
        match val {
            ir::Value::Number(i) => {
                let in_usize = i as usize;
                if (in_usize as f64) != i {
                    return Err(VMError::ExpectedInteger);
                }

                Ok(in_usize)
            }
            _ => Err(VMError::ExpectedPointer),
        }
    }

    pub fn push(&mut self, value: ir::Value) -> Result<(), VMError> {
        self.stack.push(value.clone());
        self.handle_plus(value)?;
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

    pub fn run(&mut self, main: &LocalStr, ir: ir::IR) -> Result<(), VMError> {
        self.new_scope(ir)?;
        self.run_block(main)
    }

    pub fn new_scope(&mut self, ir: ir::IR) -> Result<&Scope, VMError> {
        self.scope_manager.root(ir, local_str!("root"))
    }

    pub fn pop_scope(&mut self) -> Result<(), VMError> {
        let to_clean = self.scope_manager.pop_scope()?;
        for val in to_clean {
            self.handle_minus(val)?;
        }
        Ok(())
    }

    pub fn run_block(&mut self, block_name: &LocalStr) -> Result<(), VMError> {
        let block = match self.scope_manager.ir().get_block(block_name).cloned() {
            Some(i) => i,
            None => {
                return Err(VMError::UnknownBlock(block_name.clone()));
            }
        };

        if block.is_global() {
            self.scope_manager.parent(block_name.clone())?;
        } else {
            self.scope_manager.child(block_name.clone())?;
        }

        if block.is_single_eval() {
            let val = self.scope_manager.lookup_single_eval(block_name);
            if let Ok(v) = val {
                self.push(v)?;
                self.pop_scope().unwrap();
                return Ok(());
            }
        }

        self.call_stack.push(block_name.clone());

        let v = match block {
            ir::Block::Native(f) => f(ir::NativeHandlerCtx { vm: self }),
            ir::Block::IR(_, typ, vec) => {
                for code in vec {
                    let result = match code {
                        ir::IRCode::PutValue(val) => self.push(val.clone()),
                        ir::IRCode::Return => {
                            break;
                        }
                        ir::IRCode::Call(name) => self.run_block(&name),
                        ir::IRCode::LocalBlock(name, typ, vec) => {
                            self.scope_manager.add_local(name.clone())?;
                            self.scope_manager
                                .ir_mut()
                                .add_block(name.clone(), ir::Block::IR(false, typ, vec), true)
                                .map_err(|_| VMError::CannotShadowBlocksInLocalScope(name.clone()))
                        }
                        ir::IRCode::If(block) => {
                            let val = self.pop()?;
                            if let ir::Value::Boolean(b) = val {
                                if b {
                                    self.run_block(&block)
                                } else {
                                    Ok(())
                                }
                            } else {
                                Err(VMError::ExpectedBoolean)
                            }
                        }
                        ir::IRCode::While(block) => {
                            while let ir::Value::Boolean(true) = self.pop()? {
                                self.run_block(&block)?;
                            }
                            Ok(())
                        }
                    };
                    if let Err(e) = result {
                        return Err(e);
                    }
                }

                if let BlockRunType::Once = typ {
                    let top = self.stack.last();
                    if let None = top {
                        return Err(VMError::ExpectedValue);
                    } else if let Some(i) = top {
                        self.scope_manager
                            .add_single_eval(block_name.clone(), i.clone())?;
                        self.handle_plus(i.clone())?;
                    }
                }

                Ok(())
            }
        };

        if let Err(e) = v {
            return Err(e);
        }

        self.pop_scope().unwrap();
        self.call_stack.pop().unwrap();

        Ok(())
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new(VMOptions::default())
    }
}
