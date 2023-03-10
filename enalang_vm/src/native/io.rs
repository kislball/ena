use crate::{machine, native};
use enalang_ir as ir;
use flexstr::{shared_fmt, ToSharedStr};
use std::{fs, path::Path};

pub fn print(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let ir::Value::String(st) = ctx.vm.pop()? {
        print!("{st}");
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
                    shared_fmt!("{e:?}"),
                )));
            }
        };
        let mut total_files = 0;
        for file in files {
            let file = match file {
                Ok(i) => i,
                Err(e) => {
                    return Err(machine::VMError::RuntimeException(ir::Value::String(
                        shared_fmt!("{e:?}"),
                    )));
                }
            };
            ctx.vm.push(ir::Value::String(
                file.file_name().to_str().unwrap().to_shared_str(),
            ))?;
            total_files += 1;
        }
        ctx.vm.push(ir::Value::Number(total_files as f64))?;
        Ok(())
    } else {
        Err(machine::VMError::ExpectedString)
    }
}

pub fn group() -> native::NativeGroup {
    let mut group = native::NativeGroup::new("ena.vm.io");

    group.add_native("print", print).unwrap();
    group.add_native("file_exists?", file_exists).unwrap();
    group.add_native("list_files_in_dir", files_in_dir).unwrap();

    group
}
