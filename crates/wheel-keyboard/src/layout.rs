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

/// A complete layout. Built-in ones: [`Layout::spanish`], [`Layout::english`],
/// [`Layout::italian`] and [`Layout::japanese`]; build your own for other languages.
///
/// All built-in layouts keep shared letters in the same place, so muscle memory
/// carries over from one language to another.
#[derive(Clone, Debug)]
pub struct Layout {
    /// Short code, e.g. `"es"`.
    pub code: &'static str,
    /// Name in English, e.g. `"Spanish"`.
    pub name: &'static str,
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
    /// Capitalise automatically at the start and after a full stop (Text context).
    /// Off for romaji input, where capitals would confuse the input method.
    pub auto_capitalize: bool,
}

impl Default for Layout {
    fn default() -> Self {
        Layout::spanish()
    }
}

fn more_left() -> RingPair {
    RingPair::parse(
        ["⌘", "⌥", "⌃", "Esc", "⏎", "⌦", "Tab", "⇧"],
        ["1", "2", "3", "4", "5", "(", ")", "="],
    )
}

fn more_right_text() -> RingPair {
    RingPair::parse(
        ["↑", "/", "→", "&", "↓", "@", "←", "#"],
        ["6", "7", "8", "9", "0", "!", ":", ";"],
    )
}

fn more_right_terminal() -> RingPair {
    RingPair::parse(
        ["↑", "|", "→", "~", "↓", "$", "←", "*"],
        ["6", "7", "8", "9", "0", ">", "<", "\\"],
    )
}

impl Layout {
    /// Codes of the built-in layouts, in the order of [`Layout::all`].
    pub const CODES: [&'static str; 4] = ["es", "en", "it", "ja"];

    /// All built-in layouts.
    pub fn all() -> Vec<Layout> {
        vec![
            Layout::spanish(),
            Layout::english(),
            Layout::italian(),
            Layout::japanese(),
        ]
    }

    /// A built-in layout by code (`"es"`, `"en"`, `"it"`, `"ja"`).
    pub fn by_code(code: &str) -> Option<Layout> {
        Layout::all()
            .into_iter()
            .find(|l| l.code.eq_ignore_ascii_case(code))
    }

    /// The original layout. The 16 inner letters cover about 95 % of the letters
    /// written in Spanish; vowels and consonants sit on different thumbs so the
    /// thumbs alternate, and Space rides with the consonants to balance the load.
    pub fn spanish() -> Self {
        Layout {
            code: "es",
            name: "Spanish",
            consonants: RingPair::parse(
                ["S", "R", "N", "P", "G", "B", "T", "C"],
                ["V", "Q", "H", "J", "W", "X", "Z", "F"],
            ),
            vowels: RingPair::parse(
                ["E", "I", "D", "U", "M", "L", "O", "A"],
                ["Y", "Ñ", "?", "K", "-", "¿", ".", ","],
            ),
            more_left: more_left(),
            more_right_text: more_right_text(),
            more_right_terminal: more_right_terminal(),
            accents: vec![
                ('A', vec!['á']),
                ('E', vec!['é']),
                ('I', vec!['í']),
                ('O', vec!['ó']),
                ('U', vec!['ú', 'ü']),
            ],
            auto_capitalize: true,
        }
    }

    /// English. Inner rings: E T A O I N S H R D L C U M W G, about 90 % of
    /// English letters. H and W take the inner slots of Spanish P and B; the
    /// apostrophe takes Ñ's place for contractions.
    pub fn english() -> Self {
        Layout {
            code: "en",
            name: "English",
            consonants: RingPair::parse(
                ["S", "R", "N", "H", "G", "W", "T", "C"],
                ["V", "Q", "F", "J", "P", "X", "Z", "B"],
            ),
            vowels: RingPair::parse(
                ["E", "I", "D", "U", "M", "L", "O", "A"],
                ["Y", "'", "?", "K", "-", "!", ".", ","],
            ),
            more_left: more_left(),
            more_right_text: more_right_text(),
            more_right_terminal: more_right_terminal(),
            accents: vec![],
            auto_capitalize: true,
        }
    }

    /// Italian. Inner rings: E A I O N L R T S C D P U M V G, about 96 % of
    /// Italian letters. V takes B's inner slot; the apostrophe (l'acqua,
    /// un'ora) takes Ñ's place. Holding a vowel gives à è é ì ò ù.
    pub fn italian() -> Self {
        Layout {
            code: "it",
            name: "Italian",
            consonants: RingPair::parse(
                ["S", "R", "N", "P", "G", "V", "T", "C"],
                ["B", "Q", "H", "J", "W", "X", "Z", "F"],
            ),
            vowels: RingPair::parse(
                ["E", "I", "D", "U", "M", "L", "O", "A"],
                ["Y", "'", "?", "K", "-", "!", ".", ","],
            ),
            more_left: more_left(),
            more_right_text: more_right_text(),
            more_right_terminal: more_right_terminal(),
            accents: vec![
                ('A', vec!['à']),
                ('E', vec!['è', 'é']),
                ('I', vec!['ì']),
                ('O', vec!['ò']),
                ('U', vec!['ù']),
            ],
            auto_capitalize: true,
        }
    }

    /// Japanese, typed as romaji for the system input method (kana mode) to
    /// convert. Vowels are about half of romaji, so the vowel thumb also gets
    /// N (ん and the n- row), M and R; K, H, Y, Z and D join the consonant
    /// thumb. `-` gives the long-vowel mark ー, `[` `]` give 「」, and X gives
    /// small kana (xtu → っ). No automatic capitals.
    pub fn japanese() -> Self {
        Layout {
            code: "ja",
            name: "Japanese (romaji)",
            consonants: RingPair::parse(
                ["S", "K", "H", "Y", "G", "Z", "T", "D"],
                ["W", "B", "P", "J", "C", "F", "X", "V"],
            ),
            vowels: RingPair::parse(
                ["E", "I", "N", "U", "M", "R", "O", "A"],
                ["-", "?", "L", "Q", "[", "]", ".", ","],
            ),
            more_left: more_left(),
            more_right_text: more_right_text(),
            more_right_terminal: more_right_terminal(),
            accents: vec![],
            auto_capitalize: false,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn letters(l: &Layout) -> Vec<char> {
        let mut v: Vec<char> = [&l.consonants, &l.vowels]
            .iter()
            .flat_map(|p| p.inner.iter().chain(p.outer.iter()))
            .filter_map(|k| match k {
                Key::Char(c) if c.is_alphabetic() => Some(*c),
                _ => None,
            })
            .collect();
        v.sort();
        v
    }

    #[test]
    fn every_layout_reaches_a_to_z_once() {
        for l in Layout::all() {
            let got = letters(&l);
            let mut dedup = got.clone();
            dedup.dedup();
            assert_eq!(got, dedup, "{}: a letter appears twice", l.code);
            for c in 'A'..='Z' {
                assert!(got.contains(&c), "{}: missing {c}", l.code);
            }
        }
        assert!(letters(&Layout::spanish()).contains(&'Ñ'));
    }

    #[test]
    fn shared_letters_keep_their_place() {
        // Vowels and the most common consonants sit in the same spot everywhere
        // except where a language needs the slot (Japanese romaji).
        let es = Layout::spanish();
        for l in [Layout::english(), Layout::italian()] {
            assert_eq!(l.vowels.inner, es.vowels.inner, "{}", l.code);
            for i in [0, 1, 2, 6, 7] {
                assert_eq!(l.consonants.inner[i], es.consonants.inner[i], "{}", l.code);
            }
        }
        let ja = Layout::japanese();
        for i in [0, 1, 3, 6, 7] {
            assert_eq!(ja.vowels.inner[i], es.vowels.inner[i]);
        }
    }

    #[test]
    fn codes_match() {
        for (l, code) in Layout::all().iter().zip(Layout::CODES) {
            assert_eq!(l.code, code);
            assert_eq!(Layout::by_code(code).unwrap().code, code);
        }
        assert!(Layout::by_code("xx").is_none());
    }
}
