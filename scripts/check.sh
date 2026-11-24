#!/usr/bin/env bash
# Comprueba las garantías de SECURITY.md y que la página está entera.
set -euo pipefail
cd "$(dirname "$0")/.."
fallo=0

# INV-1: sin peticiones de red ni scripts externos.
if grep -nE '\bfetch\(|XMLHttpRequest|WebSocket|sendBeacon|EventSource|<script[^>]+src=' index.html; then
  echo "✗ index.html usa la red o carga scripts externos (INV-1)"; fallo=1
fi

# La única carga externa permitida son las fuentes de Google.
externas=$(grep -oE '(href|src)="https?://[^"]+"' index.html | grep -vE 'fonts\.(googleapis|gstatic)\.com' || true)
if [ -n "$externas" ]; then
  echo "$externas"; echo "✗ carga externa no permitida en index.html"; fallo=1
fi

# Estructura mínima.
for f in index.html LICENSE README.md docs/SPEC.md; do
  [ -f "$f" ] || { echo "✗ falta $f"; fallo=1; }
done
grep -q '</html>\|</script>' index.html || { echo "✗ index.html parece cortado"; fallo=1; }

[ "$fallo" -eq 0 ] && echo "✓ comprobaciones superadas"
exit "$fallo"
