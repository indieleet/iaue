type FnSymbol<'a> = libloading::Symbol<
    'a,
    libloading::Symbol<'a, unsafe extern "C" fn(f32, f32, f32, usize, &[f32]) -> Vec<(f32, f32)>>,
>;

#[derive(Debug, Clone, Copy, Default)]
pub enum WaveType {
    #[default]
    Sine = 0,
    Square = 1,
    Saw = 2,
    Triangle = 3,
}

impl std::ops::Add<u8> for WaveType {
    type Output = WaveType;
    fn add(self, rhs: u8) -> Self::Output {
        WaveType::from(self as u8 + rhs) 
    }
}

impl std::ops::AddAssign<u8> for WaveType {
    fn add_assign(&mut self, rhs: u8) {
        *self = WaveType::from(*self as u8 + rhs) 
    }
}
impl From<u8> for WaveType {
    fn from(value: u8) -> Self {
        match value {
            0 | 4 => WaveType::Sine,
            1 => WaveType::Square,
            2 => WaveType::Saw,
            _ => WaveType::Triangle,
        }
    }
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

impl core::default::Default for Operator {
    fn default() -> Self {
        Operator {
            wave: WaveType::Sine,
            level: 255,
            feedback: 0,
            num: 1,
            den: 1,
            detune: 0,
        }
    }
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
                write!(f, "SIN")
            }
            WaveType::Square => {
                write!(f, "SQR")
            }
            WaveType::Saw => {
                write!(f, "SAW")
            }
            WaveType::Triangle => {
                write!(f, "TRI")
            }
            _ => {
                write!(f, "OTH")
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
        op1: Operator,
        op2: Operator,
        op3: Operator,
        op4: Operator,
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
