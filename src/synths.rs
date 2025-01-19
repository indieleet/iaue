type FnSymbol<'a> = libloading::Symbol<'a, libloading::Symbol<'a, unsafe extern "C" fn(f32, f32, f32, usize, &[f32]) -> Vec<(f32, f32)>>>;
#[derive(Debug, Clone)]
pub enum Synths<'a> {
    Rust {name: &'a str, num: u8, level: u8, fn_symbol: Option<FnSymbol<'a>>},
    Macro {name: &'a str, level: u8}
}
