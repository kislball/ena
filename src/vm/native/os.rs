use flexstr::ToLocalStr;

use crate::vm::{ir, machine};
use std::env;

pub fn vm_get_env(ctx: ir::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let ir::Value::String(env_name) = ctx.vm.pop()? {
        match env::var(env_name.as_str()) {
            Ok(st) => ctx.vm.push(ir::Value::String(st.to_local_str())),
            Err(_) => ctx.vm.push(ir::Value::Null)
        }
    } else {
        Err(machine::VMError::ExpectedString)
    }
}

pub fn group() -> ir::NativeGroup {
    let mut group = ir::NativeGroup::new("ena.vm.os");

    group.add_native("get_env", vm_get_env).unwrap();

    group
}