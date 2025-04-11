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

#[derive(Debug, Clone, Copy, Default)]
pub struct Lfo {
    pub wave: WaveType,
    pub dest: FMModDest,
    pub speed: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Operator {
    pub wave: WaveType,
    pub level: u8,
    pub feedback: u8,
    pub num: u8,
    pub den: u8,
    pub detune: i8,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum FMModDest {
    #[default]
    Level = 0,
    Pitch = 1,
    Level1 = 2,
    Level2 = 3,
    Level3 = 4,
    Level4 = 5,
    Pitch1 = 6,
    Pitch2 = 7,
    Pitch3 = 8,
    Pitch4 = 9,
    Feedback1 = 10,
    Feedback2 = 11,
    Feedback3 = 12,
    Feedback4 = 13,
}

impl core::fmt::Display for WaveType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            WaveType::Sine => {
                write!(f, "Sine")
            }
            _ => {
                write!(f, "Other")
            }
        }
    }
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
        feedback4: u8,
        num1: u8,
        num2: u8,
        num3: u8,
        num4: u8,
        den1: u8,
        den2: u8,
        den3: u8,
        den4: u8,
        detune1: i8,
        detune2: i8,
        detune3: i8,
        detune4: i8,
        lfo1: Lfo,
    },
    Macro {
        name: &'a str,
        level: u8,
        engine: u8,
        par1: u8,
        par2: u8,
        par3: u8,
    },
}
