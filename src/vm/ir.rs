use crate::vm::ir;
use crate::vm::machine;
use core::fmt;
use flexstr::LocalStr;
use flexstr::ToLocalStr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug)]
pub enum IRError {
    WordAlreadyExists,
}

#[derive(Debug, Clone)]
pub struct IR<'a> {
    pub blocks: HashMap<LocalStr, Block<'a>>,
}

impl<'a> Default for IR<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> IR<'a> {
    pub fn new() -> Self {
        IR {
            blocks: HashMap::new(),
        }
    }

    pub fn into_serializable(&'a self) -> IRSerializable<'a> {
        let mut blocks: Vec<IRSerializable<'a>> = Vec::new();

        for (name, block) in &self.blocks {
            if let Block::IR(typ, data) = block {
                blocks.push(IRSerializable::Block(name, *typ, data.to_vec()));
            }
        }

        IRSerializable::Root(blocks)
    }

    pub fn add(&mut self, another: &ir::IR<'a>) -> Result<(), IRError> {
        for (name, block) in &another.blocks {
            self.add_block(name.clone(), block.clone())?;
        }
        Ok(())
    }

    pub fn get_block(&self, id: LocalStr) -> Option<&Block<'a>> {
        self.blocks.get(&id)
    }

    pub fn add_native(&mut self, name: LocalStr, f: NativeHandler<'a>) -> Result<(), IRError> {
        self.add_block(name, Block::Native(f))
    }

    pub fn add_block(&mut self, name: LocalStr, block: Block<'a>) -> Result<(), IRError> {
        if self.blocks.contains_key(&name) {
            return Err(IRError::WordAlreadyExists);
        }
        self.blocks.insert(name, block);
        Ok(())
    }
}

pub type NativeHandler<'a> = fn(&mut machine::VM, &ir::IR<'a>) -> Result<(), machine::VMError>;

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum BlockRunType {
    Once,
    Unique,
}

#[derive(Clone)]
pub enum Block<'a> {
    IR(BlockRunType, Vec<IRCode<'a>>),
    Native(NativeHandler<'a>),
}

impl<'a> fmt::Debug for Block<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Block::IR(typ, vec) => {
                write!(f, "IRBlock({typ:?}, {vec:?})")
            }
            Block::Native(_) => write!(f, "NativeHandler"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum IRSerializable<'a> {
    Block(&'a str, BlockRunType, Vec<IRCode<'a>>),
    Root(Vec<IRSerializable<'a>>),
}

#[derive(Debug)]
pub enum SerializationError {
    ExpectedRoot,
    ExpectedBlock,
    BincodeErr(bincode::ErrorKind),
    IRError(IRError),
}

pub fn from_vec(data: &[u8]) -> Result<IRSerializable, SerializationError> {
    bincode::deserialize(data).map_err(|err| SerializationError::BincodeErr(*err))
}

impl<'a> IRSerializable<'a> {
    pub fn into_vec(&self) -> Result<Vec<u8>, SerializationError> {
        bincode::serialize(self).map_err(|err| SerializationError::BincodeErr(*err))
    }

    pub fn into_ir(self) -> Result<IR<'a>, SerializationError> {
        let mut ir = IR::new();

        if let IRSerializable::Root(data) = self {
            for ser_block in data {
                if let IRSerializable::Block(name, typ, data) = ser_block {
                    let block = Block::IR(typ, data.to_vec());
                    ir.add_block(name.to_local_str(), block)
                        .map_err(SerializationError::IRError)?;
                } else {
                    return Err(SerializationError::ExpectedBlock);
                }
            }

            Ok(ir)
        } else {
            Err(SerializationError::ExpectedRoot)
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Value {
    Number(f64),
    String(LocalStr),
    Boolean(bool),
    Pointer(usize),
    Block(LocalStr),
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IRCode<'a> {
    PutValue(Value),
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
            self.add_native(Self::merge_prefix(group.prefix, k), *v)?;
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
            if self.prefix.is_empty() {
                ir.add_native(k.to_local_str(), *v)?;
            } else {
                ir.add_native(Self::merge_prefix(self.prefix, k).to_local_str(), *v)?;
            }
        }

        Ok(())
    }

    fn merge_prefix(prefix: &'a str, name: &'a str) -> &'a str {
        if prefix.is_empty() {
            name
        } else {
            // not dangerous since this stuff should not be freed until the end of the program.
            let leaky: &'static str = Box::leak(format!("{prefix}.{name}").into_boxed_str());
            leaky
        }
    }
}
