//! Acceptance tests from docs/SPEC.md, driven with synthetic touches.

use wheel_keyboard::*;

const R: f32 = 100.0;

/// Touch at the origin, slide to `angle` degrees (0 = up, clockwise) at `dist`
/// times the radius, lift.
fn swipe(kb: &mut Keyboard, side: Side, angle: f32, dist: f32) -> Vec<Action> {
    let a = angle.to_radians();
    kb.touch_down(side, 0.0, 0.0, R, 0);
    kb.touch_move(
        side,
        dist * R * a.sin() * 0.5,
        -dist * R * a.cos() * 0.5,
        10,
    );
    kb.touch_move(side, dist * R * a.sin(), -dist * R * a.cos(), 20);
    kb.touch_up(side, 30)
}

fn tap(kb: &mut Keyboard, side: Side) -> Vec<Action> {
    kb.touch_down(side, 0.0, 0.0, R, 0);
    kb.touch_up(side, 30)
}

fn text(actions: &[Action]) -> String {
    actions
        .iter()
        .map(|a| match a {
            Action::Text(s) => s.clone(),
            other => format!("{other:?}"),
        })
        .collect()
}

fn terminal() -> Keyboard {
    // Terminal context: no automatic capitals, which keeps expectations simple.
    Keyboard::new(
        Config {
            context: Context::Terminal,
            ..Config::default()
        },
        Layout::spanish(),
    )
}

#[test]
fn a11_consonants_and_vowels() {
    let mut kb = terminal();
    assert_eq!(text(&tap(&mut kb, Side::Left)), " ");
    assert_eq!(text(&swipe(&mut kb, Side::Left, 0.0, 0.3)), "s");
    assert_eq!(text(&swipe(&mut kb, Side::Left, 45.0, 0.8)), "q");
    assert_eq!(text(&swipe(&mut kb, Side::Right, 0.0, 0.3)), "e");
    assert_eq!(text(&swipe(&mut kb, Side::Right, 315.0, 0.3)), "a");
    assert_eq!(tap(&mut kb, Side::Right), vec![Action::Backspace(1)]);
}

#[test]
fn a11_ring_crossing_is_reported() {
    let mut kb = terminal();
    kb.touch_down(Side::Left, 0.0, 0.0, R, 0);
    let f1 = kb.touch_move(Side::Left, 0.0, -30.0, 10);
    assert!(f1.changed && !f1.ring_crossed);
    let f2 = kb.touch_move(Side::Left, 0.0, -80.0, 20);
    assert!(f2.ring_crossed);
    assert_eq!(text(&kb.touch_up(Side::Left, 30)), "v");
}

#[test]
fn a12_vowels_on_the_left_are_mirrored() {
    let mut kb = Keyboard::new(
        Config {
            vowel_hand: Side::Left,
            context: Context::Terminal,
            ..Config::default()
        },
        Layout::spanish(),
    );
    assert_eq!(text(&swipe(&mut kb, Side::Left, 0.0, 0.3)), "e");
    assert_eq!(text(&swipe(&mut kb, Side::Left, 45.0, 0.3)), "a");
    assert_eq!(text(&tap(&mut kb, Side::Right)), " ");
}

#[test]
fn a03_back_to_centre_cancels() {
    let mut kb = terminal();
    kb.touch_down(Side::Left, 0.0, 0.0, R, 0);
    kb.touch_move(Side::Left, 0.0, -30.0, 10);
    kb.touch_move(Side::Left, 0.0, -2.0, 20);
    assert!(kb.preview(Side::Left, 20).unwrap().cancel);
    assert!(kb.touch_up(Side::Left, 30).is_empty());
}

#[test]
fn auto_capital_at_start_and_after_full_stop() {
    let mut kb = Keyboard::default();
    assert_eq!(text(&swipe(&mut kb, Side::Left, 0.0, 0.3)), "S");
    assert_eq!(text(&swipe(&mut kb, Side::Right, 0.0, 0.3)), "e");
    assert_eq!(text(&swipe(&mut kb, Side::Right, 270.0, 0.8)), ".");
    tap(&mut kb, Side::Left);
    assert_eq!(text(&swipe(&mut kb, Side::Left, 0.0, 0.3)), "S");
}

fn learn_queso(kb: &mut Keyboard) {
    let mut m = Model::new();
    for _ in 0..16 {
        m.learn_word("queso", None);
    }
    kb.set_model(m);
    kb.set_text_before_cursor("qu");
}

#[test]
fn a13_highlights_and_border_correction() {
    let mut kb = terminal();
    learn_queso(&mut kb);
    assert_eq!(kb.highlights().first().map(|h| h.0), Some(Key::Char('E')));
    // 25° is inside I's sector (centred at 45°) but within 8° of E's border.
    kb.touch_down(Side::Right, 0.0, 0.0, R, 0);
    let a = 25f32.to_radians();
    kb.touch_move(Side::Right, 30.0 * a.sin(), -30.0 * a.cos(), 10);
    let p = kb.preview(Side::Right, 10).unwrap();
    assert_eq!(p.label, "e");
    assert_eq!(p.corrected_from, Some(Key::Char('I')));
    assert_eq!(text(&kb.touch_up(Side::Right, 20)), "e");
    // The centre of I always types i.
    kb.set_text_before_cursor("qu");
    assert_eq!(text(&swipe(&mut kb, Side::Right, 45.0, 0.3)), "i");
}

#[test]
fn a14_backspace_after_correction_restores_the_tapped_letter() {
    let mut kb = terminal();
    learn_queso(&mut kb);
    let out = swipe(&mut kb, Side::Right, 25.0, 0.3);
    assert_eq!(text(&out), "e");
    let undo = tap(&mut kb, Side::Right);
    assert_eq!(undo, vec![Action::Backspace(1), Action::Text("i".into())]);
    assert_eq!(kb.text_before_cursor(), "qui");
    assert_eq!(kb.model().rejections, 1);
}

#[test]
fn a15_suggestions_complete_words() {
    let mut kb = terminal();
    let mut m = Model::new();
    m.learn_word("hola", None);
    m.learn_word("mundo", Some("hola"));
    kb.set_model(m);
    kb.set_text_before_cursor("hola mundo ho");
    assert_eq!(kb.suggestions(), vec!["hola"]);
    assert_eq!(
        kb.pick_suggestion(0),
        vec![Action::Backspace(2), Action::Text("hola ".into())]
    );
    assert_eq!(kb.text_before_cursor(), "hola mundo hola ");
    assert_eq!(kb.suggestions(), vec!["mundo"]);
}

#[test]
fn a16_password_fields_learn_nothing() {
    let mut kb = terminal();
    kb.set_secure_input(true);
    for _ in 0..3 {
        swipe(&mut kb, Side::Left, 0.0, 0.3);
        swipe(&mut kb, Side::Right, 0.0, 0.3);
        tap(&mut kb, Side::Left);
    }
    assert_eq!(kb.model().letters, 0);
    assert!(kb.suggestions().is_empty());
}

#[test]
fn words_are_learnt_when_finished() {
    let mut kb = terminal();
    swipe(&mut kb, Side::Left, 0.0, 0.3); // s
    swipe(&mut kb, Side::Right, 0.0, 0.3); // e
    tap(&mut kb, Side::Left); // space
    assert_eq!(kb.model().letters, 2);
    assert_eq!(kb.model().word_count(), 1);
}

#[test]
fn a04_modifier_then_letter_is_a_shortcut() {
    let mut kb = terminal();
    kb.press(Button::More, 0);
    assert_eq!(kb.rings(Side::Left).inner[0], Key::Modifier(Modifier::Cmd));
    assert!(swipe(&mut kb, Side::Left, 0.0, 0.3).is_empty());
    assert_eq!(kb.layer(), Layer::Letters);
    assert_eq!(kb.modifier(Modifier::Cmd), ModState::Once);
    // C is at NW of the consonant ring.
    assert_eq!(
        swipe(&mut kb, Side::Left, 315.0, 0.3),
        vec![Action::Key {
            key: Key::Char('C'),
            mods: Modifiers::CMD
        }]
    );
    assert_eq!(kb.modifier(Modifier::Cmd), ModState::Off);
}

#[test]
fn double_tap_locks_a_modifier() {
    let mut kb = terminal();
    kb.press(Button::Shift, 0);
    kb.press(Button::Shift, 200);
    assert_eq!(kb.modifier(Modifier::Shift), ModState::Locked);
    assert_eq!(text(&swipe(&mut kb, Side::Left, 0.0, 0.3)), "S");
    assert_eq!(text(&swipe(&mut kb, Side::Left, 0.0, 0.3)), "S");
}

#[test]
fn a05_chord_holds_the_modifier_while_the_thumb_rests() {
    let mut kb = terminal();
    kb.press(Button::More, 0);
    kb.touch_down(Side::Left, 0.0, 0.0, R, 0);
    kb.touch_move(Side::Left, 0.0, -30.0, 10); // ⌘
    kb.tick(200);
    assert_eq!(kb.active_modifiers(), Modifiers::CMD);
    // Right wheel now offers ⌘ shortcuts: C at N, V at NE.
    let a = 45f32.to_radians();
    for t in [300, 400] {
        kb.touch_down(Side::Right, 0.0, 0.0, R, t);
        kb.touch_move(Side::Right, 30.0 * a.sin(), -30.0 * a.cos(), t + 10);
        assert_eq!(
            kb.touch_up(Side::Right, t + 20),
            vec![Action::Key {
                key: Key::Char('V'),
                mods: Modifiers::CMD
            }]
        );
    }
    assert!(kb.touch_up(Side::Left, 500).is_empty());
    assert_eq!(kb.active_modifiers(), Modifiers::NONE);
}

#[test]
fn r08_holding_a_vowel_adds_the_accent() {
    let mut kb = terminal();
    kb.touch_down(Side::Right, 0.0, 0.0, R, 0);
    kb.touch_move(Side::Right, 0.0, -30.0, 10); // E
    assert_eq!(kb.preview(Side::Right, 500).unwrap().label, "é");
    assert_eq!(text(&kb.touch_up(Side::Right, 500)), "é");
}

#[test]
fn holding_backspace_repeats() {
    let mut kb = terminal();
    kb.set_text_before_cursor("abcdef");
    kb.touch_down(Side::Right, 0.0, 0.0, R, 0);
    assert!(kb.tick(400).is_empty());
    assert_eq!(kb.tick(600).len(), 3); // 450, 525, 600
    assert!(kb.touch_up(Side::Right, 610).is_empty());
    assert_eq!(kb.text_before_cursor(), "abc");
}

#[test]
fn a17_jis_shows_yen_and_kana() {
    let mut kb = Keyboard::new(
        Config {
            jis: true,
            ..Config::default()
        },
        Layout::spanish(),
    );
    kb.press(Button::More, 0);
    assert!(kb.rings(Side::Right).outer.contains(&Key::Char('¥')));
    assert_eq!(kb.press(Button::Kana, 0), vec![Action::Kana(true)]);
    let mut iso = Keyboard::default();
    iso.press(Button::More, 0);
    assert!(!iso.rings(Side::Right).outer.contains(&Key::Char('¥')));
}

#[test]
fn model_survives_save_and_load() {
    let mut kb = terminal();
    learn_queso(&mut kb);
    let saved = kb.model().to_text();
    let mut other = terminal();
    other.set_model(Model::from_text(&saved).unwrap());
    assert_eq!(other.model(), kb.model());
}

fn with_layout(code: &str, context: Context) -> Keyboard {
    Keyboard::new(
        Config {
            context,
            ..Config::default()
        },
        Layout::by_code(code).unwrap(),
    )
}

#[test]
fn a26_english_types_the() {
    let mut kb = with_layout("en", Context::Text);
    let mut out = String::new();
    out += &text(&swipe(&mut kb, Side::Left, 270.0, 0.3)); // T
    out += &text(&swipe(&mut kb, Side::Left, 135.0, 0.3)); // H
    out += &text(&swipe(&mut kb, Side::Right, 0.0, 0.3)); // E
    assert_eq!(out, "The");
    assert_eq!(text(&swipe(&mut kb, Side::Right, 45.0, 0.8)), "'");
}

#[test]
fn a27_italian_apostrophe_and_grave_accent() {
    let mut kb = with_layout("it", Context::Terminal);
    assert_eq!(text(&swipe(&mut kb, Side::Right, 225.0, 0.3)), "l");
    assert_eq!(text(&swipe(&mut kb, Side::Right, 45.0, 0.8)), "'");
    // Holding E: first è, then é.
    kb.touch_down(Side::Right, 0.0, 0.0, R, 0);
    kb.touch_move(Side::Right, 0.0, -30.0, 10);
    assert_eq!(kb.preview(Side::Right, 500).unwrap().label, "è");
    assert_eq!(kb.preview(Side::Right, 950).unwrap().label, "é");
    assert_eq!(text(&kb.touch_up(Side::Right, 500)), "è");
    assert_eq!(kb.text_before_cursor(), "l'è");
}

#[test]
fn a28_japanese_romaji_has_no_automatic_capitals() {
    let mut kb = with_layout("ja", Context::Text);
    let strokes = [
        (Side::Right, 315.0), // a
        (Side::Right, 225.0), // r
        (Side::Right, 45.0),  // i
        (Side::Left, 180.0),  // g
        (Side::Right, 315.0), // a
        (Side::Left, 270.0),  // t
        (Side::Right, 270.0), // o
        (Side::Right, 135.0), // u
    ];
    let typed: String = strokes
        .iter()
        .map(|(s, a)| text(&swipe(&mut kb, *s, *a, 0.3)))
        .collect();
    assert_eq!(typed, "arigatou");
    assert_eq!(text(&swipe(&mut kb, Side::Right, 0.0, 0.8)), "-");
}
