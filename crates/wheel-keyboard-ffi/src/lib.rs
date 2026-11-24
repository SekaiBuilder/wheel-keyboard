//! C API for the wheel keyboard. The header is `include/wheel_keyboard.h`.
//!
//! One opaque `WkKeyboard` per keyboard. Touch functions queue actions; read them
//! with `wk_next_action` until it returns false. Strings are UTF-8. Functions that
//! fill a buffer return the length the full string needs (without the NUL) and
//! write a NUL-terminated string only when it fits.

use std::collections::VecDeque;
use std::ffi::{CStr, CString, c_char};

use wheel_keyboard::{
    Action, Button, Context, Key, Keyboard, Layer, Layout, ModState, Model, Modifier, Modifiers,
    RingId, Side,
};

pub struct WkKeyboard {
    kb: Keyboard,
    queue: VecDeque<Action>,
    current: CString,
}

#[repr(C)]
pub struct WkAction {
    /// 1 text, 2 backspace, 3 key, 4 kana.
    pub kind: u32,
    /// For kind 3: a `WK_KEY_*` code.
    pub key: u32,
    /// For kind 3 with `WK_KEY_CHAR`: the Unicode scalar of the character.
    pub codepoint: u32,
    /// For kind 3: `WK_MOD_*` bits.
    pub mods: u8,
    /// For kind 2: characters to delete. For kind 4: 1 kana, 0 alphanumeric.
    pub count: u32,
    /// For kind 1: the text, valid until the next call on this keyboard.
    pub text: *const c_char,
}

fn side(s: u32) -> Side {
    if s == 0 { Side::Left } else { Side::Right }
}

fn key_code(k: Key) -> (u32, u32) {
    match k {
        Key::Char(c) => (0, c as u32),
        Key::Space => (1, 0),
        Key::Backspace => (2, 0),
        Key::Enter => (3, 0),
        Key::Tab => (4, 0),
        Key::Escape => (5, 0),
        Key::Delete => (6, 0),
        Key::Up => (7, 0),
        Key::Down => (8, 0),
        Key::Left => (9, 0),
        Key::Right => (10, 0),
        Key::Modifier(Modifier::Ctrl) => (11, 0),
        Key::Modifier(Modifier::Alt) => (12, 0),
        Key::Modifier(Modifier::Shift) => (13, 0),
        Key::Modifier(Modifier::Cmd) => (14, 0),
    }
}

fn modifier(m: u32) -> Option<Modifier> {
    Some(match m {
        1 => Modifier::Ctrl,
        2 => Modifier::Alt,
        4 => Modifier::Shift,
        8 => Modifier::Cmd,
        _ => return None,
    })
}

/// Copies `s` into `buf` if it fits; returns the length it needs.
unsafe fn fill(s: &str, buf: *mut c_char, cap: usize) -> usize {
    let bytes = s.as_bytes();
    if !buf.is_null() && bytes.len() < cap {
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf as *mut u8, bytes.len());
            *buf.add(bytes.len()) = 0;
        }
    }
    bytes.len()
}

unsafe fn kb<'a>(p: *mut WkKeyboard) -> Option<&'a mut WkKeyboard> {
    unsafe { p.as_mut() }
}

unsafe fn str_arg<'a>(p: *const c_char) -> Option<&'a str> {
    if p.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(p) }.to_str().ok()
}

fn queue(w: &mut WkKeyboard, actions: Vec<Action>) -> usize {
    w.queue.extend(actions);
    w.queue.len()
}

/// Creates a keyboard with the default (Spanish) layout and settings.
#[unsafe(no_mangle)]
pub extern "C" fn wk_new() -> *mut WkKeyboard {
    Box::into_raw(Box::new(WkKeyboard {
        kb: Keyboard::default(),
        queue: VecDeque::new(),
        current: CString::default(),
    }))
}

/// # Safety
/// `kb` must come from `wk_new` and not be used afterwards.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wk_free(kb: *mut WkKeyboard) {
    if !kb.is_null() {
        drop(unsafe { Box::from_raw(kb) });
    }
}

/// Sets a `WK_OPT_*` option.
///
/// # Safety
/// `kb` must be a live keyboard from `wk_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wk_set_option(kb: *mut WkKeyboard, option: u32, value: i32) {
    let Some(w) = (unsafe { self::kb(kb) }) else {
        return;
    };
    let c = &mut w.kb.config;
    match option {
        0 => c.vowel_hand = side(value as u32),
        1 => {
            c.context = match value {
                1 => Context::Terminal,
                2 => Context::Finder,
                3 => Context::Browser,
                _ => Context::Text,
            }
        }
        2 => c.jis = value != 0,
        3 => c.prediction = value != 0,
        4 => c.border_correction = value != 0,
        5 => w.kb.set_secure_input(value != 0),
        6 => c.ring_margin = (value as f32 / 1000.0).clamp(0.0, 0.08),
        7 => {
            if let Some(l) = usize::try_from(value)
                .ok()
                .and_then(|i| Layout::all().into_iter().nth(i))
            {
                w.kb.set_layout(l);
            }
        }
        _ => {}
    }
}

/// # Safety
/// `kb` must be a live keyboard from `wk_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wk_touch_down(
    kb: *mut WkKeyboard,
    side: u32,
    x: f32,
    y: f32,
    radius: f32,
    now_ms: u64,
) {
    if let Some(w) = unsafe { self::kb(kb) } {
        w.kb.touch_down(self::side(side), x, y, radius, now_ms);
    }
}

/// Returns `WK_MOVE_*` bits: 1 the selection changed, 2 the thumb crossed rings.
///
/// # Safety
/// `kb` must be a live keyboard from `wk_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wk_touch_move(
    kb: *mut WkKeyboard,
    side: u32,
    x: f32,
    y: f32,
    now_ms: u64,
) -> u32 {
    let Some(w) = (unsafe { self::kb(kb) }) else {
        return 0;
    };
    let f = w.kb.touch_move(self::side(side), x, y, now_ms);
    f.changed as u32 | (f.ring_crossed as u32) << 1
}

/// Returns the number of queued actions.
///
/// # Safety
/// `kb` must be a live keyboard from `wk_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wk_touch_up(kb: *mut WkKeyboard, side: u32, now_ms: u64) -> usize {
    let Some(w) = (unsafe { self::kb(kb) }) else {
        return 0;
    };
    let a = w.kb.touch_up(self::side(side), now_ms);
    queue(w, a)
}

/// # Safety
/// `kb` must be a live keyboard from `wk_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wk_touch_cancel(kb: *mut WkKeyboard, side: u32) {
    if let Some(w) = unsafe { self::kb(kb) } {
        w.kb.touch_cancel(self::side(side));
    }
}

/// Call every frame while a thumb is down. Returns the number of queued actions.
///
/// # Safety
/// `kb` must be a live keyboard from `wk_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wk_tick(kb: *mut WkKeyboard, now_ms: u64) -> usize {
    let Some(w) = (unsafe { self::kb(kb) }) else {
        return 0;
    };
    let a = w.kb.tick(now_ms);
    queue(w, a)
}

/// Presses a `WK_BUTTON_*`. Returns the number of queued actions.
///
/// # Safety
/// `kb` must be a live keyboard from `wk_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wk_press(kb: *mut WkKeyboard, button: u32, now_ms: u64) -> usize {
    let Some(w) = (unsafe { self::kb(kb) }) else {
        return 0;
    };
    let b = match button {
        0 => Button::Shift,
        1 => Button::More,
        2 => Button::Enter,
        3 => Button::Kana,
        _ => return w.queue.len(),
    };
    let a = w.kb.press(b, now_ms);
    queue(w, a)
}

/// Pops the next queued action into `out`. Returns false when there are none.
///
/// # Safety
/// `kb` must be a live keyboard from `wk_new`; `out` must point to a `WkAction`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wk_next_action(kb: *mut WkKeyboard, out: *mut WkAction) -> bool {
    let Some(w) = (unsafe { self::kb(kb) }) else {
        return false;
    };
    let Some(out) = (unsafe { out.as_mut() }) else {
        return false;
    };
    let Some(a) = w.queue.pop_front() else {
        return false;
    };
    *out = WkAction {
        kind: 0,
        key: 0,
        codepoint: 0,
        mods: 0,
        count: 0,
        text: std::ptr::null(),
    };
    match a {
        Action::Text(s) => {
            w.current = CString::new(s.replace('\0', "")).unwrap_or_default();
            out.kind = 1;
            out.text = w.current.as_ptr();
        }
        Action::Backspace(n) => {
            out.kind = 2;
            out.count = n as u32;
        }
        Action::Key { key, mods } => {
            let (k, cp) = key_code(key);
            out.kind = 3;
            out.key = k;
            out.codepoint = cp;
            out.mods = mods.bits();
        }
        Action::Kana(on) => {
            out.kind = 4;
            out.count = on as u32;
        }
    }
    true
}

/// Tells the keyboard the text before the cursor (UTF-8).
///
/// # Safety
/// `kb` must be a live keyboard; `text` a NUL-terminated string or NULL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wk_set_text_before_cursor(kb: *mut WkKeyboard, text: *const c_char) {
    let Some(w) = (unsafe { self::kb(kb) }) else {
        return;
    };
    w.kb.set_text_before_cursor(unsafe { str_arg(text) }.unwrap_or(""));
}

/// Label of a key as it should be drawn now. `ring`: 0 inner, 1 outer, 2 centre.
///
/// # Safety
/// `kb` must be a live keyboard; `buf` must hold `cap` bytes (or be NULL).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wk_key_label(
    kb: *mut WkKeyboard,
    side: u32,
    ring: u32,
    dir: u32,
    buf: *mut c_char,
    cap: usize,
) -> usize {
    let Some(w) = (unsafe { self::kb(kb) }) else {
        return 0;
    };
    let r = w.kb.rings(self::side(side));
    let key = match ring {
        0 => r.inner[(dir % 8) as usize],
        1 => r.outer[(dir % 8) as usize],
        _ => r.center,
    };
    unsafe { fill(&w.kb.label(key), buf, cap) }
}

/// The selected ring (0 inner, 1 outer) and direction. Returns false if nothing is selected.
///
/// # Safety
/// `kb` must be a live keyboard; `ring` and `dir` valid pointers or NULL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wk_selection(
    kb: *mut WkKeyboard,
    side: u32,
    ring: *mut u32,
    dir: *mut u32,
) -> bool {
    let Some(w) = (unsafe { self::kb(kb) }) else {
        return false;
    };
    let Some(s) = w.kb.selection(self::side(side)) else {
        return false;
    };
    unsafe {
        if let Some(r) = ring.as_mut() {
            *r = (s.ring == RingId::Outer) as u32;
        }
        if let Some(d) = dir.as_mut() {
            *d = s.dir as u32;
        }
    }
    true
}

/// How strongly to highlight a key, 0 for not at all (otherwise its probability).
///
/// # Safety
/// `kb` must be a live keyboard.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wk_highlight(kb: *mut WkKeyboard, side: u32, ring: u32, dir: u32) -> f32 {
    let Some(w) = (unsafe { self::kb(kb) }) else {
        return 0.0;
    };
    let r = w.kb.rings(self::side(side));
    let key = if ring == 0 {
        r.inner[(dir % 8) as usize]
    } else {
        r.outer[(dir % 8) as usize]
    };
    w.kb.highlights()
        .into_iter()
        .find(|(k, _)| *k == key)
        .map_or(0.0, |(_, p)| p as f32)
}

/// What lifting the thumb would type. Returns 0 if the thumb is not down.
/// `cancel` is set to 1 when lifting would type nothing; `corrected` to 1 when
/// prediction moved the selection.
///
/// # Safety
/// `kb` must be a live keyboard; pointers valid or NULL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wk_preview(
    kb: *mut WkKeyboard,
    side: u32,
    now_ms: u64,
    buf: *mut c_char,
    cap: usize,
    cancel: *mut u8,
    corrected: *mut u8,
) -> usize {
    let Some(w) = (unsafe { self::kb(kb) }) else {
        return 0;
    };
    let Some(p) = w.kb.preview(self::side(side), now_ms) else {
        return 0;
    };
    unsafe {
        if let Some(c) = cancel.as_mut() {
            *c = p.cancel as u8;
        }
        if let Some(c) = corrected.as_mut() {
            *c = p.corrected_from.is_some() as u8;
        }
        fill(&p.label, buf, cap)
    }
}

/// # Safety
/// `kb` must be a live keyboard.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wk_suggestion_count(kb: *mut WkKeyboard) -> usize {
    unsafe { self::kb(kb) }.map_or(0, |w| w.kb.suggestions().len())
}

/// # Safety
/// `kb` must be a live keyboard; `buf` must hold `cap` bytes (or be NULL).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wk_suggestion(
    kb: *mut WkKeyboard,
    index: usize,
    buf: *mut c_char,
    cap: usize,
) -> usize {
    let Some(w) = (unsafe { self::kb(kb) }) else {
        return 0;
    };
    let s = w.kb.suggestions();
    s.get(index).map_or(0, |s| unsafe { fill(s, buf, cap) })
}

/// Types suggestion `index` and a space. Returns the number of queued actions.
///
/// # Safety
/// `kb` must be a live keyboard.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wk_pick_suggestion(kb: *mut WkKeyboard, index: usize) -> usize {
    let Some(w) = (unsafe { self::kb(kb) }) else {
        return 0;
    };
    let a = w.kb.pick_suggestion(index);
    queue(w, a)
}

/// 0 letters, 1 the ⋯ layer.
///
/// # Safety
/// `kb` must be a live keyboard.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wk_layer(kb: *mut WkKeyboard) -> u32 {
    unsafe { self::kb(kb) }.map_or(0, |w| (w.kb.layer() == Layer::More) as u32)
}

/// State of a `WK_MOD_*` modifier: 0 off, 1 next key only, 2 locked.
///
/// # Safety
/// `kb` must be a live keyboard.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wk_modifier_state(kb: *mut WkKeyboard, modifier: u32) -> u32 {
    let Some(w) = (unsafe { self::kb(kb) }) else {
        return 0;
    };
    match self::modifier(modifier).map(|m| w.kb.modifier(m)) {
        Some(ModState::Once) => 1,
        Some(ModState::Locked) => 2,
        _ => 0,
    }
}

/// `WK_MOD_*` bits of the modifiers in effect (including a chord).
///
/// # Safety
/// `kb` must be a live keyboard.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wk_active_modifiers(kb: *mut WkKeyboard) -> u8 {
    unsafe { self::kb(kb) }.map_or(0, |w| w.kb.active_modifiers().bits())
}

/// Saves what the keyboard learnt as UTF-8 text. Store it wherever the app keeps
/// private data; never send it anywhere.
///
/// # Safety
/// `kb` must be a live keyboard; `buf` must hold `cap` bytes (or be NULL).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wk_model_save(kb: *mut WkKeyboard, buf: *mut c_char, cap: usize) -> usize {
    let Some(w) = (unsafe { self::kb(kb) }) else {
        return 0;
    };
    unsafe { fill(&w.kb.model().to_text(), buf, cap) }
}

/// Loads a model saved with `wk_model_save`. Returns false if the text is not one.
///
/// # Safety
/// `kb` must be a live keyboard; `text` a NUL-terminated string or NULL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wk_model_load(kb: *mut WkKeyboard, text: *const c_char) -> bool {
    let Some(w) = (unsafe { self::kb(kb) }) else {
        return false;
    };
    match unsafe { str_arg(text) }.and_then(Model::from_text) {
        Some(m) => {
            w.kb.set_model(m);
            true
        }
        None => false,
    }
}

/// Forgets everything learnt.
///
/// # Safety
/// `kb` must be a live keyboard.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wk_model_clear(kb: *mut WkKeyboard) {
    if let Some(w) = unsafe { self::kb(kb) } {
        w.kb.clear_model();
    }
}

// Keep the modifier constants in step with the header.
const _: () = assert!(Modifiers::CTRL.bits() == 1 && Modifiers::CMD.bits() == 8);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn types_through_the_c_api() {
        unsafe {
            let kb = wk_new();
            wk_set_option(kb, 1, 1); // terminal: no automatic capitals
            wk_touch_down(kb, 0, 0.0, 0.0, 100.0, 0);
            assert_eq!(wk_touch_move(kb, 0, 0.0, -30.0, 10) & 1, 1);
            assert_eq!(wk_touch_up(kb, 0, 20), 1);
            let mut a = WkAction {
                kind: 0,
                key: 0,
                codepoint: 0,
                mods: 0,
                count: 0,
                text: std::ptr::null(),
            };
            assert!(wk_next_action(kb, &mut a));
            assert_eq!(a.kind, 1);
            assert_eq!(CStr::from_ptr(a.text).to_str().unwrap(), "s");
            assert!(!wk_next_action(kb, &mut a));

            let mut buf = [0 as c_char; 8];
            let n = wk_key_label(kb, 1, 0, 0, buf.as_mut_ptr(), buf.len());
            assert_eq!(n, 1);
            assert_eq!(CStr::from_ptr(buf.as_ptr()).to_str().unwrap(), "e");

            let need = wk_model_save(kb, std::ptr::null_mut(), 0);
            assert!(need > 0);
            wk_free(kb);
        }
    }
}
