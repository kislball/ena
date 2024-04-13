use crate::{define_native_group, machine::{self, VMError}, native};
use enalang_ir as ir;
use flexstr::{local_fmt, ToLocalStr};
use ir::Value;
use std::{fs, path::Path};

pub fn print(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let ir::Value::String(st) = ctx.vm.pop()? {
        print!("{st}");
    } else {
        return Err(machine::VMError::ExpectedString);
    }

    Ok(())
}

pub fn read_file(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let ir::Value::String(st) = ctx.vm.pop()? {
        let str = fs::read_to_string::<String>(format!("{st}"))
            .map_err(|x| VMError::FS(local_fmt!("{x}")))?;
        ctx.vm.push(Value::String(local_fmt!("{str}")))?;
    } else {
        return Err(machine::VMError::ExpectedString);
    }

    Ok(())
}

pub fn file_exists(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let ir::Value::String(st) = ctx.vm.pop()? {
        ctx.vm
            .push(ir::Value::Boolean(Path::new(&st.as_str()).is_file()))
    } else {
        Err(machine::VMError::ExpectedString)
    }
}

pub fn files_in_dir(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let ir::Value::String(st) = ctx.vm.pop()? {
        let res = fs::read_dir(st.as_str());
        let files = match res {
            Ok(i) => i,
            Err(e) => {
                return Err(machine::VMError::RuntimeException(ir::Value::String(
                    local_fmt!("{e:?}"),
                )));
            }
        };
        let mut total_files = 0;
        for file in files {
            let file = match file {
                Ok(i) => i,
                Err(e) => {
                    return Err(machine::VMError::RuntimeException(ir::Value::String(
                        local_fmt!("{e:?}"),
                    )));
                }
            };
            ctx.vm.push(ir::Value::String(
                file.file_name().to_str().unwrap().to_local_str(),
            ))?;
            total_files += 1;
        }
        ctx.vm.push(ir::Value::Number(total_files as f64))?;
        Ok(())
    } else {
        Err(machine::VMError::ExpectedString)
    }
}

define_native_group! {
    group,
    "ena.vm.io",
    "print" => print,
    "read_file" => read_file,
    "file_exists?" => file_exists,
    "list_files_in_dir" => files_in_dir
}
