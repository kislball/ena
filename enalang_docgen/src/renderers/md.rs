use crate::{DocEntry, DocRenderer};
use serde::Serialize;

pub struct MarkdownRenderer;

pub const TEMPLATE: &'static str = r#"
{{ for entry in doc }}
# {entry.name}
{entry.comment | unescaped}
{{ endfor }}
"#;

#[derive(Serialize)]
pub struct MarkdownContext {
    pub doc: Vec<DocEntry>,
}

impl DocRenderer for MarkdownRenderer {
    fn render(&self, doc: crate::Documentation) -> String {
        let mut tt = tinytemplate::TinyTemplate::new();
        tt.add_template("markdown", TEMPLATE).unwrap();

        let ctx = MarkdownContext { doc: doc.0 };
        tt.render("markdown", &ctx).unwrap().trim().to_string()
    }
}
