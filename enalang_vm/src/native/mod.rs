use crate::machine;
use enalang_ir as ir;
use flexstr::{local_fmt, LocalStr, ToLocalStr};
use std::collections::HashMap;

pub mod core;
pub mod exceptions;
pub mod io;
pub mod os;
pub mod strings;
pub mod types;
pub mod vm;

pub struct NativeHandlerCtx<'a> {
    pub vm: &'a mut machine::VM,
}

pub type NativeHandler = fn(ctx: NativeHandlerCtx) -> Result<(), machine::VMError>;

pub struct NativeGroup {
    pub natives: HashMap<LocalStr, NativeHandler>,
    pub prefix: LocalStr,
}

impl NativeGroup {
    pub fn new(prefix: &str) -> Self {
        Self {
            natives: HashMap::new(),
            prefix: prefix.to_local_str(),
        }
    }

    pub fn add_child(&mut self, group: &NativeGroup) -> Result<(), ir::IRError> {
        for (k, v) in &group.natives {
            self.add_native(Self::merge_prefix(&group.prefix, k).as_str(), *v)?;
        }
        Ok(())
    }

    pub fn add_native(&mut self, name: &str, f: NativeHandler) -> Result<(), ir::IRError> {
        if self.natives.contains_key(name) {
            return Err(ir::IRError::BlockAlreadyExists(name.to_local_str()));
        }
        self.natives.insert(name.to_local_str(), f);
        Ok(())
    }

    // pub fn apply(&self, ir: &mut ir::IR) -> Result<(), ir::IRError> {
    //     for (k, v) in &self.natives {
    //         if self.prefix.is_empty() {
    //             ir.add_native(k.to_local_str(), *v, true)?;
    //         } else {
    //             ir.add_native(
    //                 Self::merge_prefix(self.prefix.as_str(), k).to_local_str(),
    //                 *v,
    //                 true,
    //             )?;
    //         }
    //     }

    //     Ok(())
    // }

    pub fn merge_prefix(prefix: &LocalStr, name: &LocalStr) -> LocalStr {
        if prefix.is_empty() {
            name.clone()
        } else {
            local_fmt!("{prefix}.{name}")
        }
    }
}

pub fn group() -> NativeGroup {
    let mut group = NativeGroup::new("");

    group.add_child(&vm::group()).unwrap();
    group.add_child(&io::group()).unwrap();
    group.add_child(&core::group()).unwrap();
    group.add_child(&types::group()).unwrap();
    group.add_child(&exceptions::group()).unwrap();
    group.add_child(&strings::group()).unwrap();
    group.add_child(&os::group()).unwrap();

    group
}
