# Wheel Keyboard — specification

Status: v3.1, engine in Rust. Last updated 2026-11-24.

The Rust engine (`crates/wheel-keyboard`) is the reference implementation.
The browser demo (`docs/index.html`) is a test bench that follows the same
rules. Each acceptance criterion names the test that checks it
(`crates/wheel-keyboard/tests/acceptance.rs` unless stated).

## Purpose

Type with two thumbs on a touchscreen faster and with fewer errors than with
an on-screen QWERTY keyboard, shortcuts included (⌘C, ⌘V, ⌘Tab…). It was
first built to type on a Mac from an iPhone (remote-mac) and is now a
standalone engine any app can embed.

## Design

### Wheels

- Eight directions, numbered clockwise from the top: 0 N, 1 NE, 2 E, 3 SE,
  4 S, 5 SW, 6 W, 7 NW; 45° sectors. The wheel is centred where the thumb
  lands and selection is measured from that point.
- Rings, as a fraction of the radius R: dead zone 0–0.15, inner 0.15–0.50,
  outer from 0.50.
- Hysteresis: once in a ring, the thumb has to pass the border by the ring
  margin (default 0.03 R, range 0–0.08) to switch; a direction keeps 3° past
  its sector edge.
- Built-in layout (`Layout::spanish()`):
  - Consonants, inner: N S · NE R · E N · SE P · S G · SW B · W T · NW C;
    outer: V Q H J W X Z F.
  - Vowels, inner: N E · NE I · E D · SE U · S M · SW L · W O · NW A;
    outer: Y Ñ ? K - ¿ . ,
  - The 16 inner letters cover about 95 % of letters written in Spanish.
- The vowel hand is a setting (right by default). With vowels on the left the
  wheels swap and mirror (E ↔ W).
- Centre: a tap without leaving the dead zone types Space on the consonant
  wheel and Backspace on the vowel wheel (holding repeats after 450 ms, every
  75 ms).
- Leaving the dead zone and coming back before lifting types nothing.
- Crossing between rings is reported to the host for a light haptic tick or a
  flash.

### Buttons and the ⋯ layer

- Above the wheels: ⇧ and ⋯ on the left, ⏎ (and あ/A on JIS keyboards) on the
  right.
- ⋯ layer, left: ⌘ ⌥ ⌃ Esc ⏎ ⌦ ⇥ ⇧ inner; 1–5 ( ) = outer. Right: arrows on
  N E S W plus context symbols inner; 6–0 and symbols outer (¥ instead of ; on
  JIS).
- Modifiers: one tap applies to the next key; two taps within 400 ms lock it;
  choosing one returns the wheel to letters.
- Chord: resting the left thumb on a modifier for 160 ms holds it while the
  right thumb types, and the right wheel shows shortcuts for the context.

### Context

Text, Terminal, Finder or Browser. It changes automatic capitals (Text only:
at the start, after ". ", "! ", "? " and after a new line), the ⋯ symbols
and the chord shortcuts.

### Accents

Holding a letter still for 450 ms cycles through its variants (A → á,
U → ú → ü), shown in the preview before lifting.

### Prediction (starts empty)

- Learns when a word ends (space, punctuation, Enter or a suggestion), from
  the word as it ended up: counts of each letter after the previous 0–3
  letters of the word (accents folded, ñ apart), word counts and word-pair
  counts.
- Letter probability: recursive Dirichlet smoothing, α = 1.5, from uniform up
  to the three-letter context.
- Highlights up to 3 letters with p ≥ 0.12 once the one-letter context has
  at least 8 cases. Highlighting never changes size or position.
- **Border rule:** only when the angle is within 8° of a sector border and the
  neighbour in the same ring is a letter. It switches to the neighbour if
  `P(neighbour) ≥ c · P(touched)` and
  `gauss(Δneighbour) · P(neighbour) > gauss(Δtouched) · P(touched)`, with
  σ = 12°, c = 3 + 0.5 per undone correction (at most 13), and at least 60
  letters learnt. The preview shows "corrected" before lifting.
- Backspace right after a corrected letter puts back the letter touched and
  raises c.
- Three suggestions: completions of the current prefix (score = uses + 4 ×
  uses after the previous word) or, with no prefix, words that follow the
  previous one. Picking one types it and a space.
- At most 5000 words; the 500 least used are dropped when it overflows.
- Saved as plain text (`Model::to_text`); where it is stored is up to the
  host. It can be erased.
- With secure input on (password field) nothing is learnt or suggested.

## Requirements

| ID | Requirement |
|---|---|
| R-01 | Two wheels, one per half of the touch area, opening where each thumb lands; both thumbs at once |
| R-03 | While the thumb moves, the selected sector lights up and a preview shows what lifting will type; returning to the centre cancels |
| R-04 | Centre tap: Space on the consonant wheel, Backspace on the vowel wheel (hold repeats) |
| R-05 | ⋯ layer with one-shot modifiers, locked with a double tap, and a chord by resting the thumb; after choosing a modifier the wheel returns to letters |
| R-06 | With a modifier active the key comes out as a shortcut, not as a letter |
| R-07 | Context Text, Terminal, Finder or Browser: automatic capitals, symbols and chord shortcuts |
| R-08 | A vowel held 450 ms comes out accented |
| R-15 | Each wheel has 8 directions and two rings; 16 frequent letters inner, vowels on one hand and consonants on the other, hand selectable |
| R-16 | Hysteresis between rings and between directions; a cue when changing rings |
| R-17 | ⇧ ⋯ ⏎ buttons above the wheels |
| R-18 | With a JIS keyboard, あ/A (かな/英数) button and ¥ in the ⋯ layer |
| R-19 | Prediction starts empty and learns when words end; highlights up to 3 likely letters without changing the drawing |
| R-20 | Correction only within the 8° border band, with a minimum confidence; visible before lifting; an immediate Backspace undoes it and makes it more cautious |
| R-21 | Three suggested words; picking one completes it and adds a space |
| R-22 | What is learnt never leaves the device, has a cap, can be erased, and nothing is learnt in a password field |
| R-23 | The engine has no dependencies and no platform code, and is usable from C (`wheel_keyboard.h`) |
| R-24 | A desktop app for macOS, Windows and Linux, with touch and mouse input |

## Acceptance

| ID | Given / when / then | Test |
|---|---|---|
| A-03 | Leaving the centre and coming back before lifting types nothing | `a03_back_to_centre_cancels` |
| A-04 | ⋯, ⌘, the layer closes, C → ⌘C as a shortcut, ⌘ turns off | `a04_modifier_then_letter_is_a_shortcut` |
| A-05 | Resting on ⌘ with the left thumb and tapping V twice with the right → ⌘V ⌘V; lifting the left types nothing | `a05_chord_holds_the_modifier_while_the_thumb_rests` |
| A-11 | Consonants: tap → space, N → s, NE outer → q; vowels: N → e, NW → a, tap → Backspace | `a11_consonants_and_vowels` |
| A-11b | Moving from the inner to the outer ring reports one ring crossing | `a11_ring_crossing_is_reported` |
| A-12 | With vowels on the left, left N → e and left NE → a (mirrored) | `a12_vowels_on_the_left_are_mirrored` |
| A-13 | After learning "queso", "qu" highlights E; a touch 20° from I's centre towards E types e ("corrected"); I's centre types i | `a13_highlights_and_border_correction` |
| A-14 | Backspace right after a correction leaves the touched letter and raises caution | `a14_backspace_after_correction_restores_the_tapped_letter` |
| A-15 | After "hola mundo", "ho" suggests "hola"; picking it leaves "hola " and then suggests "mundo" | `a15_suggestions_complete_words` |
| A-16 | With secure input nothing is learnt or suggested | `a16_password_fields_learn_nothing` |
| A-17 | JIS: ¥ in the ⋯ layer and あ/A toggles kana; not with ISO | `a17_jis_shows_yen_and_kana` |
| A-18 | Capitals at the start and after ". " in Text context | `auto_capital_at_start_and_after_full_stop` |
| A-19 | A double tap locks Shift | `double_tap_locks_a_modifier` |
| A-20 | Holding E 450 ms types é | `r08_holding_a_vowel_adds_the_accent` |
| A-21 | Holding Backspace repeats | `holding_backspace_repeats` |
| A-22 | The model survives saving and loading; at most 5000 words | `model_survives_save_and_load`, `model::tests` |
| A-23 | Ring and direction hysteresis values | `geometry::tests` |
| A-24 | A C program links the static library and types "s" | CI step "C example" (`examples/c/main.c`) and `wheel-keyboard-ffi` unit test |
| A-25 | Desktop app: a mouse swipe up on the left wheel types S, on the right E, a tap types a space | Manual, 2026-11-24, macOS (synthetic mouse events): typed "Se q" |
| A-07 | Typing a sentence on a phone in landscape with no zoom or accidental selection | Manual, on device (browser demo) |

## Open decisions

- D-02: How a host should tell the engine the context (front app) and secure
  input. Today the host sets `config.context` and `set_secure_input`.
- D-03: Offer a personalised layout built from what was learnt?
- D-05: Built-in layouts for other languages (English, Italian, Portuguese…).

## Not yet verified

- No real-thumb measurements: the 47/48 % load split is an estimate from
  Spanish letter frequencies.
- The desktop app has not been tried on a real touchscreen, nor on Windows or
  Linux (only built for them in CI).
- The model has been tested with injected words, not after days of use.
