use enalang_compiler::ir;
use crate::native;
use core::fmt::Debug;
use flexstr::LocalStr;
use std::collections::HashMap;

#[derive(Debug)]
pub enum BlocksError {
    BlockAlreadyExists,
}

impl Into<ir::IRError> for BlocksError {
    fn into(self) -> ir::IRError {
        match self {
            BlocksError::BlockAlreadyExists => ir::IRError::BlockAlreadyExists,
        }
    }
}

#[derive(Clone)]
pub enum VMBlock {
    NativeHandler(native::NativeHandler),
    IR(ir::Block),
}

impl Debug for VMBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NativeHandler(_) => f.write_str("NativeHandler"),
            Self::IR(arg0) => f.debug_tuple("IR").field(arg0).finish(),
        }
    }
}

impl VMBlock {
    pub fn is_single_eval(&self) -> bool {
        match self {
            VMBlock::NativeHandler(_) => false,
            VMBlock::IR(block) => block.is_single_eval(),
        }
    }

    pub fn is_global(&self) -> bool {
        match self {
            VMBlock::NativeHandler(_) => true,
            VMBlock::IR(block) => block.is_global(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Blocks {
    pub blocks: HashMap<LocalStr, VMBlock>,
    pub annotations: HashMap<LocalStr, LocalStr>,
}

impl Default for Blocks {
    fn default() -> Self {
        Self {
            blocks: HashMap::new(),
            annotations: HashMap::new(),
        }
    }
}

impl Blocks {
    pub fn new(native: native::NativeGroup, ir: ir::IR) -> Result<Self, BlocksError> {
        let mut default = Self::default();

        default.add_ir(ir)?;
        default.add_native(native)?;

        Ok(default)
    }

    pub fn get_block(&self, name: &LocalStr) -> Option<&VMBlock> {
        self.blocks.get(name)
    }

    pub fn add_block(&mut self, name: LocalStr, block: VMBlock) -> Result<(), BlocksError> {
        if self.blocks.contains_key(&name) {
            return Err(BlocksError::BlockAlreadyExists);
        }
        self.blocks.insert(name, block);
        Ok(())
    }

    pub fn add_native(&mut self, native: native::NativeGroup) -> Result<(), BlocksError> {
        for (name, f) in native.natives {
            self.add_block(name, VMBlock::NativeHandler(f))?;
        }
        Ok(())
    }

    pub fn add_ir(&mut self, ir: ir::IR) -> Result<(), BlocksError> {
        for (name, block) in ir.blocks {
            self.add_block(name, VMBlock::IR(block))?;
        }

        for (name, annotation) in ir.annotations {
            self.annotations.insert(name, annotation);
        }
        Ok(())
    }
}
