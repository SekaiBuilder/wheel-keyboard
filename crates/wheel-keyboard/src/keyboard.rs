//! The keyboard state machine: feed it touches, get back what to type.

use crate::geometry::{self, BORDER_BAND_DEG, Hit};
use crate::key::{Key, Modifier, Modifiers};
use crate::layout::{Context, Layout, RingId, Rings, chord_ring};
use crate::model::Model;

/// Holding a vowel this long (ms) turns it into its accented form.
pub const ACCENT_DELAY_MS: u64 = 450;
/// Holding the Backspace centre this long (ms) starts repeating.
pub const REPEAT_DELAY_MS: u64 = 450;
/// Interval between repeated Backspaces (ms).
pub const REPEAT_INTERVAL_MS: u64 = 75;
/// Resting the left thumb on a modifier this long (ms) starts a chord.
pub const CHORD_DELAY_MS: u64 = 160;
/// Two taps on a modifier within this time (ms) lock it.
pub const DOUBLE_TAP_MS: u64 = 400;
const MIN_LETTERS_TO_CORRECT: u32 = 60;
const MIN_SUPPORT: u32 = 8;
const HIGHLIGHT_MIN_P: f64 = 0.12;
const TEXT_MEMORY: usize = 256;

/// Left or right wheel (left or right half of the touch area).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    fn index(self) -> usize {
        match self {
            Side::Left => 0,
            Side::Right => 1,
        }
    }
}

/// Which wheel layer is showing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Layer {
    #[default]
    Letters,
    /// The ⋯ layer: modifiers, numbers, arrows and symbols.
    More,
}

/// The buttons above the wheels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    /// ⇧: one tap for the next key, double tap to lock.
    Shift,
    /// ⋯: toggles the ⋯ layer.
    More,
    Enter,
    /// あ/A on JIS Macs: toggles kana (かな) and alphanumeric (英数) input.
    Kana,
}

/// State of one modifier.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ModState {
    #[default]
    Off,
    /// Applies to the next key, then turns off.
    Once,
    /// Stays on until tapped again.
    Locked,
}

/// Settings.
#[derive(Clone, Debug)]
pub struct Config {
    /// Which thumb gets the vowels. The other gets the consonants and Space.
    pub vowel_hand: Side,
    pub context: Context,
    /// The target has a JIS keyboard: enables the あ/A button and ¥.
    pub jis: bool,
    /// Learn, highlight and suggest.
    pub prediction: bool,
    /// Let prediction fix taps that land on the border between two letters.
    pub border_correction: bool,
    /// Margin past a ring border before switching rings (fraction of the radius, 0–0.08).
    pub ring_margin: f32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            vowel_hand: Side::Right,
            context: Context::Text,
            jis: false,
            prediction: true,
            border_correction: true,
            ring_margin: geometry::DEFAULT_RING_MARGIN,
        }
    }
}

/// Something for the host to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// Insert this text at the cursor.
    Text(String),
    /// Delete this many characters before the cursor.
    Backspace(usize),
    /// Press a key that is not plain text (Enter, Tab, arrows, Esc, Delete) or a
    /// shortcut such as ⌘C.
    Key { key: Key, mods: Modifiers },
    /// Switch the input method: `true` kana (かな), `false` alphanumeric (英数).
    Kana(bool),
}

/// What changed when the thumb moved, for haptics and redrawing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MoveFeedback {
    /// The selected key changed: redraw.
    pub changed: bool,
    /// The thumb crossed from one ring to the other: a light tick or flash.
    pub ring_crossed: bool,
}

/// The current selection on a wheel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Selection {
    pub ring: RingId,
    pub dir: u8,
    pub key: Key,
    /// Set when prediction moved the selection from the key under the thumb.
    pub corrected_from: Option<Key>,
    /// The direction under the thumb before any correction.
    pub raw_dir: u8,
}

/// What will be typed if the thumb lifts now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Preview {
    /// Text or key glyphs, with shortcut modifiers in front (e.g. "⌘C").
    pub label: String,
    /// The key under the thumb, when prediction corrected it.
    pub corrected_from: Option<Key>,
    /// Lifting now types nothing (the thumb left the centre and came back).
    pub cancel: bool,
}

#[derive(Clone, Debug, Default)]
struct Wheel {
    down: bool,
    x0: f32,
    y0: f32,
    radius: f32,
    exited: bool,
    hit: Option<Hit>,
    sel: Option<Selection>,
    sel_since: u64,
    repeat_at: Option<u64>,
    repeated: bool,
}

#[derive(Clone, Copy, Debug)]
struct Chord {
    modifier: Modifier,
    used: bool,
}

/// The keyboard. Feed it touches with [`touch_down`](Keyboard::touch_down),
/// [`touch_move`](Keyboard::touch_move) and [`touch_up`](Keyboard::touch_up), call
/// [`tick`](Keyboard::tick) every frame while a thumb is down, and apply the
/// [`Action`]s it returns.
///
/// It keeps its own copy of the last few hundred characters it typed, which is
/// what prediction and auto-capitalisation look at. If the text changes
/// elsewhere (the user moved the cursor, pasted…), tell it with
/// [`set_text_before_cursor`](Keyboard::set_text_before_cursor).
#[derive(Clone, Debug)]
pub struct Keyboard {
    pub config: Config,
    layout: Layout,
    model: Model,
    wheels: [Wheel; 2],
    mods: [ModState; 4],
    last_tap: [Option<u64>; 4],
    layer: Layer,
    chord: Option<Chord>,
    text: String,
    last_fix: Option<char>,
    kana: bool,
    secure: bool,
}

impl Default for Keyboard {
    fn default() -> Self {
        Keyboard::new(Config::default(), Layout::default())
    }
}

impl Keyboard {
    pub fn new(config: Config, layout: Layout) -> Self {
        Keyboard {
            config,
            layout,
            model: Model::new(),
            wheels: Default::default(),
            mods: Default::default(),
            last_tap: Default::default(),
            layer: Layer::Letters,
            chord: None,
            text: String::new(),
            last_fix: None,
            kana: false,
            secure: false,
        }
    }

    // ----- Model and state -----

    pub fn model(&self) -> &Model {
        &self.model
    }
    pub fn set_model(&mut self, model: Model) {
        self.model = model;
    }
    pub fn clear_model(&mut self) {
        self.model.clear();
    }
    pub fn layout(&self) -> &Layout {
        &self.layout
    }
    pub fn set_layout(&mut self, layout: Layout) {
        self.layout = layout;
    }
    pub fn layer(&self) -> Layer {
        self.layer
    }
    pub fn modifier(&self, m: Modifier) -> ModState {
        self.mods[m.index()]
    }
    /// Turns a modifier off (e.g. the user tapped its indicator).
    pub fn release_modifier(&mut self, m: Modifier) {
        self.mods[m.index()] = ModState::Off;
    }
    pub fn kana(&self) -> bool {
        self.kana
    }

    /// The focused field is a password field: while on, nothing is learnt and
    /// nothing is suggested.
    pub fn set_secure_input(&mut self, secure: bool) {
        self.secure = secure;
    }
    pub fn secure_input(&self) -> bool {
        self.secure
    }

    /// Replaces the keyboard's idea of the text before the cursor.
    pub fn set_text_before_cursor(&mut self, text: &str) {
        self.text = tail(text, TEXT_MEMORY).to_string();
        self.last_fix = None;
    }
    pub fn text_before_cursor(&self) -> &str {
        &self.text
    }

    fn learning(&self) -> bool {
        self.config.prediction && !self.secure
    }

    /// Modifiers in effect: tapped or locked ones, plus a chord.
    pub fn active_modifiers(&self) -> Modifiers {
        let mut m = Modifiers::NONE;
        for x in Modifier::ALL {
            if self.mods[x.index()] != ModState::Off {
                m = m.with(x);
            }
        }
        if let Some(c) = self.chord {
            m = m.with(c.modifier);
        }
        m
    }

    fn has_command_mods(&self) -> bool {
        !self.active_modifiers().command_part().is_empty()
    }

    fn auto_cap(&self) -> bool {
        if self.config.context != Context::Text {
            return false;
        }
        if self.text.is_empty() || self.text.ends_with('\n') {
            return true;
        }
        let mut it = self.text.chars().rev();
        matches!((it.next(), it.next()), (Some(s), Some(p)) if s.is_whitespace() && ".!?¿¡".contains(p))
    }

    /// Whether the next letter comes out in upper case.
    pub fn upper_case(&self) -> bool {
        self.mods[Modifier::Shift.index()] != ModState::Off || self.auto_cap()
    }

    // ----- What to draw -----

    /// The keys a wheel shows right now.
    pub fn rings(&self, side: Side) -> Rings {
        let vowel_side = self.config.vowel_hand;
        let center = if side == vowel_side {
            Key::Backspace
        } else {
            Key::Space
        };
        if side == Side::Right {
            if let Some(c) = self.chord {
                let r = chord_ring(c.modifier, self.config.context);
                return Rings {
                    inner: r.inner,
                    outer: r.outer,
                    center,
                };
            }
        }
        if self.layer == Layer::More {
            if side == Side::Left {
                let r = &self.layout.more_left;
                return Rings {
                    inner: r.inner,
                    outer: r.outer,
                    center,
                };
            }
            let r = match self.config.context {
                Context::Terminal => &self.layout.more_right_terminal,
                _ => &self.layout.more_right_text,
            };
            let mut outer = r.outer;
            if self.config.jis {
                for k in outer.iter_mut() {
                    if *k == Key::Char(';') {
                        *k = Key::Char('¥');
                    }
                }
            }
            return Rings {
                inner: r.inner,
                outer,
                center,
            };
        }
        let mut r = if side == vowel_side {
            self.layout.vowels.clone()
        } else {
            self.layout.consonants.clone()
        };
        if vowel_side == Side::Left {
            r = r.mirrored();
        }
        Rings {
            inner: r.inner,
            outer: r.outer,
            center,
        }
    }

    /// How a key should be labelled now: letters follow Shift and auto-capitals.
    pub fn label(&self, key: Key) -> String {
        match key {
            Key::Char(c) if c.is_alphabetic() => {
                if self.upper_case() || self.has_command_mods() {
                    c.to_string()
                } else {
                    c.to_lowercase().collect()
                }
            }
            k => k.glyph(),
        }
    }

    pub fn selection(&self, side: Side) -> Option<Selection> {
        self.wheels[side.index()].sel
    }

    pub fn is_down(&self, side: Side) -> bool {
        self.wheels[side.index()].down
    }

    /// Where the wheel's thumb touched down, if it is down.
    pub fn touch_origin(&self, side: Side) -> Option<(f32, f32)> {
        let w = &self.wheels[side.index()];
        w.down.then_some((w.x0, w.y0))
    }

    /// Up to three likely next letters with their probability. Highlight them,
    /// but never resize or move them: muscle memory comes first.
    pub fn highlights(&self) -> Vec<(Key, f64)> {
        if !self.learning()
            || self.layer != Layer::Letters
            || self.chord.is_some()
            || self.has_command_mods()
        {
            return vec![];
        }
        let p = self.model.predict(current_word(&self.text));
        if p.support < MIN_SUPPORT {
            return vec![];
        }
        p.ranked()
            .into_iter()
            .take(3)
            .filter(|(_, pr)| *pr >= HIGHLIGHT_MIN_P)
            .map(|(c, pr)| (Key::Char(c.to_uppercase().next().unwrap_or(c)), pr))
            .collect()
    }

    /// Up to three suggested words, cased as they would be typed.
    pub fn suggestions(&self) -> Vec<String> {
        self.suggestion_parts().0
    }

    fn suggestion_parts(&self) -> (Vec<String>, Vec<String>, usize, Option<String>) {
        if !self.learning() {
            return (vec![], vec![], 0, None);
        }
        let prefix = current_word(&self.text);
        let before = &self.text[..self.text.len() - prefix.len()];
        let prev = word_before_space(before);
        let raw = self.model.suggest(prefix, prev.as_deref(), 3);
        let cap = prefix.chars().next().is_some_and(char::is_uppercase)
            || (prefix.is_empty() && self.auto_cap());
        let shown = raw
            .iter()
            .map(|w| if cap { capitalise(w) } else { w.clone() })
            .collect();
        (shown, raw, prefix.chars().count(), prev)
    }

    /// What lifting the thumb would type now.
    pub fn preview(&self, side: Side, now_ms: u64) -> Option<Preview> {
        let w = &self.wheels[side.index()];
        if !w.down {
            return None;
        }
        if let Some(sel) = w.sel {
            let mut label = match self.accented(sel.key, self.accent_level(w, now_ms)) {
                Some(a) if self.upper_case() => a.to_uppercase().collect(),
                Some(a) => a.to_string(),
                None => self.label(sel.key),
            };
            let cmd = self.active_modifiers().command_part();
            if !cmd.is_empty() && !sel.key.is_modifier() {
                label = cmd.glyphs() + &label;
            }
            return Some(Preview {
                label,
                corrected_from: sel.corrected_from,
                cancel: false,
            });
        }
        if w.exited {
            return Some(Preview {
                label: "×".into(),
                corrected_from: None,
                cancel: true,
            });
        }
        Some(Preview {
            label: self.label(self.rings(side).center),
            corrected_from: None,
            cancel: false,
        })
    }

    // ----- Touches -----

    /// A thumb touched the wheel area. `radius` is the wheel radius in the same
    /// units as `x` and `y`. A second touch on a wheel that is already down is ignored.
    pub fn touch_down(&mut self, side: Side, x: f32, y: f32, radius: f32, now_ms: u64) {
        let backspace_center = self.rings(side).center == Key::Backspace;
        let w = &mut self.wheels[side.index()];
        if w.down {
            return;
        }
        *w = Wheel {
            down: true,
            x0: x,
            y0: y,
            radius,
            sel_since: now_ms,
            ..Default::default()
        };
        if backspace_center {
            w.repeat_at = Some(now_ms + REPEAT_DELAY_MS);
        }
    }

    /// The thumb moved.
    pub fn touch_move(&mut self, side: Side, x: f32, y: f32, now_ms: u64) -> MoveFeedback {
        let i = side.index();
        if !self.wheels[i].down {
            return MoveFeedback::default();
        }
        let (dx, dy, radius, prev_hit, prev_sel) = {
            let w = &self.wheels[i];
            (x - w.x0, y - w.y0, w.radius, w.hit, w.sel)
        };
        let hit = geometry::hit(
            prev_hit,
            dx,
            dy,
            radius,
            self.config.ring_margin.clamp(0.0, 0.08),
        );
        let sel = hit.map(|h| self.select(side, h));
        let w = &mut self.wheels[i];
        if hit.is_some() {
            w.exited = true;
        }
        w.hit = hit;
        w.sel = sel;
        let same = match (prev_sel, sel) {
            (None, None) => true,
            (Some(a), Some(b)) => a.ring == b.ring && a.dir == b.dir,
            _ => false,
        };
        let ring_crossed = matches!((prev_sel, sel), (Some(a), Some(b)) if a.ring != b.ring);
        let corrected_changed =
            matches!((prev_sel, sel), (Some(a), Some(b)) if a.corrected_from != b.corrected_from);
        if !same {
            w.sel_since = now_ms;
            if side == Side::Left {
                if let Some(c) = self.chord {
                    if !c.used && !sel.is_some_and(|s| s.key == Key::Modifier(c.modifier)) {
                        self.chord = None;
                    }
                }
            }
        }
        self.update_chord(now_ms);
        MoveFeedback {
            changed: !same || corrected_changed,
            ring_crossed,
        }
    }

    /// Call regularly (every frame) while a thumb is down: it repeats Backspace
    /// and starts chords.
    pub fn tick(&mut self, now_ms: u64) -> Vec<Action> {
        self.update_chord(now_ms);
        let mut out = vec![];
        for side in [Side::Left, Side::Right] {
            let i = side.index();
            while let Some(at) = self.wheels[i].repeat_at {
                let w = &self.wheels[i];
                if !w.down || w.exited || w.sel.is_some() {
                    self.wheels[i].repeat_at = None;
                    break;
                }
                if at > now_ms {
                    break;
                }
                self.wheels[i].repeated = true;
                self.wheels[i].repeat_at = Some(at + REPEAT_INTERVAL_MS);
                out.extend(self.emit(Key::Backspace, 0, None));
            }
        }
        out
    }

    /// The thumb lifted.
    pub fn touch_up(&mut self, side: Side, now_ms: u64) -> Vec<Action> {
        let i = side.index();
        if !self.wheels[i].down {
            return vec![];
        }
        let w = std::mem::take(&mut self.wheels[i]);
        let mut out = vec![];
        if let Some(sel) = w.sel {
            if let Key::Modifier(m) = sel.key {
                let chord_used = side == Side::Left && self.chord.is_some_and(|c| c.used);
                if !chord_used {
                    self.tap_modifier(m, now_ms);
                }
            } else {
                let acc = self.accent_level(&w, now_ms);
                out = self.emit(sel.key, acc, sel.corrected_from);
            }
        } else if !w.exited && !w.repeated {
            out = self.emit(self.rings(side).center, 0, None);
        }
        if side == Side::Left {
            self.chord = None;
        }
        out
    }

    /// The touch was cancelled by the system: type nothing.
    pub fn touch_cancel(&mut self, side: Side) {
        self.wheels[side.index()] = Wheel::default();
        if side == Side::Left {
            self.chord = None;
        }
    }

    /// A button above the wheels was pressed.
    pub fn press(&mut self, button: Button, now_ms: u64) -> Vec<Action> {
        match button {
            Button::Shift => {
                self.tap_modifier(Modifier::Shift, now_ms);
                vec![]
            }
            Button::More => {
                self.layer = if self.layer == Layer::More {
                    Layer::Letters
                } else {
                    Layer::More
                };
                vec![]
            }
            Button::Enter => self.emit(Key::Enter, 0, None),
            Button::Kana => {
                self.kana = !self.kana;
                vec![Action::Kana(self.kana)]
            }
        }
    }

    /// Types suggestion `index` from [`suggestions`](Keyboard::suggestions), followed by a space.
    pub fn pick_suggestion(&mut self, index: usize) -> Vec<Action> {
        let (shown, raw, prefix_len, prev) = self.suggestion_parts();
        let (Some(word), Some(raw_word)) = (shown.get(index), raw.get(index)) else {
            return vec![];
        };
        let mut out = vec![];
        if prefix_len > 0 {
            out.push(Action::Backspace(prefix_len));
            for _ in 0..prefix_len {
                self.text.pop();
            }
        }
        let typed = format!("{word} ");
        self.push_text(&typed);
        out.push(Action::Text(typed));
        self.last_fix = None;
        if self.learning() {
            self.model.learn_word(raw_word, prev.as_deref());
        }
        out
    }

    // ----- Internals -----

    fn select(&self, side: Side, h: Hit) -> Selection {
        let rings = self.rings(side);
        let keys = if h.ring == RingId::Inner {
            rings.inner
        } else {
            rings.outer
        };
        let sel = Selection {
            ring: h.ring,
            dir: h.dir,
            key: keys[h.dir as usize],
            corrected_from: None,
            raw_dir: h.dir,
        };
        self.correct(sel, &keys, h.angle)
    }

    /// The border rule: in the centre of a letter you always get that letter; only
    /// in the band next to the border may a much likelier neighbour win.
    fn correct(&self, sel: Selection, keys: &[Key; 8], angle: f32) -> Selection {
        if !self.config.border_correction
            || !self.learning()
            || self.layer != Layer::Letters
            || self.chord.is_some()
            || self.has_command_mods()
            || !sel.key.is_letter()
            || self.model.letters < MIN_LETTERS_TO_CORRECT
        {
            return sel;
        }
        let off = geometry::angle_diff(angle, sel.dir as f32 * 45.0);
        if off.abs() < 22.5 - BORDER_BAND_DEG {
            return sel;
        }
        let nd = (sel.dir + if off > 0.0 { 1 } else { 7 }) % 8;
        let nk = keys[nd as usize];
        let (Key::Char(c_sel), Key::Char(c_n)) = (sel.key, nk) else {
            return sel;
        };
        if !nk.is_letter() {
            return sel;
        }
        let p = self.model.predict(current_word(&self.text));
        if p.support < MIN_SUPPORT {
            return sel;
        }
        let (Some(ps), Some(pn)) = (p.prob(c_sel), p.prob(c_n)) else {
            return sel;
        };
        let caution = 3.0 + (self.model.rejections as f64 * 0.5).min(10.0);
        if pn < caution * ps {
            return sel;
        }
        let dn = geometry::angle_diff(angle, nd as f32 * 45.0);
        if (geometry::gauss(dn) as f64) * pn <= (geometry::gauss(off) as f64) * ps {
            return sel;
        }
        Selection {
            dir: nd,
            key: nk,
            corrected_from: Some(sel.key),
            ..sel
        }
    }

    fn update_chord(&mut self, now_ms: u64) {
        let w = &self.wheels[Side::Left.index()];
        if self.chord.is_some() || self.layer != Layer::More || !w.down {
            return;
        }
        if let Some(Selection {
            key: Key::Modifier(m),
            ..
        }) = w.sel
        {
            if now_ms.saturating_sub(w.sel_since) >= CHORD_DELAY_MS {
                self.chord = Some(Chord {
                    modifier: m,
                    used: false,
                });
            }
        }
    }

    fn accent_level(&self, w: &Wheel, now_ms: u64) -> u8 {
        let Some(Selection {
            key: Key::Char(c), ..
        }) = w.sel
        else {
            return 0;
        };
        let n = self.layout.accents_of(c).len() as u64;
        if n == 0 || self.has_command_mods() {
            return 0;
        }
        ((now_ms.saturating_sub(w.sel_since) / ACCENT_DELAY_MS) % (n + 1)) as u8
    }

    fn accented(&self, key: Key, level: u8) -> Option<char> {
        let Key::Char(c) = key else { return None };
        if level == 0 {
            return None;
        }
        let v = self.layout.accents_of(c);
        v.get((level as usize).min(v.len()).checked_sub(1)?)
            .copied()
    }

    fn tap_modifier(&mut self, m: Modifier, now_ms: u64) {
        let i = m.index();
        let quick = self.last_tap[i].is_some_and(|t| now_ms.saturating_sub(t) < DOUBLE_TAP_MS);
        self.mods[i] = match self.mods[i] {
            ModState::Locked => ModState::Off,
            ModState::Once if quick => ModState::Locked,
            ModState::Once => ModState::Off,
            ModState::Off => ModState::Once,
        };
        self.last_tap[i] = Some(now_ms);
        if self.layer == Layer::More && self.mods[i] != ModState::Off {
            self.layer = Layer::Letters;
        }
    }

    fn push_text(&mut self, s: &str) {
        self.text.push_str(s);
        if self.text.chars().count() > TEXT_MEMORY * 2 {
            self.text = tail(&self.text, TEXT_MEMORY).to_string();
        }
    }

    fn commit_word(&mut self) {
        if !self.learning() {
            return;
        }
        let word = current_word(&self.text);
        if word.is_empty() {
            return;
        }
        let before = &self.text[..self.text.len() - word.len()];
        let prev = last_word(before);
        let (word, prev) = (word.to_string(), prev.map(str::to_string));
        self.model.learn_word(&word, prev.as_deref());
    }

    fn emit(&mut self, key: Key, accent: u8, corrected_from: Option<Key>) -> Vec<Action> {
        let mods = self.active_modifiers();
        if let Some(c) = &mut self.chord {
            c.used = true;
        }
        let fix = self.last_fix.take();
        let mut out = vec![];
        if !mods.command_part().is_empty() {
            out.push(Action::Key { key, mods });
        } else {
            match key {
                Key::Char(c) => {
                    let letter = c.is_alphabetic();
                    let mut s: String = match self.accented(key, accent) {
                        Some(a) => a.to_string(),
                        None if letter => c.to_lowercase().collect(),
                        None => c.to_string(),
                    };
                    let up = letter && self.upper_case();
                    if up {
                        s = s.to_uppercase();
                    }
                    if !letter {
                        self.commit_word();
                    }
                    self.push_text(&s);
                    if let Some(Key::Char(f)) = corrected_from {
                        self.last_fix = Some(if up {
                            f
                        } else {
                            f.to_lowercase().next().unwrap_or(f)
                        });
                    }
                    out.push(Action::Text(s));
                }
                Key::Space => {
                    self.commit_word();
                    self.push_text(" ");
                    out.push(Action::Text(" ".into()));
                }
                Key::Backspace => {
                    self.text.pop();
                    out.push(Action::Backspace(1));
                    if let Some(literal) = fix {
                        // Backspace right after a correction: put back the letter under the thumb.
                        self.push_text(&literal.to_string());
                        out.push(Action::Text(literal.to_string()));
                        self.model.rejections += 1;
                    }
                }
                Key::Enter => {
                    self.commit_word();
                    self.push_text("\n");
                    out.push(Action::Key { key, mods });
                }
                Key::Modifier(_) => {}
                other => out.push(Action::Key { key: other, mods }),
            }
        }
        for s in self.mods.iter_mut() {
            if *s == ModState::Once {
                *s = ModState::Off;
            }
        }
        out
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphabetic()
}

/// The letters at the end of `text`.
fn current_word(text: &str) -> &str {
    let start = text
        .char_indices()
        .rev()
        .take_while(|(_, c)| is_word_char(*c))
        .last()
        .map(|(i, _)| i);
    start.map_or("", |i| &text[i..])
}

/// The last word in `text`, skipping anything that is not a letter after it.
fn last_word(text: &str) -> Option<&str> {
    let trimmed = text.trim_end_matches(|c: char| !is_word_char(c));
    let w = current_word(trimmed);
    (!w.is_empty()).then_some(w)
}

/// The word just before trailing whitespace, lower-cased.
fn word_before_space(text: &str) -> Option<String> {
    let trimmed = text.trim_end_matches(char::is_whitespace);
    if trimmed.len() == text.len() {
        return None;
    }
    let w = current_word(trimmed);
    (!w.is_empty()).then(|| w.to_lowercase())
}

fn capitalise(w: &str) -> String {
    let mut c = w.chars();
    c.next()
        .map_or(String::new(), |f| f.to_uppercase().chain(c).collect())
}

fn tail(s: &str, n: usize) -> &str {
    let count = s.chars().count();
    if count <= n {
        return s;
    }
    let skip = s.char_indices().nth(count - n).map_or(0, |(i, _)| i);
    &s[skip..]
}
