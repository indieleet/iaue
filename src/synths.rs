type FnSymbol<'a> = libloading::Symbol<
    'a,
    libloading::Symbol<'a, unsafe extern "C" fn(f32, f32, f32, usize, &[f32]) -> Vec<(f32, f32)>>,
>;

#[derive(Debug, Clone, Copy, Default)]
pub enum WaveType {
    #[default]
    Sine,
    Square,
    Saw,
    Triangle,
}

#[derive(Debug, Clone)]
pub enum Synths<'a> {
    Rust {
        name: &'a str,
        num: u8,
        level: u8,
        fn_symbol: Option<FnSymbol<'a>>,
    },
    Fm {
        name: &'a str,
        level: u8,
        algo: u8,
        wave1: WaveType,
        wave2: WaveType,
        wave3: WaveType,
        wave4: WaveType,
        level1: u8,
        level2: u8,
        level3: u8,
        level4: u8,
        feedback1: u8,
        feedback2: u8,
        feedback3: u8,
        feedback4: u8
    },
    Macro {
        name: &'a str,
        level: u8,
        engine: u8,
        par1: u8,
        par2: u8,
        par3: u8
    },
}
