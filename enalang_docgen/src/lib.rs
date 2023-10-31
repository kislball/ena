pub mod renderers;

use enalang_ir::IR;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize)]
pub struct DocEntry {
    pub name: String,
    pub comment: String,
}

#[derive(Default, Clone, Serialize)]
pub struct Documentation(Vec<DocEntry>);

impl Documentation {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn from_ir(ir: IR) -> Self {
        let mut s = Self::new();
        s.generate(ir);
        s
    }

    pub fn generate(&mut self, ir: IR) {
        for (block, data) in ir.annotations {
            let data = data
                .lines()
                .filter(|x| x.chars().next().unwrap_or(' ') != '@')
                .collect::<Vec<&str>>()
                .join("\n");
            if !data.is_empty() {
                self.0.push(DocEntry {
                    name: block.to_string(),
                    comment: data,
                });
            }
        }
    }
}

impl From<IR> for Documentation {
    fn from(value: IR) -> Self {
        Self::from_ir(value)
    }
}

pub trait DocRenderer {
    fn render(&self, doc: Documentation) -> String;
}

#[derive(Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub enum DocRendererType {
    Html,
    JSON,
    Markdown,
}

impl DocRendererType {
    pub fn resolve_renderer(self) -> impl DocRenderer {
        renderers::NullRenderer
    }
}
