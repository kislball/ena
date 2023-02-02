use crate::vm::ir;
use crate::{ast, tok};
use flexstr::{ToLocalStr, LocalStr, IntoLocalStr, local_fmt};
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
    BlockAlreadyExists,
}

#[derive(Debug)]
pub struct CompilerError(pub ast::ASTNode, pub CompilerErrorInner);

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
            _ => {
                return Err(CompilerError(
                    ast::ASTNode(0, ast::ASTNodeInner::Closer),
                    CompilerErrorInner::ExpectedProgramAsRoot,
                ))
            }
        };

        {
            for (i, node) in nodes.iter().enumerate() {
                match &node.1 {
                    ast::ASTNodeInner::Comment(data) => {
                        if let Some(ast::ASTNode(_, ast::ASTNodeInner::Identifier(id))) =
                            nodes.get(i + 1)
                        {
                            ir.annotations
                                .insert(id.to_local_str(), data.to_local_str());
                        }
                    }
                    ast::ASTNodeInner::Identifier(id) => {
                        let next = match nodes.get(i + 1) {
                            Some(i) => i,
                            None => {
                                return Err(CompilerError(
                                    node.clone(),
                                    CompilerErrorInner::ExpectedBlock,
                                ));
                            }
                        };

                        if let ast::ASTNodeInner::Block(_, _) = next.1 {
                            match self.compile_block(id.as_str(), next, &mut ir) {
                                Err(e) => {
                                    return Err(e);
                                }
                                Ok(v) => {
                                    if let Err(ir::IRError::BlockAlreadyExists) =
                                        ir.add_block(id.to_local_str(), v)
                                    {
                                        return Err(CompilerError(
                                            node.clone(),
                                            CompilerErrorInner::BlockAlreadyExists,
                                        ));
                                    }
                                }
                            };
                        } else {
                            return Err(CompilerError(
                                node.clone(),
                                CompilerErrorInner::ExpectedBlock,
                            ));
                        }
                    }
                    ast::ASTNodeInner::Block(typ, _) => {
                        let prev = match nodes.get(i - 1) {
                            Some(i) => i,
                            None => {
                                return Err(CompilerError(
                                    node.clone(),
                                    CompilerErrorInner::ExpectedBlock,
                                ));
                            }
                        };

                        if let ast::ASTNodeInner::Identifier(_) = prev.1 {
                        } else {
                            return Err(CompilerError(
                                node.clone(),
                                CompilerErrorInner::UnexpectedBlock,
                            ));
                        }

                        match typ {
                            ast::BlockType::SingleEval | ast::BlockType::UniqueEval => {
                                continue;
                            }
                            _ => {
                                return Err(CompilerError(
                                    node.clone(),
                                    CompilerErrorInner::UnexpectedAnonymousBlock,
                                ));
                            }
                        }
                    }
                    _ => {
                        return Err(CompilerError(
                            node.clone(),
                            CompilerErrorInner::UnexpectedNode,
                        ));
                    }
                }
            }
        }

        Ok(ir)
    }

    fn get_random_name(name: &LocalStr) -> LocalStr {
        // most likely won't be a problem
        // only used for block names, which should live for the entirety of execution
        // will be a problem, if we try to reuse the same process for an another execution
        let rand = Alphanumeric
            .sample_string(&mut rand::thread_rng(), 12)
            .into_local_str();
        local_fmt!("{name}_{rand}")
    }

    fn compile_block(
        &mut self,
        name: &'a str,
        block: &'a ast::ASTNode,
        ir: &mut ir::IR,
    ) -> Result<ir::Block, CompilerError> {
        let name = name.to_local_str();
        let mut code: Vec<ir::IRCode> = vec![];
        let t: ast::BlockType;
        let v: &Vec<ast::ASTNode>;
        match &block.1 {
            ast::ASTNodeInner::Block(ty, ve) => {
                t = *ty;
                v = ve;
            }
            _ => {
                return Err(CompilerError(
                    block.clone(),
                    CompilerErrorInner::ExpectedBlock,
                ));
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
                ast::ASTNodeInner::Comment(_) => {}
                ast::ASTNodeInner::Atom(i) => {
                    code.push(ir::IRCode::PutValue(ir::Value::Atom(i.to_local_str())));
                }
                ast::ASTNodeInner::Identifier(id) => {
                    let next = v.get(i + 1);
                    match next {
                        Some(ast::ASTNode(_, ast::ASTNodeInner::Block(_, _))) => {
                            let compiled = self.compile_block(id, next.unwrap(), ir)?;
                            if let ir::Block::IR(typ, data) = compiled {
                                code.push(ir::IRCode::LocalBlock(id.to_local_str(), typ, data));
                            }
                        }
                        _ => {
                            code.push(ir::IRCode::Call(id.to_local_str()));
                        }
                    };
                }
                ast::ASTNodeInner::EscapedIdentifier(i) => {
                    code.push(ir::IRCode::PutValue(ir::Value::Block(Into::into(i))));
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
                    code.push(ir::IRCode::PutValue(ir::Value::String(Into::into(str))))
                }
                ast::ASTNodeInner::Number(num) => {
                    code.push(ir::IRCode::PutValue(ir::Value::Number(*num)));
                }
                ast::ASTNodeInner::Block(typ, _) => {
                    let nested_name = Self::get_random_name(&name);

                    let nested_ir = self.compile_block(nested_name.as_str(), node, ir)?;
                    let prev = match v.get(i - 1) {
                        Some(i) => i,
                        None => {
                            return Err(CompilerError(
                                node.clone(),
                                CompilerErrorInner::ExpectedBlock,
                            ));
                        }
                    };

                    if let ast::ASTNodeInner::Keyword(_) = prev.1 {
                    } else {
                        match typ {
                            ast::BlockType::SingleEval | ast::BlockType::UniqueEval => {
                                if !matches!(
                                    v.get(i - 1),
                                    Some(ast::ASTNode(_, ast::ASTNodeInner::Identifier(_)))
                                ) {
                                    return Err(CompilerError(
                                        node.clone(),
                                        CompilerErrorInner::UnexpectedBlock,
                                    ));
                                }
                            }
                            _ => {
                                if let Err(ir::IRError::BlockAlreadyExists) =
                                    ir.add_block(nested_name.to_local_str(), nested_ir)
                                {
                                    return Err(CompilerError(
                                        node.clone(),
                                        CompilerErrorInner::BlockAlreadyExists,
                                    ));
                                }
                                code.push(ir::IRCode::PutValue(ir::Value::Block(Into::into(
                                    nested_name,
                                ))));
                            }
                        }
                    }
                }
                ast::ASTNodeInner::Keyword(tok::KeywordType::If) => {
                    let next = match v.get(i + 1) {
                        Some(i) => i,
                        None => {
                            return Err(CompilerError(
                                node.clone(),
                                CompilerErrorInner::ExpectedBlock,
                            ));
                        }
                    };

                    match &next.1 {
                        ast::ASTNodeInner::Block(ast::BlockType::UniqueEval, _) => {}
                        _ => {
                            return Err(CompilerError(
                                node.clone(),
                                CompilerErrorInner::ExpectedUniqueEvalBlockAfterIf,
                            ));
                        }
                    }

                    let nested_name = Self::get_random_name(&name);
                    let nested_ir = self.compile_block(&nested_name, next, ir)?;
                    if let Err(ir::IRError::BlockAlreadyExists) =
                        ir.add_block(nested_name.to_local_str(), nested_ir)
                    {
                        return Err(CompilerError(
                            node.clone(),
                            CompilerErrorInner::BlockAlreadyExists,
                        ));
                    }
                    code.push(ir::IRCode::If(nested_name.to_local_str()));
                }
                ast::ASTNodeInner::Keyword(tok::KeywordType::While) => {
                    let next = match v.get(i + 1) {
                        Some(i) => i,
                        None => {
                            return Err(CompilerError(
                                node.clone(),
                                CompilerErrorInner::ExpectedBlock,
                            ));
                        }
                    };

                    match &next.1 {
                        ast::ASTNodeInner::Block(ast::BlockType::UniqueEval, _) => {}
                        _ => {
                            return Err(CompilerError(
                                node.clone(),
                                CompilerErrorInner::ExpectedUniqueEvalBlockAfterIf,
                            ));
                        }
                    }

                    let nested_name = Self::get_random_name(&name);
                    let nested_ir = self.compile_block(&nested_name, next, ir)?;
                    if let Err(ir::IRError::BlockAlreadyExists) =
                        ir.add_block(nested_name.to_local_str(), nested_ir)
                    {
                        return Err(CompilerError(
                            node.clone(),
                            CompilerErrorInner::BlockAlreadyExists,
                        ));
                    }
                    code.push(ir::IRCode::While(nested_name.to_local_str()));
                }
            }
        }

        Ok(ir::Block::IR(t, code))
    }
}
