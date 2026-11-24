/* Minimal C host: one swipe up with the left thumb types "s". */
#include <stdio.h>
#include "wheel_keyboard.h"

int main(void) {
    WkKeyboard *kb = wk_new();
    wk_set_option(kb, WK_OPT_CONTEXT, WK_CONTEXT_TERMINAL);
    wk_touch_down(kb, WK_LEFT, 0, 0, 100, 0);
    wk_touch_move(kb, WK_LEFT, 0, -30, 16);
    wk_touch_up(kb, WK_LEFT, 32);
    WkAction a;
    while (wk_next_action(kb, &a)) {
        if (a.kind == WK_ACTION_TEXT) printf("%s\n", a.text);
    }
    wk_free(kb);
    return 0;
}
