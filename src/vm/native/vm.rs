use crate::vm::{ir, machine};
use rand::{self, Rng};

pub fn vm_debug(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    let el = match vm.stack.pop() {
        Some(i) => i,
        None => ir::Value::Null,
    };

    println!("{:?}", el);

    Ok(())
}

pub fn vm_get_random(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    vm.stack.push(ir::Value::Number(
        rand::thread_rng().gen_range(0.0..=1.0) as f64
    ));
    Ok(())
}

pub fn group<'a>() -> ir::NativeGroup<'a> {
    let mut group = ir::NativeGroup::new();

    group.add_native("ena.vm.debug", vm_debug).unwrap();
    group.add_native("ena.vm.random", vm_get_random).unwrap();

    group
}
