use crate::vm::machine;
use crate::vm::ir;
use core::fmt;
use rand;
use rand::Rng;
use std::collections::HashMap;

#[derive(Debug)]
pub enum IRError {
    WordAlreadyExists,
}

#[derive(Debug)]
pub struct IR<'a> {
    pub blocks: HashMap<&'a str, Block<'a>>,
}

fn ena_vm_debug(vm: &mut machine::VM) -> Result<(), machine::VMError> {
    let el = match vm.stack.pop() {
        Some(i) => i,
        None => Value::Null,
    };

    println!("{:?}", el);

    Ok(())
}

fn ena_vm_get_random(vm: &mut machine::VM) -> Result<(), machine::VMError> {
    vm.stack.push(ir::Value::Number(rand::thread_rng().gen_range(1..=90000) as f64));
    Ok(())
}

impl<'a> IR<'a> {
    pub fn new() -> Self {
        let mut ir = IR {
            blocks: HashMap::new(),
        };

        ir.add_native("ena.vm.debug", ena_vm_debug).unwrap();
        ir.add_native("ena.vm.random", ena_vm_get_random).unwrap();

        ir
    }

    pub fn get_block(&self, id: &'a str) -> Option<&Block<'a>> {
        self.blocks.get(id)
    }

    pub fn add_native(&mut self, name: &'a str, f: NativeHandler<'a>) -> Result<(), IRError> {
        self.add_block(name, Block::Native(f))
    }

    pub fn add_block(&mut self, name: &'a str, block: Block<'a>) -> Result<(), IRError> {
        if self.blocks.contains_key(name) {
            return Err(IRError::WordAlreadyExists);
        }
        self.blocks.insert(name, block);
        Ok(())
    }
}

pub type NativeHandler<'a> = fn(&mut machine::VM<'a>) -> Result<(), machine::VMError>;

#[derive(Debug)]
pub enum BlockRunType {
    Once,
    Unique,
}

pub enum Block<'a> {
    IR(BlockRunType, Vec<IRCode<'a>>),
    Native(NativeHandler<'a>),
}

impl<'a> fmt::Debug for Block<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Block::IR(typ, vec) => {
                write!(f, "IRBlock({:?}, {:?})", typ, vec)
            },
            Block::Native(_) => write!(f, "NativeHandler")
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Value<'a> {
    Number(f64),
    String(&'a str),
    Boolean(bool),
    Pointer(usize),
    Block(&'a str),
    Null,
}

#[derive(Debug)]
pub enum IRCode<'a> {
    PutValue(Value<'a>),
    Call(&'a str),
    While(&'a str),
    If(&'a str),
    Return,
}
