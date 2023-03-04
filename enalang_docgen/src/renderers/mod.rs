use crate::{DocRenderer, Documentation};

pub mod html;
pub mod json;
pub mod md;

pub struct NullRenderer;

impl DocRenderer for NullRenderer {
    fn render(&self, _: Documentation) -> String {
        String::new()
    }
}

pub fn resolve_renderer(name: &str) -> Option<Box<dyn DocRenderer>> {
    match name {
        "json" => Some(Box::new(json::JsonRenderer)),
        _ => None,
    }
}
