use crate::vm::ir;
use crate::{ast, tok};
use rand::distributions::{Alphanumeric, DistString};

pub struct Compiler {}

#[derive(Debug)]
pub enum CompilerErrorInner {
    ExpectedProgramAsRoot,
    UnexpectedBlock,
    UnexpectedNode,
    ExpectedBlock,
    UnexpectedAnonymousBlock,
    ExpectedUnescapedBlock,
    ExpectedUniqueEvalBlockAfterIf,
    WordAlreadyExists,
}

#[derive(Debug)]
pub struct CompilerError(usize, CompilerErrorInner);

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Compiler {
    pub fn new() -> Self {
        Compiler {}
    }

    pub fn compile(&'a mut self, ast: &'a ast::ASTNode) -> Result<ir::IR, CompilerError> {
        let mut ir = ir::IR::new();
        let nodes = match &ast.1 {
            ast::ASTNodeInner::Block(ast::BlockType::Program, nodes) => nodes,
            _ => return Err(CompilerError(0, CompilerErrorInner::ExpectedProgramAsRoot)),
        };

        {
            for (i, node) in nodes.iter().enumerate() {
                match &node.1 {
                    ast::ASTNodeInner::Identifier(id) => {
                        let next = match nodes.get(i + 1) {
                            Some(i) => i,
                            None => {
                                return Err(CompilerError(i, CompilerErrorInner::ExpectedBlock));
                            }
                        };

                        if let ast::ASTNodeInner::Block(_, _) = next.1 {
                            match self.compile_block(id.as_str(), next, &mut ir) {
                                Err(e) => {
                                    return Err(e);
                                }
                                Ok(v) => match ir.add_block(id, v) {
                                    Err(ir::IRError::WordAlreadyExists) => {
                                        return Err(CompilerError(
                                            i,
                                            CompilerErrorInner::WordAlreadyExists,
                                        ));
                                    }
                                    _ => {}
                                },
                            };
                        } else {
                            return Err(CompilerError(i, CompilerErrorInner::ExpectedBlock));
                        }
                    }
                    ast::ASTNodeInner::Block(typ, _) => {
                        let prev = match nodes.get(i - 1) {
                            Some(i) => i,
                            None => {
                                return Err(CompilerError(i, CompilerErrorInner::ExpectedBlock));
                            }
                        };

                        if let ast::ASTNodeInner::Identifier(_) = prev.1 {
                        } else {
                            return Err(CompilerError(i, CompilerErrorInner::UnexpectedBlock));
                        }

                        match typ {
                            ast::BlockType::SingleEval | ast::BlockType::UniqueEval => {
                                continue;
                            }
                            _ => {
                                return Err(CompilerError(
                                    i,
                                    CompilerErrorInner::UnexpectedAnonymousBlock,
                                ));
                            }
                        }
                    }
                    _ => {
                        return Err(CompilerError(0, CompilerErrorInner::UnexpectedNode));
                    }
                }
            }
        }

        Ok(ir)
    }

    fn get_random_name(name: &'a str) -> &'static str {
        // most likely won't be a problem
        // only used for block names, which should live for the entirety of execution
        // will be a problem, if we try to reuse the same process for an another execution
        let rand = Alphanumeric
            .sample_string(&mut rand::thread_rng(), 12)
            .into_boxed_str();
        Box::leak(format!("{}_{}", name, rand).into_boxed_str())
    }

    fn compile_block(
        &mut self,
        name: &'a str,
        block: &'a ast::ASTNode,
        ir: &mut ir::IR<'a>,
    ) -> Result<ir::Block<'a>, CompilerError> {
        let mut code: Vec<ir::IRCode<'a>> = vec![];
        let t: ast::BlockType;
        let v: &Vec<ast::ASTNode>;
        match &block.1 {
            ast::ASTNodeInner::Block(ty, ve) => {
                t = *ty;
                v = ve;
            }
            _ => {
                return Err(CompilerError(0, CompilerErrorInner::ExpectedBlock));
            }
        };

        let t: ir::BlockRunType = match t {
            ast::BlockType::Program
            | ast::BlockType::EscapedUniqueEval
            | ast::BlockType::UniqueEval => ir::BlockRunType::Unique,
            _ => ir::BlockRunType::Once,
        };

        for (i, node) in v.iter().enumerate() {
            match &node.1 {
                ast::ASTNodeInner::Identifier(i) => {
                    code.push(ir::IRCode::Call(i.as_str()));
                }
                ast::ASTNodeInner::EscapedIdentifier(i) => {
                    code.push(ir::IRCode::PutValue(ir::Value::Block(i.as_str())));
                }
                ast::ASTNodeInner::Closer => {
                    continue;
                }
                ast::ASTNodeInner::Keyword(tok::KeywordType::Return) => {
                    code.push(ir::IRCode::Return);
                }
                ast::ASTNodeInner::Keyword(tok::KeywordType::True) => {
                    code.push(ir::IRCode::PutValue(ir::Value::Boolean(true)));
                }
                ast::ASTNodeInner::Keyword(tok::KeywordType::False) => {
                    code.push(ir::IRCode::PutValue(ir::Value::Boolean(false)));
                }
                ast::ASTNodeInner::Keyword(tok::KeywordType::Null) => {
                    code.push(ir::IRCode::PutValue(ir::Value::Null))
                }
                ast::ASTNodeInner::Keyword(tok::KeywordType::None) => {
                    panic!("KeywordType::None is not supposed to be in the final ast.")
                }
                ast::ASTNodeInner::String(str) => {
                    code.push(ir::IRCode::PutValue(ir::Value::String(str)))
                }
                ast::ASTNodeInner::Number(num) => {
                    code.push(ir::IRCode::PutValue(ir::Value::Number(*num)));
                }
                ast::ASTNodeInner::Block(typ, _) => {
                    let nested_name = Self::get_random_name(name);

                    let nested_ir = self.compile_block(nested_name, node, ir)?;
                    let prev = match v.get(i - 1) {
                        Some(i) => i,
                        None => {
                            return Err(CompilerError(i, CompilerErrorInner::ExpectedBlock));
                        }
                    };

                    if let ast::ASTNodeInner::Keyword(_) = prev.1 {
                    } else {
                        match typ {
                            ast::BlockType::SingleEval | ast::BlockType::UniqueEval => {
                                return Err(CompilerError(i, CompilerErrorInner::ExpectedBlock));
                            }
                            _ => {
                                match ir.add_block(nested_name, nested_ir) {
                                    Err(ir::IRError::WordAlreadyExists) => {
                                        return Err(CompilerError(
                                            i,
                                            CompilerErrorInner::WordAlreadyExists,
                                        ));
                                    }
                                    _ => {}
                                }
                                code.push(ir::IRCode::PutValue(ir::Value::Block(nested_name)));
                            }
                        }
                    }
                }
                ast::ASTNodeInner::Keyword(tok::KeywordType::If) => {
                    let next = match v.get(i + 1) {
                        Some(i) => i,
                        None => {
                            return Err(CompilerError(i, CompilerErrorInner::ExpectedBlock));
                        }
                    };

                    match &next.1 {
                        ast::ASTNodeInner::Block(ast::BlockType::UniqueEval, _) => {}
                        _ => {
                            return Err(CompilerError(
                                i,
                                CompilerErrorInner::ExpectedUniqueEvalBlockAfterIf,
                            ));
                        }
                    }

                    let nested_name = Self::get_random_name(name);
                    let nested_ir = self.compile_block(nested_name, next, ir)?;
                    match ir.add_block(nested_name, nested_ir) {
                        Err(ir::IRError::WordAlreadyExists) => {
                            return Err(CompilerError(i, CompilerErrorInner::WordAlreadyExists));
                        }
                        _ => {}
                    }
                    code.push(ir::IRCode::If(nested_name));
                }
                ast::ASTNodeInner::Keyword(tok::KeywordType::While) => {
                    let next = match v.get(i + 1) {
                        Some(i) => i,
                        None => {
                            return Err(CompilerError(i, CompilerErrorInner::ExpectedBlock));
                        }
                    };

                    match &next.1 {
                        ast::ASTNodeInner::Block(ast::BlockType::UniqueEval, _) => {}
                        _ => {
                            return Err(CompilerError(
                                i,
                                CompilerErrorInner::ExpectedUniqueEvalBlockAfterIf,
                            ));
                        }
                    }

                    let nested_name = Self::get_random_name(name);
                    let nested_ir = self.compile_block(nested_name, next, ir)?;
                    match ir.add_block(nested_name, nested_ir) {
                        Err(ir::IRError::WordAlreadyExists) => {
                            return Err(CompilerError(i, CompilerErrorInner::WordAlreadyExists));
                        }
                        _ => {}
                    }
                    code.push(ir::IRCode::While(nested_name));
                }
            }
        }

        Ok(ir::Block::IR(t, code))
    }
}
