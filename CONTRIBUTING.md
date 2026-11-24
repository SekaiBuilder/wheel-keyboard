# Contribuir al Teclado de ruedas

Gracias por querer mejorarlo. Es un teclado que aprende cómo escribe cada
persona, así que lo que se teclea es privado por definición. Lee los principios
antes de mandar cambios.

## Principios no negociables

1. **Nada sale del aparato.** Ni lo que se escribe ni lo aprendido viajan por
   la red. Lo aprendido vive sólo en `localStorage` de ese navegador. Sin
   `fetch`, `XMLHttpRequest`, `WebSocket` ni `sendBeacon`. La CI lo comprueba.
2. **Un solo archivo, sin dependencias.** `index.html` lleva todo el HTML, CSS y
   JavaScript. Nada de librerías ni pasos de compilación: se abre y funciona.
   La única carga externa son las fuentes de Google Fonts, y si fallan el
   teclado sigue funcionando con las del sistema.
3. **Con contraseña, ni aprende ni sugiere.** Cualquier cambio en la predicción
   debe respetarlo.
4. **No mover el dibujo.** La predicción resalta letras, pero nunca cambia el
   tamaño ni la posición de las letras: la memoria del dedo manda.
5. **Regla del borde.** En el centro de una letra se escribe siempre esa letra;
   la corrección sólo decide en la franja junto al borde.
6. **Docs al día.** Si cambias el comportamiento, actualiza `docs/SPEC.md` y,
   si se ve, el `README.md`.

## Probar los cambios

No hace falta instalar nada:

```bash
open index.html
```

Para probarlo de verdad, ábrelo en un móvil en horizontal (por ejemplo, con
`python3 -m http.server` y la IP del ordenador en la misma red). El modo
**Práctica** mide la velocidad y el reparto entre pulgares.

Antes de subir, la misma comprobación que la CI:

```bash
scripts/check.sh
```

## Estilo de commits

- En español, en una línea que diga qué cambia para quien escribe.
- Un cambio lógico por commit.
