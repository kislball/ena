use flexstr::LocalStr;
use flexstr::ToLocalStr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum IRError {
    #[error("block already exists")]
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
            blocks.push(IRSerializable::Block(
                name.as_str(),
                block.global,
                block.run_type,
                block.code.clone(),
            ));
        }

        for (block, content) in &self.annotations {
            blocks.push(IRSerializable::Annotation(block.clone(), content.clone()));
        }

        IRSerializable::Root(blocks)
    }

    pub fn add(&mut self, another: &IR) -> Result<(), IRError> {
        for (name, block) in &another.blocks {
            self.add_block(name.clone(), block.clone(), true)?;
        }

        for (name, comment) in &another.annotations {
            self.annotations.insert(name.clone(), comment.clone());
        }
        Ok(())
    }

    pub fn get_block(&self, id: &LocalStr) -> Option<&Block> {
        self.blocks.get(id)
    }

    pub fn add_block(
        &mut self,
        name: LocalStr,
        block: Block,
        output_err: bool,
    ) -> Result<(), IRError> {
        if self.blocks.contains_key(&name) && output_err {
            return Err(IRError::BlockAlreadyExists);
        }
        self.blocks.insert(name, block);
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum BlockRunType {
    Once,
    Unique,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub global: bool,
    pub run_type: BlockRunType,
    pub code: Vec<IRCode>,
}

impl Block {
    pub fn is_single_eval(&self) -> bool {
        matches!(self.run_type, BlockRunType::Once)
    }

    pub fn is_global(&self) -> bool {
        self.global
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum IRSerializable<'a> {
    Block(&'a str, bool, BlockRunType, Vec<IRCode>),
    Root(Vec<IRSerializable<'a>>),
    Annotation(LocalStr, LocalStr),
}

#[derive(Debug, thiserror::Error)]
pub enum SerializationError {
    #[error("expected root")]
    ExpectedRoot,
    #[error("expected block")]
    ExpectedBlock,
    #[error("bincode error - `{0}`")]
    BincodeErr(bincode::ErrorKind),
    #[error("ir error - `{0}`")]
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
                if let IRSerializable::Block(name, global, typ, data) = ser_block {
                    let block = Block {
                        code: data,
                        global,
                        run_type: typ,
                    };
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
    Exception(Box<Value>),
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
    ReturnLocal,
}
