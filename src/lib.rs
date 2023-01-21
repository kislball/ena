use ast::ASTNode;

pub mod ast;
pub mod tok;
pub mod util;
pub mod vm;

#[derive(Debug)]
pub enum EnaError {
    TokenizerError(tok::TokenizerError),
    ASTBuilderError(ast::ASTError),
    CompilerError(vm::compiler::CompilerError),
    VMError(vm::machine::VMError),
    FileDoesNotExist(String),
    ExpectedAllNodesToBePrograms,
}

// Tokenizes, parses, compiles and runs given files.
pub fn run<'a>(file_names: Vec<String>) -> Result<(), EnaError> {
    let mut asts = Vec::<ASTNode>::new();
    for file_name in file_names {
        asts.push(parse_file(&file_name)?);
    }
    let ast = concat_programs(asts)?;
    let mut compiler = vm::compiler::Compiler::new();

    let mut ir = match compiler.compile(&ast) {
        Ok(i) => i,
        Err(e) => return Err(EnaError::CompilerError(e)),
    };

    let io = vm::native::io::group();
    let vm = vm::native::vm::group();

    io.apply(&mut ir).unwrap();
    vm.apply(&mut ir).unwrap();

    let mut vm = vm::machine::VM::new();
    match vm.run_main(&ir) {
        Ok(_) => (),
        Err(e) => return Err(EnaError::VMError(e)),
    }

    Ok(())
}

pub fn concat_programs<'a>(nodes: Vec<ASTNode>) -> Result<ASTNode, EnaError> {
    let mut final_nodes = Vec::<ASTNode>::new();
    let mut inc = 0;

    for node in nodes {
        if let ast::ASTNodeInner::Block(ast::BlockType::Program, block_nodes) = node.1 {
            for block_node in &block_nodes {
                let inner = &block_node.1;
                final_nodes.push(ASTNode(inc + block_node.0, inner.clone()));
            }

            let last_node = block_nodes.last().unwrap();
            inc = last_node.0;
        } else {
            return Err(EnaError::ExpectedAllNodesToBePrograms);
        }
    }

    Ok(ASTNode(
        0,
        ast::ASTNodeInner::Block(ast::BlockType::Program, final_nodes),
    ))
}

pub fn parse_file<'a>(file_name: &String) -> Result<ASTNode, EnaError> {
    let mut tokenizer = tok::Tokenizer::new();
    let content = match std::fs::read_to_string(file_name) {
        Ok(i) => i,
        Err(_) => return Err(EnaError::FileDoesNotExist(file_name.to_string())),
    };
    let tokens = tokenizer.parse(content);
    let tokens = match tokens {
        Ok(i) => i,
        Err(e) => return Err(EnaError::TokenizerError(e)),
    };

    let mut builder = ast::ASTBuilder::new();
    let nodes = match builder.parse(tokens) {
        Ok(i) => i,
        Err(e) => return Err(EnaError::ASTBuilderError(e)),
    };

    Ok(nodes)
}
