use crate::vm::{heap, ir, machine};

pub fn drop_value(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    vm.pop()?;

    Ok(())
}

pub fn swap(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    let one = vm.stack.pop();
    let two = vm.stack.pop();

    if let Some(a) = one {
        if let Some(b) = two {
            vm.stack.push(a);
            vm.stack.push(b);
        }
    }

    Ok(())
}

pub fn plus(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    let popped = (vm.pop()?, vm.pop()?);
    if let (ir::Value::Number(a), ir::Value::Number(b)) = popped {
        vm.push(ir::Value::Number(a + b))?;
    } else if let (ir::Value::Pointer(a), ir::Value::Number(b)) = popped {
        let old_b = b;
        let b = b as usize;

        if old_b != b as f64 {
            return Err(machine::VMError::BadPointer(
                "+(expected integer)".to_string(),
            ));
        }

        vm.push(ir::Value::Pointer(a + b))?;
    } else {
        return Err(machine::VMError::ExpectedNumber("+".to_string()));
    }

    Ok(())
}

pub fn mul(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    if let (ir::Value::Number(a), ir::Value::Number(b)) = (vm.pop()?, vm.pop()?) {
        vm.push(ir::Value::Number(a * b))?;
    } else {
        return Err(machine::VMError::ExpectedNumber("*".to_string()));
    }

    Ok(())
}

pub fn div(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    if let (ir::Value::Number(a), ir::Value::Number(b)) = (vm.pop()?, vm.pop()?) {
        vm.push(ir::Value::Number(a / b))?;
    } else {
        return Err(machine::VMError::ExpectedNumber("/".to_string()));
    }

    Ok(())
}

pub fn subst(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    let popped = (vm.pop()?, vm.pop()?);
    if let (ir::Value::Number(a), ir::Value::Number(b)) = popped {
        vm.push(ir::Value::Number(a - b))?;
    } else if let (ir::Value::Pointer(a), ir::Value::Number(b)) = popped {
        let old_b = b;
        let b = b as usize;

        if old_b != b as f64 {
            return Err(machine::VMError::BadPointer(
                "-(expected integer)".to_string(),
            ));
        }

        if a < b {
            return Err(machine::VMError::BadPointer(
                "-(going negative)".to_string(),
            ));
        }

        vm.push(ir::Value::Pointer(a - b))?;
    } else {
        return Err(machine::VMError::ExpectedNumber("-".to_string()));
    }

    Ok(())
}

pub fn pow(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    if let (ir::Value::Number(a), ir::Value::Number(b)) = (vm.pop()?, vm.pop()?) {
        vm.push(ir::Value::Number(a.powf(b)))?;
    } else {
        return Err(machine::VMError::ExpectedNumber("pow".to_string()));
    }

    Ok(())
}

pub fn root(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    if let (ir::Value::Number(a), ir::Value::Number(b)) = (vm.pop()?, vm.pop()?) {
        vm.push(ir::Value::Number(a.powf(1.0 / b)))?;
    } else {
        return Err(machine::VMError::ExpectedNumber("pow".to_string()));
    }

    Ok(())
}

pub fn dup(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    let val = vm.stack.last();

    let val = match val {
        Some(i) => i,
        None => {
            return Err(machine::VMError::StackEnded("dup".to_string()));
        }
    };

    vm.push(val.clone())?;

    Ok(())
}

pub fn equal(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    let (a, b) = (vm.pop()?, vm.pop()?);
    vm.push(ir::Value::Boolean(a == b))?;
    Ok(())
}

pub fn block_exists(vm: &mut machine::VM, ir: &ir::IR) -> Result<(), machine::VMError> {
    if let ir::Value::Block(name) = vm.pop()? {
        vm.stack
            .push(ir::Value::Boolean(ir.blocks.contains_key(&name)));
        Ok(())
    } else {
        Err(machine::VMError::ExpectedBlock("block_exists?".to_string()))
    }
}

pub fn alloc(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    let size = vm.pop_usize()?;
    let block: heap::MemoryBlock;

    {
        block = heap::heap_result_into_vm(vm.heap.alloc(size))?;
    }

    vm.push(ir::Value::Pointer(block.pointer))?;

    Ok(())
}

pub fn realloc(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    let new_ptr: usize;

    if let (ir::Value::Pointer(pointer_value), i) = (vm.pop()?, vm.pop_usize()?) {
        new_ptr = heap::heap_result_into_vm(vm.heap.realloc(pointer_value, i))?;
    } else {
        return Err(machine::VMError::ExpectedPointer("realloc".to_string()));
    }

    vm.push(ir::Value::Pointer(new_ptr))?;

    Ok(())
}

pub fn free(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    let pointer = match vm.pop()? {
        ir::Value::Pointer(i) => i,
        _ => {
            return Err(machine::VMError::ExpectedPointer("free".to_string()));
        }
    };

    match vm.heap.free(pointer) {
        Err(_) => Err(machine::VMError::BadPointer("free".to_string())),
        _ => Ok(()),
    }
}

pub fn set_ref(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    let ptr: usize;
    let val: ir::Value;

    if let (ir::Value::Pointer(value), b) = (vm.pop()?, vm.pop()?) {
        ptr = value;
        val = b;
    } else {
        return Err(machine::VMError::ExpectedTwo("expected pointer and value"));
    }

    vm.heap
        .set(ptr, val)
        .map_err(|x| machine::VMError::HeapError(x))?;

    Ok(())
}

pub fn deref(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    let ptrval = match vm.stack.pop() {
        Some(i) => i,
        None => {
            return Err(machine::VMError::StackEnded("deref".to_string()));
        }
    };
    let val: ir::Value;

    if let ir::Value::Pointer(value) = ptrval {
        val = vm.heap.get(value).unwrap_or(ir::Value::Null);
        vm.heap
            .rc_minus(value)
            .map_err(|err| machine::VMError::HeapError(err))?;
    } else {
        return Err(machine::VMError::ExpectedPointer("@".to_string()));
    }

    vm.push(val)?;

    Ok(())
}

pub fn into_ptr<'a>(vm: &mut machine::VM, _: &ir::IR<'a>) -> Result<(), machine::VMError> {
    if let ir::Value::Number(num) = vm.pop()? {
        let ptr = num as usize;

        if num != ptr as f64 {
            return Err(machine::VMError::BadPointer("into_ptr".to_string()));
        }

        vm.push(ir::Value::Pointer(ptr))?;

        Ok(())
    } else {
        Err(machine::VMError::ExpectedNumber("into_ptr".to_string()))
    }
}

pub fn call<'a>(vm: &mut machine::VM, ir: &ir::IR<'a>) -> Result<(), machine::VMError> {
    if let ir::Value::Block(name) = vm.pop()? {
        vm.run_block(name, ir)?;
        Ok(())
    } else {
        Err(machine::VMError::ExpectedBlock("call".to_string()))
    }
}

// pub fn run_thread<'a>(vm: &mut machine::VM, ir: &ir::IR<'a>) -> Result<(), machine::VMError> {
//     if let ir::Value::Block(name) = vm.pop()? {
//         let n_name = name.clone();
//         thread::spawn(|| {
//             let mut new_vm = machine::VM::new();
//             new_vm.run_block(n_name, &ir.clone());
//         });
//         Ok(())
//     } else {
//         Err(machine::VMError::ExpectedBlock("run_thread".to_string()))
//     }
// }

pub fn group<'a>() -> ir::NativeGroup<'a> {
    let mut group = ir::NativeGroup::new("");

    group.add_native("drop", drop_value).unwrap();
    group.add_native("swap", swap).unwrap();
    group.add_native("dup", dup).unwrap();
    group.add_native("+", plus).unwrap();
    group.add_native("*", mul).unwrap();
    group.add_native("/", div).unwrap();
    group.add_native("-", subst).unwrap();
    group.add_native("pow", pow).unwrap();
    group.add_native("root", root).unwrap();
    group.add_native("==", equal).unwrap();
    group.add_native("call", call).unwrap();
    group.add_native("block_exists?", block_exists).unwrap();
    group.add_native("@", deref).unwrap();
    group.add_native("=", set_ref).unwrap();
    group.add_native("into_ptr", into_ptr).unwrap();
    group.add_native("unsafe_alloc", alloc).unwrap();
    group.add_native("unsafe_realloc", realloc).unwrap();
    group.add_native("unsafe_free", free).unwrap();

    group
}
