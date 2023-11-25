use crate::note::{AbsulateNotePitch, Note, NoteDuration, NoteName, Octave};

/// 一根琴弦上的发音
pub struct GuitarNotePitch {
    base: AbsulateNotePitch,
    fret: u8,
}

impl GuitarNotePitch {
    pub fn new(base: AbsulateNotePitch, fret: u8) -> Self {
        GuitarNotePitch { base, fret }
    }

    fn to_absulate(&self) -> AbsulateNotePitch {
        let note_type = (self.base.note_type as u8 + self.fret) % 12;
        let octave = self.base.octave as u8 + (self.base.note_type as u8 + self.fret) / 12;
        AbsulateNotePitch::new(note_type.into(), octave.into())
    }
}

pub struct GuitarNote {
    pitch: GuitarNotePitch,
    duration: NoteDuration,
}

impl GuitarNote {
    pub fn to_absulate(&self) -> Note {
        Note {
            pitch: self.pitch.to_absulate(),
            duration: self.duration,
        }
    }
}

pub struct Guitar {
    base: [AbsulateNotePitch; 6],
    capo_fret: u8,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum GuitarString {
    S1,
    S2,
    S3,
    S4,
    S5,
    S6,
}

impl From<u8> for GuitarString {
    fn from(s: u8) -> Self {
        match s {
            1 => GuitarString::S1,
            2 => GuitarString::S2,
            3 => GuitarString::S3,
            4 => GuitarString::S4,
            5 => GuitarString::S5,
            6 => GuitarString::S6,
            _ => panic!("invalid guitar string"),
        }
    }
}

impl Default for Guitar {
    fn default() -> Self {
        use NoteName::*;
        use Octave::*;
        Self {
            base: [
                AbsulateNotePitch::new(E, O4),
                AbsulateNotePitch::new(B, O3),
                AbsulateNotePitch::new(G, O3),
                AbsulateNotePitch::new(D, O3),
                AbsulateNotePitch::new(A, O2),
                AbsulateNotePitch::new(E, O2),
            ],
            capo_fret: 0,
        }
    }
}

impl Guitar {
    pub fn set_capo_fret(&mut self, fret: u8) {
        self.capo_fret = fret;
    }

    pub fn get_capo_fret(&self) -> u8 {
        self.capo_fret
    }

    pub fn to_absulate_note(&self, s: GuitarString, fret: u8, duration: NoteDuration) -> Note {
        let mut note = GuitarNote {
            pitch: GuitarNotePitch::new(self.base[s as usize], fret),
            duration,
        }
        .to_absulate();
        note.pitch = note.pitch.add(self.capo_fret as i32);
        note
    }
}
