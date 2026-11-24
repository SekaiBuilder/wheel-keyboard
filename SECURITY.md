# Privacy and security

A keyboard sees everything you type. This document says what Wheel Keyboard
does with it, what it guarantees and what it does not.

## In one line

> What you type and what the keyboard learns stay on your device. Nothing is
> sent to any server.

## Guarantees

| | Guarantee |
|---|---|
| **INV-1** | The engine (`crates/wheel-keyboard`) has no dependencies and no network code. The browser demo makes no requests (`fetch`, `XMLHttpRequest`, `WebSocket`, `sendBeacon`, external scripts). `scripts/check.sh`, run in CI, fails if that changes. |
| **INV-2** | What is learnt (letter, word and word-pair counts) is held in memory and saved only where the host app decides: a text file in the user's data folder for the desktop app, `localStorage` for the browser demo. |
| **INV-3** | With secure input on (a password field), nothing is learnt and nothing is suggested. |

## What it does not guarantee

- **Hosts decide where the model is stored.** If an app that embeds the engine
  uploads the saved model, that is the app's doing. Apps should keep it in
  private storage and never sync it without consent.
- **The saved model is not encrypted.** Anyone with access to the user's files
  or browser can read the learnt words. "Forget what it learnt" (desktop) and
  "Clear learnt data" (browser) erase it.
- **Google Fonts in the browser demo.** The demo page loads its typefaces from
  Google Fonts, so Google sees that the page was opened (not what is typed).
  Offline, system fonts are used.
- **Secure input depends on the host.** The engine can only skip learning if the
  host tells it the field is a password field.

## Reporting a vulnerability

Please **don't open a public issue**. Use a private advisory instead:
Security → *Report a vulnerability* in this repository.
