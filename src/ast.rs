use crate::tok;

#[derive(Debug, Clone)]
pub enum ASTNodeInner {
    Block(BlockType, Vec<ASTNode>),
    Identifier(String),
    EscapedIdentifier(String),
    Keyword(tok::KeywordType),
    Number(f64),
    String(String),
    Closer,
}

#[derive(Debug, Clone, Copy)]
pub enum BlockType {
    Program,
    EscapedSingleEval,
    SingleEval,
    EscapedUniqueEval,
    UniqueEval,
}

#[derive(Debug, Clone)]
pub struct ASTNode(pub usize, pub ASTNodeInner);

pub struct ASTBuilder {
    at: usize,
}

#[derive(Debug)]
pub enum ASTErrorInner {
    UnexpectedEnd,
    UnknownToken,
    UnexpectedToken,
}

#[derive(Debug)]
pub struct ASTError(pub usize, pub ASTErrorInner);

impl ASTBuilder {
    pub fn new() -> Self {
        ASTBuilder { at: 0 }
    }

    pub fn clean(&mut self) {
        self.at = 0;
    }

    pub fn parse(&mut self, tokens: &Vec<tok::Token>) -> Result<ASTNode, ASTError> {
        self.clean();
        let mut nodes: Vec<ASTNode> = vec![];

        while self.at < tokens.len() {
            nodes.push(self.walk(tokens)?);
            self.at += 1;
        }

        Ok(ASTNode(0, ASTNodeInner::Block(BlockType::Program, nodes)))
    }

    fn walk(&mut self, tokens: &Vec<tok::Token>) -> Result<ASTNode, ASTError> {
        let current = match tokens.get(self.at) {
            Some(i) => i,
            None => {
                return Err(ASTError(self.at, ASTErrorInner::UnexpectedEnd));
            }
        };

        let r = match &current.1 {
            tok::TokenInner::Identifier(str) => {
                Ok(ASTNode(self.at, ASTNodeInner::Identifier(str.to_string())))
            }
            tok::TokenInner::EscapedIdentifier(str) => Ok(ASTNode(
                self.at,
                ASTNodeInner::EscapedIdentifier(str.to_string()),
            )),
            tok::TokenInner::String(str) => {
                Ok(ASTNode(self.at, ASTNodeInner::String(str.to_string())))
            }
            tok::TokenInner::Number(num) => Ok(ASTNode(self.at, ASTNodeInner::Number(*num))),
            tok::TokenInner::Keyword(kw) => Ok(ASTNode(self.at, ASTNodeInner::Keyword(*kw))),
            block => {
                let typ = match block {
                    tok::TokenInner::OnceOpen => BlockType::SingleEval,
                    tok::TokenInner::UniqueOpen => BlockType::UniqueEval,
                    tok::TokenInner::OnceEscapedOpen => BlockType::EscapedSingleEval,
                    tok::TokenInner::UniqueEscapedOpen => BlockType::EscapedUniqueEval,
                    _ => {
                        self.at += 1;
                        return Ok(ASTNode(self.at, ASTNodeInner::Closer));
                    }
                };
                let mut nodes: Vec<ASTNode> = vec![];
                let begin = self.at;

                self.at += 1;

                loop {
                    let current = match tokens.get(self.at) {
                        Some(i) => i,
                        None => {
                            return Err(ASTError(self.at, ASTErrorInner::UnexpectedEnd));
                        }
                    };

                    let is_closer = match tok::is_closer(&block, &current.1) {
                        Some(i) => i,
                        None => false,
                    };

                    if is_closer {
                        break;
                    } else {
                        let tok = self.walk(tokens)?;
                        if let ASTNodeInner::Closer = tok.1 {
                            break;
                        } else {
                            nodes.push(tok);
                        }
                    }

                    self.at += 1;
                }

                Ok(ASTNode(begin, ASTNodeInner::Block(typ, nodes)))
            }
        };

        r
    }
}
