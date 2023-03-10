use crate::{ast, tok};
use enalang_ir as ir;
use flexstr::{shared_fmt, IntoSharedStr, SharedStr, ToSharedStr};
use rand::distributions::{Alphanumeric, DistString};

pub struct IRGen {}

#[derive(Debug, thiserror::Error)]
pub enum IRGenErrorInner {
    #[error("expected program node as root")]
    ExpectedProgramAsRoot,
    #[error("unexpected block")]
    UnexpectedBlock,
    #[error("unexpected node")]
    UnexpectedNode,
    #[error("expected block")]
    ExpectedBlock,
    #[error("unexpected anonymous block")]
    UnexpectedAnonymousBlock,
    #[error("expected unescaped block")]
    ExpectedUnescapedBlock,
    #[error("expected unique eval block after if/while")]
    ExpectedUniqueEvalBlockAfterIf,
    #[error("block already exists - {0}")]
    BlockAlreadyExists(SharedStr),
    #[error("cannot put local block on stack")]
    CannotPutLocalBlockOnStack,
}

#[derive(Debug, thiserror::Error)]
#[error("in node `{0:?}` - `{1}`")]
pub struct IRGenError(pub ast::ASTNode, pub IRGenErrorInner);

impl Default for IRGen {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> IRGen {
    pub fn new() -> Self {
        IRGen {}
    }

    pub fn compile(&'a mut self, ast: &'a ast::ASTNode) -> Result<ir::IR, IRGenError> {
        let mut ir = ir::IR::new();
        let nodes = match &ast.1 {
            ast::ASTNodeInner::Block(ast::BlockType::Program, nodes) => nodes,
            _ => {
                return Err(IRGenError(
                    ast::ASTNode(0, ast::ASTNodeInner::Closer),
                    IRGenErrorInner::ExpectedProgramAsRoot,
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
                            let filtered = data
                                .split('\n')
                                .filter(|x| x.starts_with('!'))
                                .map(|x| x.chars().skip(1).collect::<String>())
                                .collect::<Vec<String>>()
                                .join("\n");
                            ir.annotations
                                .insert(id.to_shared_str(), filtered.to_shared_str());
                        }
                    }
                    ast::ASTNodeInner::Identifier(id) => {
                        let next = match nodes.get(i + 1) {
                            Some(i) => i,
                            None => {
                                return Err(IRGenError(
                                    node.clone(),
                                    IRGenErrorInner::ExpectedBlock,
                                ));
                            }
                        };

                        if let ast::ASTNodeInner::Block(_, _) = next.1 {
                            match self.compile_block(id.as_str(), next, &mut ir, true) {
                                Err(e) => {
                                    return Err(e);
                                }
                                Ok(v) => {
                                    if let Err(ir::IRError::BlockAlreadyExists(_)) =
                                        ir.add_block(id.to_shared_str(), v, true)
                                    {
                                        return Err(IRGenError(
                                            node.clone(),
                                            IRGenErrorInner::BlockAlreadyExists(id.to_shared_str()),
                                        ));
                                    }
                                }
                            };
                        } else {
                            return Err(IRGenError(node.clone(), IRGenErrorInner::ExpectedBlock));
                        }
                    }
                    ast::ASTNodeInner::Block(typ, _) => {
                        let prev = match nodes.get(i - 1) {
                            Some(i) => i,
                            None => {
                                return Err(IRGenError(
                                    node.clone(),
                                    IRGenErrorInner::ExpectedBlock,
                                ));
                            }
                        };

                        if let ast::ASTNodeInner::Identifier(_) = prev.1 {
                        } else {
                            return Err(IRGenError(node.clone(), IRGenErrorInner::UnexpectedBlock));
                        }

                        match typ {
                            ast::BlockType::SingleEval | ast::BlockType::UniqueEval => {
                                continue;
                            }
                            _ => {
                                return Err(IRGenError(
                                    node.clone(),
                                    IRGenErrorInner::UnexpectedAnonymousBlock,
                                ));
                            }
                        }
                    }
                    _ => {
                        return Err(IRGenError(node.clone(), IRGenErrorInner::UnexpectedNode));
                    }
                }
            }
        }

        Ok(ir)
    }

    fn get_random_name(name: &SharedStr) -> SharedStr {
        let rand = Alphanumeric
            .sample_string(&mut rand::thread_rng(), 12)
            .into_shared_str();
        shared_fmt!("{name}_{rand}")
    }

    fn compile_block(
        &mut self,
        name: &'a str,
        block: &'a ast::ASTNode,
        ir: &mut ir::IR,
        is_global: bool,
    ) -> Result<ir::Block, IRGenError> {
        let name = name.to_shared_str();
        let mut code: Vec<ir::IRCode> = vec![];
        let t: ast::BlockType;
        let v: &Vec<ast::ASTNode>;
        match &block.1 {
            ast::ASTNodeInner::Block(ty, ve) => {
                t = *ty;
                v = ve;
            }
            _ => {
                return Err(IRGenError(block.clone(), IRGenErrorInner::ExpectedBlock));
            }
        };

        let t: ir::BlockRunType = match t {
            ast::BlockType::Program | ast::BlockType::UniqueEval => ir::BlockRunType::Unique,
            _ => ir::BlockRunType::Once,
        };

        for (i, node) in v.iter().enumerate() {
            match &node.1 {
                ast::ASTNodeInner::Comment(_) => {}
                ast::ASTNodeInner::Atom(i) => {
                    code.push(ir::IRCode::PutValue(ir::Value::Atom(i.to_shared_str())));
                }
                ast::ASTNodeInner::Identifier(id) => {
                    let next = v.get(i + 1);
                    match next {
                        Some(ast::ASTNode(_, ast::ASTNodeInner::Block(_, _))) => {
                            let compiled = self.compile_block(id, next.unwrap(), ir, false)?;
                            code.push(ir::IRCode::LocalBlock(
                                id.to_shared_str(),
                                compiled.run_type,
                                compiled.code,
                            ));
                        }
                        _ => {
                            code.push(ir::IRCode::Call(id.to_shared_str()));
                        }
                    };
                }
                ast::ASTNodeInner::EscapedIdentifier(i) => {
                    let i = i.to_shared_str();

                    code.push(ir::IRCode::PutValue(ir::Value::Block(i)));
                }
                ast::ASTNodeInner::Closer => {
                    continue;
                }
                ast::ASTNodeInner::Keyword(tok::KeywordType::Return) => {
                    code.push(ir::IRCode::Return);
                }
                ast::ASTNodeInner::Keyword(tok::KeywordType::ReturnLocal) => {
                    code.push(ir::IRCode::ReturnLocal);
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

                    let nested_ir = self.compile_block(nested_name.as_str(), node, ir, false)?;
                    let prev = match v.get(i - 1) {
                        Some(i) => i,
                        None => {
                            return Err(IRGenError(node.clone(), IRGenErrorInner::ExpectedBlock));
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
                                    return Err(IRGenError(
                                        node.clone(),
                                        IRGenErrorInner::UnexpectedBlock,
                                    ));
                                }
                            }
                            _ => {
                                if let Err(ir::IRError::BlockAlreadyExists(_)) =
                                    ir.add_block(nested_name.to_shared_str(), nested_ir, true)
                                {
                                    return Err(IRGenError(
                                        node.clone(),
                                        IRGenErrorInner::BlockAlreadyExists(
                                            nested_name.to_shared_str(),
                                        ),
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
                            return Err(IRGenError(node.clone(), IRGenErrorInner::ExpectedBlock));
                        }
                    };

                    match &next.1 {
                        ast::ASTNodeInner::Block(ast::BlockType::UniqueEval, _) => {}
                        _ => {
                            return Err(IRGenError(
                                node.clone(),
                                IRGenErrorInner::ExpectedUniqueEvalBlockAfterIf,
                            ));
                        }
                    }

                    let nested_name = Self::get_random_name(&name);
                    let nested_ir = self.compile_block(&nested_name, next, ir, false)?;
                    if let Err(ir::IRError::BlockAlreadyExists(_)) =
                        ir.add_block(nested_name.to_shared_str(), nested_ir, true)
                    {
                        return Err(IRGenError(
                            node.clone(),
                            IRGenErrorInner::BlockAlreadyExists(nested_name.to_shared_str()),
                        ));
                    }
                    code.push(ir::IRCode::If(nested_name.to_shared_str()));
                }
                ast::ASTNodeInner::Keyword(tok::KeywordType::While) => {
                    let next = match v.get(i + 1) {
                        Some(i) => i,
                        None => {
                            return Err(IRGenError(node.clone(), IRGenErrorInner::ExpectedBlock));
                        }
                    };

                    match &next.1 {
                        ast::ASTNodeInner::Block(ast::BlockType::UniqueEval, _) => {}
                        _ => {
                            return Err(IRGenError(
                                node.clone(),
                                IRGenErrorInner::ExpectedUniqueEvalBlockAfterIf,
                            ));
                        }
                    }

                    let nested_name = Self::get_random_name(&name);
                    let nested_ir = self.compile_block(&nested_name, next, ir, false)?;
                    if let Err(ir::IRError::BlockAlreadyExists(_)) =
                        ir.add_block(nested_name.to_shared_str(), nested_ir, true)
                    {
                        return Err(IRGenError(
                            node.clone(),
                            IRGenErrorInner::BlockAlreadyExists(nested_name.to_shared_str()),
                        ));
                    }
                    code.push(ir::IRCode::While(nested_name.to_shared_str()));
                }
            }
        }

        Ok(ir::Block {
            global: is_global,
            code: code,
            run_type: t,
        })
    }
}
