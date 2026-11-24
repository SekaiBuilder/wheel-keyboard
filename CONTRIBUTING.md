# Contributing to Wheel Keyboard

Thanks for helping. A keyboard sees everything people type and this one learns
from it, so privacy is part of the design. Read the principles before sending
changes.

## Non-negotiable principles

1. **Nothing leaves the device.** Neither what is typed nor what is learnt
   travels over the network. The engine has no network code and no
   dependencies; the browser demo makes no requests besides its fonts. CI checks
   both.
2. **The engine stays platform-free.** `crates/wheel-keyboard` takes touches and
   returns actions. No drawing, no OS calls, no dependencies. Platform code
   goes in the FFI, the desktop app or the host app.
3. **No learning in password fields.** Any change to prediction must keep it.
4. **Never move the drawing.** Prediction highlights letters but never changes
   their size or position: muscle memory comes first.
5. **Border rule.** In the middle of a letter you always get that letter;
   correction only decides in the band next to a border.
6. **Keep the docs in step.** If behaviour changes, update `docs/SPEC.md`, the
   README and, when it touches the C API, `include/wheel_keyboard.h`.

## Development

```bash
cargo test --workspace                       # engine, C API and acceptance tests
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p wheel-keyboard-desktop          # desktop app
scripts/check.sh                             # privacy checks (also in CI)
```

The browser demo is `docs/index.html`: open it directly, or serve the folder
(`python3 -m http.server -d docs`) and open it on a phone on the same network.

On Linux the desktop app needs the usual windowing libraries
(`libxkbcommon-dev libwayland-dev libgl1-mesa-dev` on Debian/Ubuntu).

## Pull requests

- One logical change per commit, with a message that says what changes for the
  person typing.
- New behaviour comes with a test in `crates/wheel-keyboard/tests/` and a line
  in `docs/SPEC.md`.
- By contributing you agree that your contribution is licensed under the
  Apache License 2.0, like the rest of the project.
