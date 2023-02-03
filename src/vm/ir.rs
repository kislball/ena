use crate::vm::ir;
use crate::vm::machine;
use core::fmt;
use flexstr::local_fmt;
use flexstr::LocalStr;
use flexstr::ToLocalStr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug)]
pub enum IRError {
    BlockAlreadyExists,
}

#[derive(Debug, Clone)]
pub struct IR {
    pub blocks: HashMap<LocalStr, Block>,
    pub annotations: HashMap<LocalStr, LocalStr>,
}

impl Default for IR {
    fn default() -> Self {
        Self::new()
    }
}

impl IR {
    pub fn new() -> Self {
        IR {
            blocks: HashMap::new(),
            annotations: HashMap::new(),
        }
    }

    pub fn into_serializable(&self) -> IRSerializable {
        let mut blocks: Vec<IRSerializable> = Vec::new();

        for (name, block) in &self.blocks {
            if let Block::IR(typ, data) = block {
                blocks.push(IRSerializable::Block(name, *typ, data.to_vec()));
            }
        }

        for (block, content) in &self.annotations {
            blocks.push(IRSerializable::Annotation(block.clone(), content.clone()));
        }

        IRSerializable::Root(blocks)
    }

    pub fn add(&mut self, another: &ir::IR) -> Result<(), IRError> {
        for (name, block) in &another.blocks {
            self.add_block(name.clone(), block.clone(), true)?;
        }

        for (name, comment) in &another.annotations {
            self.annotations.insert(name.clone(), comment.clone());
        }
        Ok(())
    }

    pub fn get_block(&self, id: LocalStr) -> Option<&Block> {
        self.blocks.get(&id)
    }

    pub fn add_native(&mut self, name: LocalStr, f: NativeHandler, output_err: bool) -> Result<(), IRError> {
        self.add_block(name, Block::Native(f), output_err)
    }

    pub fn add_block(&mut self, name: LocalStr, block: Block, output_err: bool) -> Result<(), IRError> {
        if self.blocks.contains_key(&name) && output_err {
            return Err(IRError::BlockAlreadyExists);
        }
        self.blocks.insert(name, block);
        Ok(())
    }
}

pub struct NativeHandlerCtx<'a> {
    pub vm: &'a mut machine::VM,
    pub ir: &'a ir::IR,
    pub single_evals: &'a mut HashMap<LocalStr, ir::Value>,
    pub locals: &'a mut Vec<LocalStr>,
}

pub type NativeHandler = fn(ctx: NativeHandlerCtx) -> Result<(), machine::VMError>;

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum BlockRunType {
    Once,
    Unique,
}

#[derive(Clone)]
pub enum Block {
    IR(BlockRunType, Vec<IRCode>),
    Native(NativeHandler),
}

impl fmt::Debug for Block {
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
    Block(&'a str, BlockRunType, Vec<IRCode>),
    Root(Vec<IRSerializable<'a>>),
    Annotation(LocalStr, LocalStr),
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

    pub fn into_ir(self) -> Result<IR, SerializationError> {
        let mut ir = IR::new();

        if let IRSerializable::Root(data) = self {
            for ser_block in data {
                if let IRSerializable::Block(name, typ, data) = ser_block {
                    let block = Block::IR(typ, data.to_vec());
                    ir.add_block(name.to_local_str(), block, true)
                        .map_err(SerializationError::IRError)?;
                } else if let IRSerializable::Annotation(name, data) = ser_block {
                    ir.annotations.insert(name, data);
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
    VMError(Box<machine::VMError>),
    Atom(LocalStr),
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IRCode {
    PutValue(Value),
    LocalBlock(LocalStr, BlockRunType, Vec<IRCode>),
    Call(LocalStr),
    While(LocalStr),
    If(LocalStr),
    Return,
}

pub struct NativeGroup {
    natives: HashMap<LocalStr, NativeHandler>,
    prefix: LocalStr,
}

impl NativeGroup {
    pub fn new(prefix: &str) -> Self {
        Self {
            natives: HashMap::new(),
            prefix: prefix.to_local_str(),
        }
    }

    pub fn add_child(&mut self, group: &NativeGroup) -> Result<(), IRError> {
        for (k, v) in &group.natives {
            self.add_native(Self::merge_prefix(group.prefix.as_str(), k).as_str(), *v)?;
        }
        Ok(())
    }

    pub fn add_native(&mut self, name: &str, f: NativeHandler) -> Result<(), IRError> {
        if self.natives.contains_key(name) {
            return Err(IRError::BlockAlreadyExists);
        }
        self.natives.insert(name.to_local_str(), f);
        Ok(())
    }

    pub fn apply(&self, ir: &mut IR) -> Result<(), IRError> {
        for (k, v) in &self.natives {
            if self.prefix.is_empty() {
                ir.add_native(k.to_local_str(), *v, true)?;
            } else {
                ir.add_native(
                    Self::merge_prefix(self.prefix.as_str(), k).to_local_str(),
                    *v,
                    true,
                )?;
            }
        }

        Ok(())
    }

    fn merge_prefix<'a>(prefix: &'a str, name: &'a str) -> LocalStr {
        if prefix.is_empty() {
            name.to_local_str()
        } else {
            local_fmt!("{prefix}.{name}")
        }
    }
}
