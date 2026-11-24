# Privacidad y seguridad

Un teclado ve todo lo que escribes. Este documento dice qué hace el Teclado de
ruedas con eso, qué garantiza y qué no.

## En una línea

> Lo que escribes y lo que el teclado aprende se quedan en tu navegador. No se
> envía nada a ningún servidor.

## Garantías

| | Garantía |
|---|---|
| **INV-1** | El código no hace peticiones de red: ni `fetch`, ni `XMLHttpRequest`, ni `WebSocket`, ni `sendBeacon`, ni scripts externos. `scripts/check.sh` (también en la CI) falla si aparecen. |
| **INV-2** | Lo aprendido (frecuencias de letras y palabras) se guarda sólo en `localStorage` de ese navegador y en ese aparato. |
| **INV-3** | Con un campo de contraseña activo no aprende ni sugiere. |

## Lo que no garantiza

- **Fuentes de Google.** La página pide las tipografías a Google Fonts, así que
  Google ve que se abrió la página (no lo que se escribe). Sin conexión se usan
  las del sistema.
- **`localStorage` no está cifrado.** Quien tenga acceso a tu navegador puede
  leer las palabras aprendidas. Se borran desde Ajustes o borrando los datos
  del sitio.
- **Integrado en otra app**, como remote-mac, la privacidad del envío depende
  de esa app.

## Reportar una vulnerabilidad

Por favor, **no abras un issue público**. Usa un aviso privado en este
repositorio: Security → *Report a vulnerability*.
