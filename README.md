# Teclado de ruedas

Un teclado para escribir con los dos pulgares en el móvil o la tableta. Cada
pulgar tiene una rueda de ocho direcciones: tocas, deslizas hacia la letra y
sueltas. Aprende cómo escribes, resalta la letra que seguramente viene y
corrige los toques que caen justo en el borde entre dos letras.

Nació para [remote-mac](#contexto) (manejar el Mac desde el iPhone), pero el
teclado es independiente y cualquiera puede usarlo.

*English: a two-thumb, eight-direction wheel keyboard for touchscreens, with an
on-device model that learns as you type, highlights likely letters, corrects
taps on the border between two letters and suggests words. Plain HTML, CSS and
JavaScript, no dependencies. MIT licensed: credit Francesco Catania.*

## Probarlo

Abre `index.html` en el navegador del móvil, en horizontal. No necesita
instalar nada ni conexión: es un solo archivo de HTML, CSS y JavaScript, sin
librerías. Lo aprendido se guarda sólo en ese navegador.

## Cómo funciona

- **Dos ruedas, una por pulgar.** La rueda aparece donde cae el dedo. Tocar sin
  mover escribe un espacio (rueda de las consonantes) o borra (rueda de las
  vocales; si lo mantienes, sigue borrando).
- **Ocho direcciones, dos anillos.** En el anillo de dentro están las 16 letras
  más frecuentes del español (cerca del 95 % de lo que se escribe); en el de
  fuera, las raras y los signos. Ocho es el máximo de direcciones que se
  aciertan sin mirar.
- **Vocales en un pulgar, consonantes en el otro.** En español casi todas las
  sílabas alternan consonante y vocal, así que los pulgares se turnan solos.
  La mano de las vocales se elige en Ajustes.
- **El espacio va con las consonantes** para equilibrar la carga: unos 47 % y
  48 % de las pulsaciones por pulgar (estimación; el modo práctica lo mide).
- **Paso entre anillos corto, con aviso.** Al pasar del anillo de dentro al de
  fuera hay una vibración (Android) o un destello (iPhone, donde Safari no deja
  vibrar mientras el dedo se mueve). No hace falta llegar hasta la letra.
- **Capa ⋯** con ⌘ ⌥ ⌃ ⇧ Esc ⇥, flechas, números y signos según el contexto
  (texto o terminal). Un modificador vale para la siguiente tecla, con doble
  toque queda fijo, y si dejas el pulgar sobre ⌘ la otra rueda enseña los
  atajos con su nombre (C Copiar, V Pegar…).
- **Teclado JIS:** con un Mac de teclado japonés aparecen あ/A (かな y 英数) y ¥.
- **Tildes:** deja el dedo quieto sobre una vocal.

### Predicción que aprende

- Empieza en blanco. Al terminar cada palabra cuenta qué letra viene después de
  las 0 a 3 anteriores, qué palabras usas y cuál suele seguir a cuál.
- Resalta hasta tres letras probables, sin cambiar el tamaño del dibujo, para
  no estropear la memoria del dedo.
- **Regla del borde:** si el dedo cae en el centro de una letra, se escribe esa
  letra siempre. Sólo en la franja de 8° junto al borde decide la predicción, y
  sólo si la vecina es bastante más probable. Se ve antes de soltar
  («corregido»). Si se equivoca, borrar enseguida devuelve la letra que tocaste
  y la vuelve más prudente.
- Barra de tres palabras sugeridas.
- Nada sale del aparato. Con un campo de contraseña activo no aprende ni sugiere.

Los detalles, números y requisitos están en [`docs/SPEC.md`](docs/SPEC.md).

## Cómo se decidió

| Versión | Qué era | Veredicto |
|---|---|---|
| v1 | Anillos ordenados por frecuencia de letras | Retirada: «mal distribuido», se quería algo que recordara al teclado de siempre |
| v2 | Abanico QWERTY de tres filas (números arriba), con histéresis entre filas | Retirada: tres filas resultaron complicadas |
| v3 | Ocho direcciones, vocales y consonantes en pulgares distintos, predicción | Elegida entre tres propuestas (ocho direcciones, teclas con deslizamiento, margarita a dos manos) |
| v3.1 | Círculo siempre entero, botones encima de cada pulgar, paso entre anillos más corto y suave | Actual |

Todas las pruebas hechas hasta ahora son con toques simulados en el navegador;
la tabla de evidencia y lo que falta por probar están en la spec. La página
[`docs/referencias.html`](docs/referencias.html) compara los proyectos de abajo
y las tres propuestas de la v3.

## Fuentes

Diseño de teclados y entrada por gestos:
- Kurtenbach y Buxton, *marking menus* (1993–94): elegir por dirección es más
  rápido que con menús; hasta 8 opciones por nivel con pocos errores.
- Teclado *flick* japonés (kana): tocar y deslizar en cruz.
- [MessagEase](https://www.exideas.com/ME/) y
  [Thumb-Key](https://github.com/dessalines/thumb-key): letras frecuentes en el
  gesto más corto, disposición fija.
- [KALQ](https://keithv.com/pub/twothumb/) — Oulasvirta et al., «Improving
  Two-Thumb Text Entry on Touchscreen Devices», CHI 2013: teclado partido,
  vocales en un pulgar, 37 ppm con práctica y corrección probabilística.
- [8VIM](https://github.com/8VIM/8VIM): círculo con gestos encadenados (lo que
  conviene evitar: cuesta aprenderlo).
- [Unexpected Keyboard](https://github.com/Julow/Unexpected-Keyboard):
  modificadores y símbolos de terminal a un deslizamiento.
- Teclado de mando de Steam (Big Picture), la «margarita»:
  [artículo en Kotaku](https://kotaku.com/valve-thinks-its-cracked-typing-with-a-controller-1709175825).

Predicción y corrección:
- [The Machine Intelligence Behind Gboard](https://research.google/blog/the-machine-intelligence-behind-gboard/)
  (Google Research) y [Spatial model personalization in Gboard](https://research.google/pubs/spatial-model-personalization-in-gboard/)
  (Sivek y Riley, 2022).
- [Usability Guided Key-Target Resizing for Soft Keyboards](https://www.microsoft.com/en-us/research/publication/usability-guided-key-target-resizing-for-soft-keyboards/)
  — Gunawardana, Paek y Meek, IUI 2010
  ([DOI 10.1145/1719970.1719986](https://doi.org/10.1145/1719970.1719986)):
  agrandar la zona de las teclas probables sin quitar a ninguna su centro.
- Goodman et al., «Language modeling for soft keyboards», 2002: el enfoque
  fuente-canal en el que se basa el anterior.
- [FlorisBoard](https://github.com/florisboard/florisboard): teclado libre con
  sugerencias, referencia práctica.

Datos:
- Frecuencia de letras en español (E 13,7 %, A 12,5 %, O 8,7 %, S 8,0 %, R 6,9 %,
  N 6,7 %, I 6,3 %, D 5,9 %, L 5,0 %, C 4,7 %, T 4,6 %, U 3,9 %, M 3,2 %,
  P 2,5 %, B 1,4 %, G 1,0 %): tablas habituales de frecuencia del español.

## Contexto

Es parte del trabajo de remote-mac, una app para controlar el Mac desde el
iPhone o el iPad. Allí se usará como tipo de teclado elegible; las letras se
mandan como texto, así que funcionan con cualquier distribución del Mac
(español, JIS, EE. UU.).

## Licencia

[MIT](LICENSE). Puedes usarlo, copiarlo, cambiarlo y venderlo, siempre que
conserves el aviso de copyright con el nombre del autor:
**Francesco Catania (@sekaibuilder)**. Si lo usas en un proyecto, nómbralo en
los créditos.
