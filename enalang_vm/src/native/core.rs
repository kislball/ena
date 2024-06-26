use crate::{define_native_group, heap, machine, native};
use enalang_ir as ir;
use flexstr::local_fmt;

pub fn hash(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    let val = ctx.vm.pop()?;
    ctx.vm.push(
        val.get_hash()
            .map(|x| ir::Value::String(local_fmt!("{x}")))
            .unwrap_or(ir::Value::Null),
    )?;
    Ok(())
}

pub fn nop(_: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    Ok(())
}

pub fn drop_value(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    ctx.vm.pop()?;

    Ok(())
}

pub fn peek_value_at(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    let num = ctx.vm.stack.len() - 1 - ctx.vm.pop_pointer()?;
    match ctx.vm.stack.get(num) {
        Some(i) => ctx.vm.push(i.clone()),
        None => Err(machine::VMError::StackEnded),
    }
}

pub fn drop_value_at(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    let num = ctx.vm.pop_pointer()?;
    if ((ctx.vm.stack.len() - 1) - num) >= ctx.vm.stack.len() {
        return Err(machine::VMError::StackEnded);
    }
    let val = ctx.vm.stack.remove((ctx.vm.stack.len() - 1) - num);
    ctx.vm.handle_minus(val)
}

pub fn swap(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    let one = ctx.vm.stack.pop();
    let two = ctx.vm.stack.pop();

    if let Some(a) = one {
        if let Some(b) = two {
            ctx.vm.stack.push(a);
            ctx.vm.stack.push(b);
        }
    }

    Ok(())
}

fn shape_ptr_num_pair(one: ir::Value, two: ir::Value) -> Result<(usize, usize), machine::VMError> {
    match (one, two) {
        (ir::Value::Pointer(a), ir::Value::Number(b)) => Ok((a, b as usize)),
        (ir::Value::Number(a), ir::Value::Pointer(b)) => Ok((a as usize, b)),
        _ => Err(machine::VMError::ExpectedNumber),
    }
}

pub fn plus(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    let popped = (ctx.vm.pop()?, ctx.vm.pop()?);
    if let (ir::Value::Number(a), ir::Value::Number(b)) = popped {
        ctx.vm.push(ir::Value::Number(a + b))?;
    } else if let Ok((a, b)) = shape_ptr_num_pair(popped.0, popped.1) {
        ctx.vm.push(ir::Value::Pointer(a + b))?;
    } else {
        return Err(machine::VMError::ExpectedNumber);
    }

    Ok(())
}

pub fn mul(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let (ir::Value::Number(a), ir::Value::Number(b)) = (ctx.vm.pop()?, ctx.vm.pop()?) {
        ctx.vm.push(ir::Value::Number(a * b))?;
    } else {
        return Err(machine::VMError::ExpectedNumber);
    }

    Ok(())
}

pub fn div(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let (ir::Value::Number(a), ir::Value::Number(b)) = (ctx.vm.pop()?, ctx.vm.pop()?) {
        ctx.vm.push(ir::Value::Number(a / b))?;
    } else {
        return Err(machine::VMError::ExpectedNumber);
    }

    Ok(())
}

pub fn subst(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    let popped = (ctx.vm.pop()?, ctx.vm.pop()?);
    if let (ir::Value::Number(a), ir::Value::Number(b)) = popped {
        ctx.vm.push(ir::Value::Number(a - b))?;
    } else if let Ok((a, b)) = shape_ptr_num_pair(popped.0, popped.1) {
        ctx.vm.push(ir::Value::Pointer(a - b))?;
    } else {
        return Err(machine::VMError::ExpectedNumber);
    }

    Ok(())
}

pub fn pow(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let (ir::Value::Number(a), ir::Value::Number(b)) = (ctx.vm.pop()?, ctx.vm.pop()?) {
        ctx.vm.push(ir::Value::Number(a.powf(b)))?;
    } else {
        return Err(machine::VMError::ExpectedNumber);
    }

    Ok(())
}

pub fn root(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let (ir::Value::Number(a), ir::Value::Number(b)) = (ctx.vm.pop()?, ctx.vm.pop()?) {
        ctx.vm.push(ir::Value::Number(a.powf(1.0 / b)))?;
    } else {
        return Err(machine::VMError::ExpectedNumber);
    }

    Ok(())
}

pub fn dup(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    let val = ctx.vm.stack.last();

    let val = match val {
        Some(i) => i,
        None => {
            return Err(machine::VMError::StackEnded);
        }
    };

    ctx.vm.push(val.clone())?;

    Ok(())
}

pub fn equal(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    let (a, b) = (ctx.vm.pop()?, ctx.vm.pop()?);

    ctx.vm.push(ir::Value::Boolean(a == b))
}

pub fn block_exists(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let ir::Value::Block(name) = ctx.vm.pop()? {
        ctx.vm.stack.push(ir::Value::Boolean(
            ctx.vm.scope_manager.blocks().blocks.contains_key(&name),
        ));
        Ok(())
    } else {
        Err(machine::VMError::ExpectedBlock)
    }
}

pub fn alloc(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    let size = ctx.vm.pop_pointer()?;
    let block: heap::MemoryBlock;

    {
        block = heap::heap_result_into_vm(ctx.vm.heap.alloc(size))?;
    }

    ctx.vm.stack.push(ir::Value::Pointer(block.pointer));

    Ok(())
}

pub fn realloc(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    let new_ptr: usize;

    if let (ir::Value::Pointer(pointer_value), i) = (ctx.vm.pop()?, ctx.vm.pop_pointer()?) {
        new_ptr = heap::heap_result_into_vm(ctx.vm.heap.realloc(pointer_value, i))?;
    } else {
        return Err(machine::VMError::ExpectedPointer);
    }

    ctx.vm.stack.push(ir::Value::Pointer(new_ptr));

    Ok(())
}

pub fn free(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    let pointer = match ctx.vm.pop()? {
        ir::Value::Pointer(i) => i,
        _ => {
            return Err(machine::VMError::ExpectedPointer);
        }
    };

    match ctx.vm.heap.free(pointer) {
        Err(_) => Err(machine::VMError::BadPointer(pointer)),
        _ => Ok(()),
    }
}

pub fn set_ref(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    let ptr = if let ir::Value::Pointer(point) = ctx.vm.pop()? {
        point
    } else {
        return Err(machine::VMError::ExpectedPointer);
    };
    let val = ctx.vm.stack.pop();
    let val = match val {
        Some(i) => i,
        None => return Err(machine::VMError::ExpectedValue),
    };

    ctx.vm
        .heap
        .set(ptr, val.clone())
        .map_err(machine::VMError::HeapError)?;

    ctx.vm.handle_minus(val)?;
    Ok(())
}

pub fn deref(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    let ptrval = match ctx.vm.stack.pop() {
        Some(i) => i,
        None => {
            return Err(machine::VMError::StackEnded);
        }
    };
    let val: ir::Value;

    if let ir::Value::Pointer(value) = ptrval {
        val = ctx.vm.heap.get(value).unwrap_or(ir::Value::Null);
        ctx.vm
            .heap
            .rc_minus(value)
            .map_err(machine::VMError::HeapError)?;
    } else {
        return Err(machine::VMError::ExpectedPointer);
    }

    ctx.vm.push(val)?;

    Ok(())
}

pub fn call(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let ir::Value::Block(name) = ctx.vm.pop()? {
        ctx.vm.run_block(&name)?;
        Ok(())
    } else {
        Err(machine::VMError::ExpectedBlock)
    }
}

pub fn neg(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let ir::Value::Boolean(b) = ctx.vm.pop()? {
        ctx.vm.push(ir::Value::Boolean(!b))
    } else {
        Err(machine::VMError::ExpectedBoolean)
    }
}

pub fn or(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let (ir::Value::Boolean(a), ir::Value::Boolean(b)) = (ctx.vm.pop()?, ctx.vm.pop()?) {
        ctx.vm.push(ir::Value::Boolean(a || b))
    } else {
        Err(machine::VMError::ExpectedBoolean)
    }
}

pub fn and(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let (ir::Value::Boolean(a), ir::Value::Boolean(b)) = (ctx.vm.pop()?, ctx.vm.pop()?) {
        ctx.vm.push(ir::Value::Boolean(a && b))
    } else {
        Err(machine::VMError::ExpectedBoolean)
    }
}

pub fn gt(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let (ir::Value::Number(a), ir::Value::Number(b)) = (ctx.vm.pop()?, ctx.vm.pop()?) {
        ctx.vm.push(ir::Value::Boolean(a > b))
    } else {
        Err(machine::VMError::ExpectedBoolean)
    }
}

pub fn lt(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let (ir::Value::Number(a), ir::Value::Number(b)) = (ctx.vm.pop()?, ctx.vm.pop()?) {
        ctx.vm.push(ir::Value::Boolean(a < b))
    } else {
        Err(machine::VMError::ExpectedBoolean)
    }
}

pub fn lte(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let (ir::Value::Number(a), ir::Value::Number(b)) = (ctx.vm.pop()?, ctx.vm.pop()?) {
        ctx.vm.push(ir::Value::Boolean(a <= b))
    } else {
        Err(machine::VMError::ExpectedBoolean)
    }
}

pub fn gte(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let (ir::Value::Number(a), ir::Value::Number(b)) = (ctx.vm.pop()?, ctx.vm.pop()?) {
        ctx.vm.push(ir::Value::Boolean(a >= b))
    } else {
        Err(machine::VMError::ExpectedBoolean)
    }
}

pub fn clear_stack(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    for value in &ctx.vm.stack.clone() {
        ctx.vm.handle_minus(value.clone())?;
    }

    ctx.vm.stack = Vec::new();

    Ok(())
}

pub fn into_base(_ctx: native::NativeHandlerCtx) {}

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

define_native_group! {
    group,
    "",
    "drop" => drop_value,
    "peek" => peek_value_at,
    "drop_at" => drop_value_at,
    "swap" => swap,
    "dup" => dup,
    "clear" => clear_stack,
    "+" => plus,
    "*" => mul,
    "hash" => hash,
    "/" => div,
    "-" => subst,
    "!" => neg,
    "or" => or,
    "and" => and,
    ">" => gt,
    "<" => lt,
    ">=" => gte,
    "<=" => lte,
    "pow" => pow,
    "nop" => nop,
    "root" => root,
    "==" => equal,
    "call" => call,
    "block_exists?" => block_exists,
    "@" => deref,
    "=" => set_ref,
    "alloc" => alloc,
    "unsafe_realloc" => realloc,
    "unsafe_free" => free
}
