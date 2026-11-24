#!/usr/bin/env bash
# Checks the privacy guarantees in SECURITY.md.
set -euo pipefail
cd "$(dirname "$0")/.."
fail=0
demo=docs/index.html

# INV-1 (demo): no network requests and no external scripts.
if grep -nE '\bfetch\(|XMLHttpRequest|WebSocket|sendBeacon|EventSource|<script[^>]+src=' "$demo"; then
  echo "✗ $demo uses the network or loads external scripts (INV-1)"; fail=1
fi
# The only external resource allowed is Google Fonts.
external=$(grep -oE '(href|src)="https?://[^"]+"' "$demo" | grep -vE 'fonts\.(googleapis|gstatic)\.com' || true)
if [ -n "$external" ]; then
  echo "$external"; echo "✗ external resource not allowed in $demo"; fail=1
fi

# INV-1 (engine): no dependencies and no network code.
if awk '/^\[dependencies\]/{f=1;next} /^\[/{f=0} f && NF && $0 !~ /^#/' crates/wheel-keyboard/Cargo.toml | grep -q .; then
  echo "✗ crates/wheel-keyboard must have no dependencies (INV-1)"; fail=1
fi
if grep -rnE 'std::net|TcpStream|UdpSocket|std::process' crates/wheel-keyboard/src; then
  echo "✗ crates/wheel-keyboard must not use the network or spawn processes (INV-1)"; fail=1
fi

for f in LICENSE NOTICE README.md docs/SPEC.md crates/wheel-keyboard-ffi/include/wheel_keyboard.h; do
  [ -f "$f" ] || { echo "✗ missing $f"; fail=1; }
done

[ "$fail" -eq 0 ] && echo "✓ checks passed"
exit "$fail"
