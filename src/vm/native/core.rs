use std::collections::HashMap;

use flexstr::LocalStr;

use crate::vm::{heap, ir, machine};

pub fn drop_value(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    vm.pop()?;

    Ok(())
}

pub fn peek_value_at(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    let num = vm.pop_usize()?;
    match vm.stack.get(num) {
        Some(i) => vm.push(i.clone()),
        None => Err(machine::VMError::StackEnded),
    }
}

pub fn drop_value_at(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    let num = vm.pop_usize()?;
    if ((vm.stack.len() - 1) - num) >= vm.stack.len() {
        return Err(machine::VMError::StackEnded);
    }
    let val = vm.stack.remove((vm.stack.len() - 1) - num);
    vm.handle_minus(val)
}

pub fn swap(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
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

pub fn plus(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    let popped = (vm.pop()?, vm.pop()?);
    if let (ir::Value::Number(a), ir::Value::Number(b)) = popped {
        vm.push(ir::Value::Number(a + b))?;
    } else if let (ir::Value::Pointer(a), ir::Value::Number(b)) = popped {
        let old_b = b;
        let b = b as usize;

        if old_b != b as f64 {
            return Err(machine::VMError::BadPointer(b));
        }

        vm.push(ir::Value::Pointer(a + b))?;
    } else {
        return Err(machine::VMError::ExpectedNumber);
    }

    Ok(())
}

pub fn mul(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    if let (ir::Value::Number(a), ir::Value::Number(b)) = (vm.pop()?, vm.pop()?) {
        vm.push(ir::Value::Number(a * b))?;
    } else {
        return Err(machine::VMError::ExpectedNumber);
    }

    Ok(())
}

pub fn div(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    if let (ir::Value::Number(a), ir::Value::Number(b)) = (vm.pop()?, vm.pop()?) {
        vm.push(ir::Value::Number(a / b))?;
    } else {
        return Err(machine::VMError::ExpectedNumber);
    }

    Ok(())
}

pub fn subst(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    let popped = (vm.pop()?, vm.pop()?);
    if let (ir::Value::Number(a), ir::Value::Number(b)) = popped {
        vm.push(ir::Value::Number(a - b))?;
    } else if let (ir::Value::Pointer(a), ir::Value::Number(b)) = popped {
        let old_b = b;
        let b = b as usize;

        if old_b != b as f64 {
            return Err(machine::VMError::BadPointer(b));
        }

        if a < b {
            return Err(machine::VMError::BadPointer(b));
        }

        vm.push(ir::Value::Pointer(a - b))?;
    } else {
        return Err(machine::VMError::ExpectedNumber);
    }

    Ok(())
}

pub fn pow(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    if let (ir::Value::Number(a), ir::Value::Number(b)) = (vm.pop()?, vm.pop()?) {
        vm.push(ir::Value::Number(a.powf(b)))?;
    } else {
        return Err(machine::VMError::ExpectedNumber);
    }

    Ok(())
}

pub fn root(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    if let (ir::Value::Number(a), ir::Value::Number(b)) = (vm.pop()?, vm.pop()?) {
        vm.push(ir::Value::Number(a.powf(1.0 / b)))?;
    } else {
        return Err(machine::VMError::ExpectedNumber);
    }

    Ok(())
}

pub fn dup(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    let val = vm.stack.last();

    let val = match val {
        Some(i) => i,
        None => {
            return Err(machine::VMError::StackEnded);
        }
    };

    vm.push(val.clone())?;

    Ok(())
}

pub fn equal(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    let (a, b) = (vm.pop()?, vm.pop()?);

    vm.push(ir::Value::Boolean(a == b))
}

pub fn block_exists(
    vm: &mut machine::VM,
    ir: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    if let ir::Value::Block(name) = vm.pop()? {
        vm.stack
            .push(ir::Value::Boolean(ir.blocks.contains_key(&name)));
        Ok(())
    } else {
        Err(machine::VMError::ExpectedBlock)
    }
}

pub fn alloc(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    let size = vm.pop_usize()?;
    let block: heap::MemoryBlock;

    {
        block = heap::heap_result_into_vm(vm.heap.alloc(size))?;
    }

    vm.stack.push(ir::Value::Pointer(block.pointer));

    Ok(())
}

pub fn realloc(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    let new_ptr: usize;

    if let (ir::Value::Pointer(pointer_value), i) = (vm.pop()?, vm.pop_usize()?) {
        new_ptr = heap::heap_result_into_vm(vm.heap.realloc(pointer_value, i))?;
    } else {
        return Err(machine::VMError::ExpectedPointer);
    }

    vm.stack.push(ir::Value::Pointer(new_ptr));

    Ok(())
}

pub fn free(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    let pointer = match vm.pop()? {
        ir::Value::Pointer(i) => i,
        _ => {
            return Err(machine::VMError::ExpectedPointer);
        }
    };

    match vm.heap.free(pointer) {
        Err(_) => Err(machine::VMError::BadPointer(pointer)),
        _ => Ok(()),
    }
}

pub fn set_ref(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    let ptr = if let ir::Value::Pointer(point) = vm.pop()? {
        point
    } else {
        return Err(machine::VMError::ExpectedPointer);
    };
    let val = vm.pop()?;

    vm.heap.set(ptr, val).map_err(machine::VMError::HeapError)?;

    Ok(())
}

pub fn deref(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    let ptrval = match vm.stack.pop() {
        Some(i) => i,
        None => {
            return Err(machine::VMError::StackEnded);
        }
    };
    let val: ir::Value;

    if let ir::Value::Pointer(value) = ptrval {
        val = vm.heap.get(value).unwrap_or(ir::Value::Null);
        vm.heap
            .rc_minus(value)
            .map_err(machine::VMError::HeapError)?;
    } else {
        return Err(machine::VMError::ExpectedPointer);
    }

    vm.push(val)?;

    Ok(())
}

pub fn call(
    vm: &mut machine::VM,
    ir: &ir::IR,
    locals: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    if let ir::Value::Block(name) = vm.pop()? {
        vm.run_block(name, ir, locals)?;
        Ok(())
    } else {
        Err(machine::VMError::ExpectedBlock)
    }
}

pub fn neg(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    if let ir::Value::Boolean(b) = vm.pop()? {
        vm.push(ir::Value::Boolean(!b))
    } else {
        Err(machine::VMError::ExpectedBoolean)
    }
}

pub fn or(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    if let (ir::Value::Boolean(a), ir::Value::Boolean(b)) = (vm.pop()?, vm.pop()?) {
        vm.push(ir::Value::Boolean(a || b))
    } else {
        Err(machine::VMError::ExpectedBoolean)
    }
}

pub fn and(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    if let (ir::Value::Boolean(a), ir::Value::Boolean(b)) = (vm.pop()?, vm.pop()?) {
        vm.push(ir::Value::Boolean(a && b))
    } else {
        Err(machine::VMError::ExpectedBoolean)
    }
}

pub fn gt(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    if let (ir::Value::Number(a), ir::Value::Number(b)) = (vm.pop()?, vm.pop()?) {
        vm.push(ir::Value::Boolean(a > b))
    } else {
        Err(machine::VMError::ExpectedBoolean)
    }
}

pub fn lt(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    if let (ir::Value::Number(a), ir::Value::Number(b)) = (vm.pop()?, vm.pop()?) {
        vm.push(ir::Value::Boolean(a < b))
    } else {
        Err(machine::VMError::ExpectedBoolean)
    }
}

pub fn lte(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    if let (ir::Value::Number(a), ir::Value::Number(b)) = (vm.pop()?, vm.pop()?) {
        vm.push(ir::Value::Boolean(a <= b))
    } else {
        Err(machine::VMError::ExpectedBoolean)
    }
}

pub fn gte(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    if let (ir::Value::Number(a), ir::Value::Number(b)) = (vm.pop()?, vm.pop()?) {
        vm.push(ir::Value::Boolean(a >= b))
    } else {
        Err(machine::VMError::ExpectedBoolean)
    }
}

pub fn clear_stack(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    for value in &vm.stack.clone() {
        vm.handle_minus(value.clone())?;
    }

    vm.stack = Vec::new();

    Ok(())
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

pub fn group() -> ir::NativeGroup {
    let mut group = ir::NativeGroup::new("");

    group.add_native("drop", drop_value).unwrap();
    group.add_native("peek", peek_value_at).unwrap();
    group.add_native("drop_at", drop_value_at).unwrap();
    group.add_native("swap", swap).unwrap();
    group.add_native("dup", dup).unwrap();
    group.add_native("clear", clear_stack).unwrap();
    group.add_native("+", plus).unwrap();
    group.add_native("*", mul).unwrap();
    group.add_native("/", div).unwrap();
    group.add_native("-", subst).unwrap();
    group.add_native("!", neg).unwrap();
    group.add_native("or", or).unwrap();
    group.add_native("and", and).unwrap();
    group.add_native(">", gt).unwrap();
    group.add_native("<", lt).unwrap();
    group.add_native(">=", gte).unwrap();
    group.add_native("<=", lte).unwrap();
    group.add_native("pow", pow).unwrap();
    group.add_native("root", root).unwrap();
    group.add_native("==", equal).unwrap();
    group.add_native("call", call).unwrap();
    group.add_native("block_exists?", block_exists).unwrap();
    group.add_native("@", deref).unwrap();
    group.add_native("=", set_ref).unwrap();
    group.add_native("unsafe_alloc", alloc).unwrap();
    group.add_native("unsafe_realloc", realloc).unwrap();
    group.add_native("unsafe_free", free).unwrap();

    group
}
