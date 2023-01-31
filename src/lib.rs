pub mod ast;
pub mod tok;
pub mod util;
pub mod vm;

#[derive(Debug)]
pub enum EnaError {
    TokenizerError(tok::TokenizerError),
    ASTError(ast::ASTError),
    CompilerError(vm::compiler::CompilerError),
    IRError(vm::ir::IRError),
    SerializationError(vm::ir::SerializationError),
}

pub fn merge<'a>(irs: &Vec<vm::ir::IR<'a>>) -> Result<vm::ir::IR<'a>, EnaError> {
    let mut ir = vm::ir::IR::new();

    for sub_ir in irs {
        ir.add(sub_ir).map_err(EnaError::IRError)?;
    }

    Ok(ir)
}

pub fn compile_many<'a>(contents: &Vec<String>) -> Result<vm::ir::IR<'a>, EnaError> {
    let mut ir = vm::ir::IR::new();

    for content in contents {
        ir.add(&compile(content)?).map_err(EnaError::IRError)?;
    }

    Ok(ir)
}

pub fn compile<'a>(contents: &String) -> Result<vm::ir::IR<'a>, EnaError> {
    let mut tokenizer = tok::Tokenizer::new();
    let tokens = tokenizer
        .parse(contents.to_string())
        .map_err(EnaError::TokenizerError)?;
    let mut ast_builder = ast::ASTBuilder::new();
    let parsed = Box::leak(Box::new(
        ast_builder.parse(tokens).map_err(EnaError::ASTError)?,
    ));
    let compiler = Box::leak(Box::new(vm::compiler::Compiler::new()));
    let ir = compiler.compile(parsed).map_err(EnaError::CompilerError)?;
    Ok(ir)
}
