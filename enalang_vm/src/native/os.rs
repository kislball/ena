use crate::{define_native_group, machine, native};
use enalang_ir as ir;
use flexstr::ToLocalStr;
use std::env;

pub fn vm_get_env(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let ir::Value::String(env_name) = ctx.vm.pop()? {
        match env::var(env_name.as_str()) {
            Ok(st) => ctx.vm.push(ir::Value::String(st.to_local_str())),
            Err(_) => ctx.vm.push(ir::Value::Null),
        }
    } else {
        Err(machine::VMError::ExpectedString)
    }
}

define_native_group! {
    group,
    "ena.vm.os",
    "get_env" => vm_get_env
}
