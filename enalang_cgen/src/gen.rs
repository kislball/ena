use enalang_ir::Value;
use flexstr::{local_fmt, LocalStr};

use crate::chead::get_chead;

macro_rules! unreachable {
    () => {
        panic!("unreachable")
    };
}

#[derive(Debug, thiserror::Error)]
pub enum CGenError {}

pub const NATIVE_CALL_PREFIX: &str = "ena.c.";
pub enum CCall {
    Native(LocalStr),
    Mangled(LocalStr),
    PutValue(enalang_ir::Value),
}

pub fn into_c(val: enalang_ir::Value) -> String {
    match val {
        Value::Pointer(ptr) => format!("PUSH_VALUE_POINTER({ptr});"),
        Value::Number(num) => format!("PUSH_VALUE_NUMBER({num});"),
        Value::Boolean(b) => {
            if b {
                String::from("PUSH_VALUE_BOOLEAN(true);")
            } else {
                String::from("PUSH_VALUE_BOOLEAN(false);")
            }
        }
        Value::Null => String::from("PUSH_VALUE_NULL;"),
        Value::Atom(atom) => format!("PUSH_VALUE_ATOM({atom:?});"),
        Value::String(str) => format!("PUSH_VALUE_STRING({str:?});"),
        Value::Block(block) => format!("PUSH_VALUE_BLOCK({block:?});"),
        // exceptions can't be created at compile-time
        // since exceptions are always heap allocated,
        // it is problematic to implement them at compile-time
        Value::Exception(_) => unreachable!(),
    }
}

impl Into<String> for CCall {
    fn into(self) -> String {
        let a: String = match self {
            CCall::Mangled(name) => name.to_string(),
            CCall::Native(name) => name.chars().skip(NATIVE_CALL_PREFIX.len()).collect(),
            CCall::PutValue(val) => into_c(val),
        };

        format!("{a}();")
    }
}

#[derive(Default)]
pub struct CGen {}

impl CGen {
    pub fn new() -> Self {
        Self {}
    }

    fn mangle_name(l: LocalStr) -> LocalStr {
        local_fmt!("_{}", sha256::digest(l.as_str()))
    }

    pub fn compile_ir(&self) -> Result<String, CGenError> {
        Ok(String::new())
    }

    pub fn compile(&self) -> Result<String, CGenError> {
        Ok(format!(
            "// AUTO-GENERATED, DO NOT EDIT\n{head}\n// ENA CGEN OUTPUT BEGIN HERE:\n{body}",
            head = get_chead(),
            body = self.compile_ir()?
        ))
    }
}
