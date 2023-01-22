use crate::vm::ir;
use crate::vm::machine;
use core::fmt;
use std::collections::HashMap;

#[derive(Debug)]
pub enum IRError {
    WordAlreadyExists,
}

#[derive(Debug)]
pub struct IR<'a> {
    pub blocks: HashMap<&'a str, Block<'a>>,
}

impl<'a> IR<'a> {
    pub fn new() -> Self {
        IR {
            blocks: HashMap::new(),
        }
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

pub type NativeHandler<'a> = fn(&mut machine::VM<'a>, &ir::IR) -> Result<(), machine::VMError>;

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
            }
            Block::Native(_) => write!(f, "NativeHandler"),
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

pub struct NativeGroup<'a> {
    natives: HashMap<&'a str, NativeHandler<'a>>,
    prefix: &'a str,
}

impl<'a> NativeGroup<'a> {
    pub fn new(prefix: &'a str) -> Self {
        Self {
            natives: HashMap::new(),
            prefix,
        }
    }

    pub fn add_child(&mut self, group: &NativeGroup<'a>) -> Result<(), IRError> {
        for (k, v) in &group.natives {
            self.add_native(k, *v)?;
        }
        Ok(())
    }

    pub fn add_native(&mut self, name: &'a str, f: NativeHandler<'a>) -> Result<(), IRError> {
        if self.natives.contains_key(name) {
            return Err(IRError::WordAlreadyExists);
        }
        self.natives.insert(name, f);
        Ok(())
    }

    pub fn apply(&self, ir: &mut IR<'a>) -> Result<(), IRError> {
        for (k, v) in &self.natives {
            if self.prefix.len() == 0 {
                ir.add_native(k, *v)?;
            } else {
                // not dangerous since this stuff should not be freed until the end of the program.
                let leaky: &'static str = Box::leak(format!("{}.{}", self.prefix, k).into_boxed_str());
                ir.add_native(leaky, *v)?;
            }
        }

        Ok(())
    }
}
