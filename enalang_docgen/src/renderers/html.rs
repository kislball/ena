use crate::{DocEntry, DocRenderer};
use serde::Serialize;

pub struct HtmlRenderer;

pub const TEMPLATE: &str = r#"
{{ for entry in doc }}
<h1 class="enadocgen-title">{entry.name}</h1>
<div class="enadocgen-comment">{entry.comment | unescaped}</div>
{{ endfor }}
"#;

#[derive(Serialize)]
pub struct HtmlContext {
    pub doc: Vec<DocEntry>,
}

impl DocRenderer for HtmlRenderer {
    fn render(&self, doc: crate::Documentation) -> String {
        let mut tt = tinytemplate::TinyTemplate::new();
        tt.add_template("html", TEMPLATE).unwrap();

        let doc = doc
            .0
            .iter()
            .map(|x| DocEntry {
                name: x.name.clone(),
                comment: markdown::to_html(&x.comment.replace('\n', "\n\n")),
            })
            .collect::<Vec<DocEntry>>();
        let ctx = HtmlContext { doc };
        tt.render("html", &ctx).unwrap().trim().to_string()
    }
}
