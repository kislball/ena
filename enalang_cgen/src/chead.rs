static CHEAD: &'static [u8] = include_bytes!("./head.c");

pub fn get_chead() -> String {
    String::from_utf8_lossy(CHEAD).to_string()
}