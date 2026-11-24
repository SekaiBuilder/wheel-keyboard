//! Layouts: which key sits in each direction of each ring.
//!
//! Directions run clockwise from the top: 0 N, 1 NE, 2 E, 3 SE, 4 S, 5 SW, 6 W, 7 NW.

use crate::key::{Key, Modifier};

/// Eight keys, one per direction, clockwise from the top.
pub type Ring = [Key; 8];

/// The inner and outer ring of one wheel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RingPair {
    pub inner: Ring,
    pub outer: Ring,
}

impl RingPair {
    /// Builds a pair from the notation of [`Key::parse`]. Panics on an unknown token,
    /// so it is meant for layouts written in code.
    pub fn parse(inner: [&str; 8], outer: [&str; 8]) -> Self {
        let p = |s: &str| Key::parse(s).unwrap_or_else(|| panic!("unknown key {s:?}"));
        RingPair {
            inner: inner.map(p),
            outer: outer.map(p),
        }
    }

    /// Reflects the pair left to right (E ↔ W), for the other hand.
    pub fn mirrored(&self) -> Self {
        let m = |r: &Ring| std::array::from_fn(|i| r[(8 - i) % 8]);
        RingPair {
            inner: m(&self.inner),
            outer: m(&self.outer),
        }
    }
}

/// What one wheel shows right now: two rings and the key under the centre.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rings {
    pub inner: Ring,
    pub outer: Ring,
    pub center: Key,
}

/// Which ring of a wheel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RingId {
    Inner,
    Outer,
}

/// Which app the keyboard is typing into. It changes auto-capitalisation,
/// the symbols in the ⋯ layer and the shortcuts offered during a chord.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Context {
    #[default]
    Text,
    Terminal,
    Finder,
    Browser,
}

/// A complete layout. [`Layout::spanish`] is the one the project was designed with;
/// build your own for other languages.
#[derive(Clone, Debug)]
pub struct Layout {
    /// Wheel of the consonant hand. Its centre is Space.
    pub consonants: RingPair,
    /// Wheel of the vowel hand. Its centre is Backspace.
    pub vowels: RingPair,
    /// ⋯ layer, left wheel.
    pub more_left: RingPair,
    /// ⋯ layer, right wheel, in text-like contexts.
    pub more_right_text: RingPair,
    /// ⋯ layer, right wheel, in a terminal.
    pub more_right_terminal: RingPair,
    /// Holding a letter still cycles through these variants, e.g. `('A', ['á'])`.
    pub accents: Vec<(char, Vec<char>)>,
}

impl Default for Layout {
    fn default() -> Self {
        Layout::spanish()
    }
}

impl Layout {
    /// The original layout. The 16 inner letters cover about 95 % of the letters
    /// written in Spanish; vowels and consonants sit on different thumbs so the
    /// thumbs alternate, and Space rides with the consonants to balance the load.
    pub fn spanish() -> Self {
        Layout {
            consonants: RingPair::parse(
                ["S", "R", "N", "P", "G", "B", "T", "C"],
                ["V", "Q", "H", "J", "W", "X", "Z", "F"],
            ),
            vowels: RingPair::parse(
                ["E", "I", "D", "U", "M", "L", "O", "A"],
                ["Y", "Ñ", "?", "K", "-", "¿", ".", ","],
            ),
            more_left: RingPair::parse(
                ["⌘", "⌥", "⌃", "Esc", "⏎", "⌦", "Tab", "⇧"],
                ["1", "2", "3", "4", "5", "(", ")", "="],
            ),
            more_right_text: RingPair::parse(
                ["↑", "/", "→", "&", "↓", "@", "←", "#"],
                ["6", "7", "8", "9", "0", "!", ":", ";"],
            ),
            more_right_terminal: RingPair::parse(
                ["↑", "|", "→", "~", "↓", "$", "←", "*"],
                ["6", "7", "8", "9", "0", ">", "<", "\\"],
            ),
            accents: vec![
                ('A', vec!['á']),
                ('E', vec!['é']),
                ('I', vec!['í']),
                ('O', vec!['ó']),
                ('U', vec!['ú', 'ü']),
            ],
        }
    }

    pub(crate) fn accents_of(&self, c: char) -> &[char] {
        self.accents
            .iter()
            .find(|(k, _)| *k == c)
            .map(|(_, v)| v.as_slice())
            .unwrap_or(&[])
    }
}

/// Shortcuts offered on the right wheel while the left thumb rests on a modifier.
pub(crate) fn chord_ring(m: Modifier, ctx: Context) -> RingPair {
    let inner: [&str; 8] = match (m, ctx) {
        (Modifier::Cmd, Context::Terminal) => ["C", "V", "K", "T", "W", "F", "N", "Tab"],
        (Modifier::Cmd, Context::Finder) => ["C", "V", "⌫", "N", "Z", "F", "O", "Tab"],
        (Modifier::Cmd, Context::Browser) => ["T", "W", "L", "R", "C", "V", "F", "Tab"],
        (Modifier::Cmd, Context::Text) => ["C", "V", "Z", "X", "A", "S", "F", "Tab"],
        (Modifier::Ctrl, Context::Terminal) => ["C", "D", "L", "R", "A", "E", "U", "Z"],
        (Modifier::Ctrl, _) => ["←", "↑", "↓", "→", "A", "E", "K", "Tab"],
        (Modifier::Alt, _) => ["←", "↑", "↓", "→", "⌫", "␣", "⏎", "Tab"],
        (Modifier::Shift, _) => ["←", "↑", "↓", "→", "Tab", "⏎", "␣", "⌫"],
    };
    const EXTRA: [&str; 20] = [
        "←", "↑", "↓", "→", "␣", "⏎", "⌫", "Esc", "W", "Q", "N", "O", "T", "S", "A", "Z", "D", "I",
        "K", "L",
    ];
    let mut outer = ["Esc"; 8];
    let mut n = 0;
    for e in EXTRA {
        if n < 8 && !inner.contains(&e) {
            outer[n] = e;
            n += 1;
        }
    }
    RingPair::parse(inner, outer)
}
