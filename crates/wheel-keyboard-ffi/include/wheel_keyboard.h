/*
 * Wheel Keyboard — C API
 * Copyright 2026 Francesco Catania (@sekaibuilder)
 * Licensed under the Apache License, Version 2.0. Keep the NOTICE file.
 *
 * One opaque WkKeyboard per keyboard. Touch functions queue actions; read them
 * with wk_next_action() until it returns false. Strings are UTF-8. Functions
 * that fill a buffer return the length the full string needs (without the NUL)
 * and write a NUL-terminated string only when it fits.
 *
 * Coordinates: any unit (points, pixels), y pointing down; pass the wheel
 * radius in the same unit. Times: milliseconds from any monotonic clock.
 */
#ifndef WHEEL_KEYBOARD_H
#define WHEEL_KEYBOARD_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct WkKeyboard WkKeyboard;

/* Sides */
#define WK_LEFT  0u
#define WK_RIGHT 1u

/* Options for wk_set_option */
#define WK_OPT_VOWEL_HAND        0u /* WK_LEFT or WK_RIGHT (default right)        */
#define WK_OPT_CONTEXT           1u /* WK_CONTEXT_*                               */
#define WK_OPT_JIS               2u /* 1: JIS keyboard (あ/A button, ¥)          */
#define WK_OPT_PREDICTION        3u /* 1: learn, highlight, suggest (default 1)    */
#define WK_OPT_BORDER_CORRECTION 4u /* 1: fix taps on letter borders (default 1)   */
#define WK_OPT_SECURE_INPUT      5u /* 1: password field, learn nothing            */
#define WK_OPT_RING_MARGIN       6u /* ring hysteresis in thousandths of R (30)    */

#define WK_CONTEXT_TEXT     0
#define WK_CONTEXT_TERMINAL 1
#define WK_CONTEXT_FINDER   2
#define WK_CONTEXT_BROWSER  3

/* Buttons for wk_press */
#define WK_BUTTON_SHIFT 0u
#define WK_BUTTON_MORE  1u
#define WK_BUTTON_ENTER 2u
#define WK_BUTTON_KANA  3u

/* wk_touch_move result bits */
#define WK_MOVE_CHANGED      1u /* redraw                          */
#define WK_MOVE_RING_CROSSED 2u /* light haptic tick or flash      */

/* Action kinds */
#define WK_ACTION_TEXT      1u /* insert text                              */
#define WK_ACTION_BACKSPACE 2u /* delete `count` characters before cursor  */
#define WK_ACTION_KEY       3u /* press `key` with `mods`                  */
#define WK_ACTION_KANA      4u /* count 1: kana (かな), 0: alphanumeric (英数) */

/* Key codes */
#define WK_KEY_CHAR      0u /* see codepoint */
#define WK_KEY_SPACE     1u
#define WK_KEY_BACKSPACE 2u
#define WK_KEY_ENTER     3u
#define WK_KEY_TAB       4u
#define WK_KEY_ESCAPE    5u
#define WK_KEY_DELETE    6u /* forward delete */
#define WK_KEY_UP        7u
#define WK_KEY_DOWN      8u
#define WK_KEY_LEFT      9u
#define WK_KEY_RIGHT    10u

/* Modifier bits */
#define WK_MOD_CTRL  1u
#define WK_MOD_ALT   2u
#define WK_MOD_SHIFT 4u
#define WK_MOD_CMD   8u /* Command on macOS, Super/Windows elsewhere */

typedef struct WkAction {
    uint32_t kind;
    uint32_t key;
    uint32_t codepoint;
    uint8_t mods;
    uint32_t count;
    const char *text; /* valid until the next call on this keyboard */
} WkAction;

WkKeyboard *wk_new(void);
void wk_free(WkKeyboard *kb);
void wk_set_option(WkKeyboard *kb, uint32_t option, int32_t value);

void wk_touch_down(WkKeyboard *kb, uint32_t side, float x, float y, float radius, uint64_t now_ms);
uint32_t wk_touch_move(WkKeyboard *kb, uint32_t side, float x, float y, uint64_t now_ms);
size_t wk_touch_up(WkKeyboard *kb, uint32_t side, uint64_t now_ms);
void wk_touch_cancel(WkKeyboard *kb, uint32_t side);
size_t wk_tick(WkKeyboard *kb, uint64_t now_ms);
size_t wk_press(WkKeyboard *kb, uint32_t button, uint64_t now_ms);
bool wk_next_action(WkKeyboard *kb, WkAction *out);

void wk_set_text_before_cursor(WkKeyboard *kb, const char *text);

/* Drawing. ring: 0 inner, 1 outer, 2 centre. dir: 0 N, clockwise to 7 NW. */
size_t wk_key_label(WkKeyboard *kb, uint32_t side, uint32_t ring, uint32_t dir, char *buf, size_t cap);
bool wk_selection(WkKeyboard *kb, uint32_t side, uint32_t *ring, uint32_t *dir);
float wk_highlight(WkKeyboard *kb, uint32_t side, uint32_t ring, uint32_t dir);
size_t wk_preview(WkKeyboard *kb, uint32_t side, uint64_t now_ms, char *buf, size_t cap,
                  uint8_t *cancel, uint8_t *corrected);
uint32_t wk_layer(WkKeyboard *kb);
uint32_t wk_modifier_state(WkKeyboard *kb, uint32_t modifier);
uint8_t wk_active_modifiers(WkKeyboard *kb);

/* Suggestions */
size_t wk_suggestion_count(WkKeyboard *kb);
size_t wk_suggestion(WkKeyboard *kb, size_t index, char *buf, size_t cap);
size_t wk_pick_suggestion(WkKeyboard *kb, size_t index);

/* What it learnt. Keep it on the device. */
size_t wk_model_save(WkKeyboard *kb, char *buf, size_t cap);
bool wk_model_load(WkKeyboard *kb, const char *text);
void wk_model_clear(WkKeyboard *kb);

#ifdef __cplusplus
}
#endif

#endif /* WHEEL_KEYBOARD_H */
