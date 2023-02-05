use crate::ir;

pub mod core;
pub mod exceptions;
pub mod io;
pub mod os;
pub mod strings;
pub mod types;
pub mod vm;

pub fn group() -> ir::NativeGroup {
    let mut group = ir::NativeGroup::new("");

    group.add_child(&vm::group()).unwrap();
    group.add_child(&io::group()).unwrap();
    group.add_child(&core::group()).unwrap();
    group.add_child(&types::group()).unwrap();
    group.add_child(&exceptions::group()).unwrap();
    group.add_child(&strings::group()).unwrap();
    group.add_child(&os::group()).unwrap();

    group
}
