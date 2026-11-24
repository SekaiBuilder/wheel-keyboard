//! Keys and modifiers.

/// A modifier key. `Cmd` is Command on macOS and the Super/Windows key elsewhere.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Modifier {
    Ctrl,
    Alt,
    Shift,
    Cmd,
}

impl Modifier {
    /// All modifiers, in the order macOS prints them (⌃⌥⇧⌘).
    pub const ALL: [Modifier; 4] = [
        Modifier::Ctrl,
        Modifier::Alt,
        Modifier::Shift,
        Modifier::Cmd,
    ];

    pub(crate) fn index(self) -> usize {
        match self {
            Modifier::Ctrl => 0,
            Modifier::Alt => 1,
            Modifier::Shift => 2,
            Modifier::Cmd => 3,
        }
    }

    pub fn glyph(self) -> &'static str {
        match self {
            Modifier::Ctrl => "⌃",
            Modifier::Alt => "⌥",
            Modifier::Shift => "⇧",
            Modifier::Cmd => "⌘",
        }
    }
}

/// A set of modifiers held together.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Modifiers(u8);

impl Modifiers {
    pub const NONE: Modifiers = Modifiers(0);
    pub const CTRL: Modifiers = Modifiers(1);
    pub const ALT: Modifiers = Modifiers(2);
    pub const SHIFT: Modifiers = Modifiers(4);
    pub const CMD: Modifiers = Modifiers(8);

    pub const fn bits(self) -> u8 {
        self.0
    }
    pub const fn from_bits(bits: u8) -> Self {
        Modifiers(bits & 0b1111)
    }
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
    pub fn contains(self, m: Modifier) -> bool {
        self.0 & (1 << m.index()) != 0
    }
    pub fn with(self, m: Modifier) -> Self {
        Modifiers(self.0 | (1 << m.index()))
    }
    pub fn without(self, m: Modifier) -> Self {
        Modifiers(self.0 & !(1 << m.index()))
    }
    /// The modifiers that turn a key into a shortcut (everything but Shift).
    pub fn command_part(self) -> Self {
        self.without(Modifier::Shift)
    }
    /// Glyphs in macOS order, e.g. "⌃⌘".
    pub fn glyphs(self) -> String {
        Modifier::ALL
            .iter()
            .filter(|m| self.contains(**m))
            .map(|m| m.glyph())
            .collect()
    }
}

/// One key on a wheel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    /// A printable character. Letters are stored in upper case.
    Char(char),
    Space,
    Backspace,
    Enter,
    Tab,
    Escape,
    /// Forward delete.
    Delete,
    Up,
    Down,
    Left,
    Right,
    Modifier(Modifier),
}

impl Key {
    pub fn is_letter(self) -> bool {
        matches!(self, Key::Char(c) if c.is_alphabetic())
    }

    pub fn is_modifier(self) -> bool {
        matches!(self, Key::Modifier(_))
    }

    /// A short glyph for drawing the key. Letters come back as stored (upper case).
    pub fn glyph(self) -> String {
        match self {
            Key::Char(c) => c.to_string(),
            Key::Space => "␣".into(),
            Key::Backspace => "⌫".into(),
            Key::Enter => "⏎".into(),
            Key::Tab => "⇥".into(),
            Key::Escape => "esc".into(),
            Key::Delete => "⌦".into(),
            Key::Up => "↑".into(),
            Key::Down => "↓".into(),
            Key::Left => "←".into(),
            Key::Right => "→".into(),
            Key::Modifier(m) => m.glyph().into(),
        }
    }

    /// Parses the notation used by the built-in layouts: a single character,
    /// or one of `␣ ⌫ ⏎ Tab Esc ⌦ ↑ ↓ ← → ⌃ ⌥ ⇧ ⌘`.
    pub fn parse(s: &str) -> Option<Key> {
        Some(match s {
            "␣" => Key::Space,
            "⌫" => Key::Backspace,
            "⏎" => Key::Enter,
            "Tab" | "⇥" => Key::Tab,
            "Esc" => Key::Escape,
            "⌦" => Key::Delete,
            "↑" => Key::Up,
            "↓" => Key::Down,
            "←" => Key::Left,
            "→" => Key::Right,
            "⌃" => Key::Modifier(Modifier::Ctrl),
            "⌥" => Key::Modifier(Modifier::Alt),
            "⇧" => Key::Modifier(Modifier::Shift),
            "⌘" => Key::Modifier(Modifier::Cmd),
            _ => {
                let mut it = s.chars();
                let c = it.next()?;
                if it.next().is_some() {
                    return None;
                }
                Key::Char(c.to_uppercase().next().unwrap_or(c))
            }
        })
    }
}
