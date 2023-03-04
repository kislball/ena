use crate::DocRenderer;
use std::collections::HashMap;

pub struct JsonRenderer;

impl DocRenderer for JsonRenderer {
    fn render(&self, doc: crate::Documentation) -> String {
        let mut map = HashMap::<String, String>::new();

        for entry in doc.0 {
            map.insert(entry.name, entry.comment);
        }

        serde_json::to_string(&map).unwrap()
    }
}
