#[derive(Debug, Copy, Clone)]
pub enum Synths<'a> {
    Rust {name: &'a str, num: u8, level: u8},
    Macro {name: &'a str, level: u8}
}
