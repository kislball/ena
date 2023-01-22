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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, clap::ValueEnum)]
pub enum Stage {
    Parse,
    Compile,
    CompileExtended,
    Run,
}

pub struct RunOptions {
    pub stage: Stage,
    pub file_names: Vec<String>,
    pub main: String,
    pub debug_stack: bool,
}

// Tokenizes, parses, compiles and runs given files.
pub fn run<'a>(options: &RunOptions) -> Result<(), EnaError> {
    let mut asts = Vec::<ASTNode>::new();
    for file_name in &options.file_names {
        asts.push(parse_file(&file_name)?);
    }
    let ast = concat_programs(asts)?;

    if options.stage == Stage::Parse {
        println!("{:#?}", ast);
        return Ok(());
    }

    let mut compiler = vm::compiler::Compiler::new();

    let mut ir = match compiler.compile(&ast) {
        Ok(i) => i,
        Err(e) => return Err(EnaError::CompilerError(e)),
    };

    if options.stage == Stage::Compile {
        println!("{:#?}", ir);
        return Ok(());
    }

    let native = vm::native::group();
    native.apply(&mut ir).unwrap(); // we should panic if stuff like this happens

    if options.stage == Stage::CompileExtended {
        println!("{:#?}", ir);
        return Ok(());
    }

    let mut vm = vm::machine::VM::new();
    vm.debug_stack = options.debug_stack;
    match vm.run(&ir, options.main.as_str()) {
        Ok(_) => (),
        Err(e) => {
            vm.print_call_stack();
            return Err(EnaError::VMError(e))
        },
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
