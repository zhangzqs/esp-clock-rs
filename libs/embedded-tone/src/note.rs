#[derive(Debug, Clone, Copy)]
pub enum SolfegeName {
    Do,
    DoSharp,
    Re,
    ReSharp,
    Mi,
    Fa,
    FaSharp,
    Sol,
    SolSharp,
    La,
    LaSharp,
    Si,
}

impl From<u8> for SolfegeName {
    fn from(x: u8) -> Self {
        match x {
            0 => SolfegeName::Do,
            1 => SolfegeName::DoSharp,
            2 => SolfegeName::Re,
            3 => SolfegeName::ReSharp,
            4 => SolfegeName::Mi,
            5 => SolfegeName::Fa,
            6 => SolfegeName::FaSharp,
            7 => SolfegeName::Sol,
            8 => SolfegeName::SolSharp,
            9 => SolfegeName::La,
            10 => SolfegeName::LaSharp,
            11 => SolfegeName::Si,
            _ => panic!("invalid note name"),
        }
    }
}

impl From<SolfegeName> for u8 {
    fn from(val: SolfegeName) -> Self {
        val as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoteName {
    C,
    Cs,
    D,
    Eb,
    E,
    F,
    Fs,
    G,
    Gs,
    A,
    Bb,
    B,
}

impl From<u8> for NoteName {
    fn from(x: u8) -> Self {
        match x {
            0 => NoteName::C,
            1 => NoteName::Cs,
            2 => NoteName::D,
            3 => NoteName::Eb,
            4 => NoteName::E,
            5 => NoteName::F,
            6 => NoteName::Fs,
            7 => NoteName::G,
            8 => NoteName::Gs,
            9 => NoteName::A,
            10 => NoteName::Bb,
            11 => NoteName::B,
            _ => panic!("invalid note type"),
        }
    }
}

impl From<NoteName> for u8 {
    fn from(val: NoteName) -> Self {
        val as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Octave {
    O0,
    O1,
    O2,
    O3,
    O4,
    O5,
    O6,
    O7,
    O8,
    O9,
    O10,
}

impl From<u8> for Octave {
    fn from(x: u8) -> Self {
        match x {
            0 => Octave::O0,
            1 => Octave::O1,
            2 => Octave::O2,
            3 => Octave::O3,
            4 => Octave::O4,
            5 => Octave::O5,
            6 => Octave::O6,
            7 => Octave::O7,
            8 => Octave::O8,
            9 => Octave::O9,
            10 => Octave::O10,
            _ => panic!("invalid octave"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NoteDuration {
    /// 附点全音符
    WholeDotted,
    /// 全音符
    Whole,
    /// 附点二分音符
    HalfDotted,
    /// 二分音符
    Half,
    /// 附点四分音符
    QuarterDotted,
    /// 四分音符
    Quarter,
    /// 附点八分音符
    EighthDotted,
    /// 八分音符
    Eighth,
    /// 附点十六分音符
    SixteenthDotted,
    /// 十六分音符
    Sixteenth,
    /// 附点三十二分音符
    ThirtySecondDotted,
    /// 三十二分音符
    ThirtySecond,
    /// 附点六十四分音符
    SixtyFourthDotted,
    /// 六十四分音符
    SixtyFourth,
    Other(f32),
}

impl From<NoteDuration> for f32 {
    fn from(val: NoteDuration) -> Self {
        match val {
            NoteDuration::WholeDotted => 3.0 / 2.0,
            NoteDuration::Whole => 1.0,
            NoteDuration::HalfDotted => 3.0 / 4.0,
            NoteDuration::Half => 1.0 / 2.0,
            NoteDuration::QuarterDotted => 3.0 / 8.0,
            NoteDuration::Quarter => 1.0 / 4.0,
            NoteDuration::EighthDotted => 3.0 / 16.0,
            NoteDuration::Eighth => 1.0 / 8.0,
            NoteDuration::SixteenthDotted => 3.0 / 32.0,
            NoteDuration::Sixteenth => 1.0 / 16.0,
            NoteDuration::ThirtySecondDotted => 3.0 / 64.0,
            NoteDuration::ThirtySecond => 1.0 / 32.0,
            NoteDuration::SixtyFourthDotted => 3.0 / 128.0,
            NoteDuration::SixtyFourth => 1.0 / 64.0,
            NoteDuration::Other(x) => x,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AbsulateNotePitch {
    pub note_type: NoteName,
    pub octave: Octave,
}

impl PartialOrd for AbsulateNotePitch {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.to_midi_note_key().partial_cmp(&other.to_midi_note_key())
    }
}

impl AbsulateNotePitch {
    pub fn new(note_type: NoteName, octave: Octave) -> Self {
        AbsulateNotePitch { note_type, octave }
    }

    pub fn add(self, half_tone: i32) -> AbsulateNotePitch {
        // 计算基准音(绝对音高)到C1的音程(单位为一个半音)
        let old_dist = self.octave as i32 * 12 + self.note_type as i32;
        let new_dist = old_dist + half_tone;
        let octave = new_dist / 12;
        let note_type = new_dist % 12;
        AbsulateNotePitch::new((note_type as u8).into(), (octave as u8).into())
    }

    pub fn frequency(&self) -> u32 {
        // 以A4为基准音，频率为440Hz
        let base = AbsulateNotePitch::new(NoteName::A, Octave::O4);
        const BASE_FREQ: f32 = 440f32;

        // 计算self绝对音程
        let d = self.octave as i32 * 12 + self.note_type as i32;
        // 计算A4的绝对音程
        let base_d = base.octave as i32 * 12 + base.note_type as i32;
        // 计算相对音程
        let half_tone = d - base_d;

        // 计算频率
        (BASE_FREQ * 2.0f32.powf(half_tone as f32 / 12.0)) as u32
    }

    pub fn to_midi_note_key(&self) -> u8 {
        let o = self.octave as u8;
        let n = self.note_type as u8;
        o * 12 + n
    }

    pub fn from_midi_note_key(key: u8) -> Self {
        let o = key / 12;
        let n = key % 12;
        Self {
            note_type: NoteName::from(n),
            octave: Octave::from(o),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RelativePitch {
    pub solfege_name: SolfegeName,
    pub octave: Octave,
}

impl RelativePitch {
    pub fn new(solfege_name: SolfegeName, octave: Octave) -> Self {
        RelativePitch {
            solfege_name,
            octave,
        }
    }

    /// 给定一个基准音，计算出对应的绝对音高
    pub fn to_absulate(self, base: AbsulateNotePitch) -> AbsulateNotePitch {
        // 计算相对音高到基准音的音程(单位为一个半音)
        let d = self.octave as i32 * 12 + self.solfege_name as i32;
        base.add(d)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Note {
    pub pitch: AbsulateNotePitch,
    pub duration: NoteDuration,
}

#[derive(Debug, Clone, Copy)]
pub struct Rest {
    pub duration: NoteDuration,
}

impl Rest {
    pub fn new(duration: NoteDuration) -> Self {
        Rest { duration }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SlideNote {
    pub start_pitch: AbsulateNotePitch,
    pub end_pitch: AbsulateNotePitch,
    pub duration: NoteDuration,
}
