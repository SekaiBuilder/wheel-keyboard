# Teclado de ruedas — prototipo web

> Copia de la spec del prototipo tal como está en remote-mac. Los commits de la
> tabla de evidencia son de ese repositorio; los códigos como CTR-04 o CAM-013
> son reglas y cambios de remote-mac.

Nivel SDD: mínimo. Estado: prototipo en prueba. Fecha: 2026-11-23.
Relación con la app: si funciona, se portará a SwiftUI como segundo tipo de
teclado elegible en Ajustes (cambio CAM aparte; nada de esto toca `apple/`).

## Propósito

Escribir en el Mac desde el móvil con los dos pulgares más rápido y con menos
errores que con el teclado QWERTY en pantalla de la app, incluidos los atajos
(⌘C, ⌘V, ⌘Tab…).

Titular, 2026-11-23: «salen dos ruedas que son la parte izquierda y la parte
derecha del teclado… al oprimirla se abre, se expanden todos los botones… y
moviendo el dedo hacia las direcciones se comienza a teclear… como el teclado
japonés en cruz… pero con un rango mayor y circular». Teclas extra con «⋯»; las
teclas comunes, en la capa inicial; el teclado debe ser adaptativo al contexto.

## Referencias y qué se toma de cada una

| Referencia | Qué enseña | Qué se toma |
|---|---|---|
| Marking menus (Kurtenbach y Buxton, 1993–94) | Marcar por dirección es ~3,5× más rápido que elegir en el menú; con <10 % de errores, 8 opciones por nivel admiten 2 niveles | 8 direcciones por anillo como máximo en el anillo interior; dos niveles como mucho (variante B) |
| Flick japonés (kana) | Tocar una tecla y deslizar en cruz elige entre 5 | La rueda sale donde cae el pulgar y la dirección elige |
| MessagEase / Thumb-Key (3×3, código abierto) | Las letras frecuentes van en el gesto más corto; atajos (deshacer, pegar) en gestos | Frecuentes en el anillo interior; atajos con nombre en las letras cuando hay modificador |
| KALQ (Oulasvirta et al., CHI 2013) | Teclado partido para pulgares: 37 ppm tras entrenar, frente a ~20 del QWERTY partido; reparte letras para alternar pulgares | Dos mitades; repartir para alternar; medir ppm y errores |
| 8pen / 8VIM | Gesto circular desde el centro; lento hasta memorizar si los gestos son largos | Gestos cortos (un trazo), etiquetas siempre visibles para novatos |
| Unexpected Keyboard | Deslizar a las esquinas de una tecla; Ctrl y compañía para terminal | Capa «⋯» con ⌘ ⌥ ⌃ Esc Tab F1–F12 y flechas |
| Frecuencia de letras en español (E 13,7 %, A 11,7 %, O 8,4 %, S 7,2 %, N 6,8 %, R 6,4 %…) | Qué letras merecen el gesto más corto | Se usó en la v1 (anillos por frecuencia); retirado en la v2: el titular prefiere la posición del QWERTY, que ya sabe de memoria |

## Diseño

### Versión 3 (2026-11-23): ocho direcciones y predicción

Cambia porque el titular, tras probar la v2: «tres líneas está un poco
complicado… que fuese más sencillo de utilizar y más intuitivo», y pidió que el
teclado aprenda con el uso, resalte la letra probable y corrija un toque que cae
en el borde «si se siente que oprimió la E, sí; si se sintió que oprimió la I,
no». Eligió la propuesta A de `referencias.html` con tres ajustes: vocales en
la mano que se elija, espacio en el pulgar de las consonantes (equilibra la
carga) y capa ⋯ según el teclado del Mac (JIS).

- **Ruedas de 8 direcciones** (0 N, 1 NE, 2 E, 3 SE, 4 S, 5 SW, 6 W, 7 NW),
  sectores de 45°, círculo entero centrado donde cae el pulgar.
  - Anillos (fracción de R): zona muerta 0–0,18; interior 0,18–0,60; exterior
    desde 0,60. Histéresis: el interior retiene `margen × 1,5`, el exterior
    `margen`; las direcciones retienen 6°.
  - Consonantes, interior: N S · NE R · E N · SE P · S G · SW B · W T · NW C;
    exterior: V Q H J W X Z F.
  - Vocales, interior: N E · NE I · E D · SE U · S M · SW L · W O · NW A;
    exterior: Y Ñ ? K - ¿ . ,
  - Las 16 letras del interior suman ~95 % de las letras en español.
  - «Vocales en la mano» (Ajustes, derecha por defecto): con «izquierda», las
    dos ruedas se intercambian y se reflejan (E↔W).
- **Centro:** tocar sin mover es espacio en la rueda de las consonantes y ⌫ en
  la de las vocales (mantener, repite). Con el espacio en el pulgar de las
  consonantes la carga estimada queda ~47 % / ~48 %.
- **Columna central:** ⇧ (un toque / doble toque fijo), ⋯ y ⏎; con teclado JIS,
  además «あ/A», que manda かな o 英数 como las teclas del Mac.
- **Capa ⋯:** izquierda ⌘ ⌥ ⌃ Esc ⏎ ⌦ ⇥ ⇧ dentro y 1–5 ( ) = fuera; derecha
  flechas en N E S W y signos según el contexto dentro, 6–0 y signos fuera
  (en JIS, ¥ en vez de «;»).
- **Aviso de paso:** el de la v2 pasa a darse al cambiar de anillo.

#### v3.1 (2026-11-23), tras probarla el titular

Titular: «necesito que el círculo quede total y los botones del medio… en la
parte de arriba del pulgar y dos en el otro pulgar»; «aligerezcas un poquito la
sensibilidad… que no sea necesario llegar hasta la letra… si se pasa la
vibración, se pueda pasar a la siguiente línea».

- El círculo se ve siempre entero: `R = min((alto − 52) / 2, ancho / 4,2, 190)`
  y el centro se mantiene a R del borde. La franja de 48 px de arriba es para
  los botones.
- Botones encima de cada rueda en reposo: izquierda ⇧ ⋯; derecha あ/A (JIS) y ⏎.
  El registro «→ Mac» pasa dentro de la zona, arriba en el centro, y la fila de
  sugerencias comparte línea con los atajos: más alto para las ruedas.
- Paso entre anillos más corto: zona muerta 0–0,15 R, interior hasta 0,50 R.
  Margen nuevo `margen3` = 0,03 R (0–0,08), igual para entrar y salir del
  interior; las direcciones retienen 3°. El aviso (vibración / destello) llega
  justo al pasar el umbral, sin tener que llegar a la letra.
- Sustituye en R-16 los valores «interior retiene margen × 1,5» y «6°».

#### Predicción (empieza en blanco)

- Aprende al cerrar cada palabra (espacio, signo, ⏎ o sugerencia), con la
  palabra tal como quedó tras borrar: cuentas de letra según las 0–3 letras
  anteriores de la palabra (tildes fuera, ñ aparte), cuentas de palabras y de
  pares de palabras.
- Probabilidad de letra: suavizado recursivo de Dirichlet, α = 1,5, desde la
  uniforme hasta el contexto de 3 letras.
- Resalta hasta 3 letras con p ≥ 0,12 (cuando el contexto de 1 letra tiene al
  menos 8 casos): sector teñido y letra en negrita. El dibujo no cambia de
  tamaño.
- **Regla del borde:** sólo si el ángulo cae a menos de 8° del borde del sector
  (o pasado por la histéresis) y la vecina del mismo anillo es una letra. Se
  cambia a la vecina si `P(vecina) ≥ c · P(tocada)` y
  `gauss(Δvecina) · P(vecina) > gauss(Δtocada) · P(tocada)`, con σ = 12°,
  c = 3 + 0,5 por cada corrección deshecha (tope 13) y al menos 60 letras
  aprendidas. Se ve antes de soltar (vista previa «corregido»).
- ⌫ justo después de una letra corregida pone la que se tocó y sube c.
- Barra de 3 palabras: completan el prefijo (puntuación = usos + 4 × usos tras
  la palabra anterior) o, sin prefijo, siguen a la anterior.
- Se guarda en el navegador (`ruedas.modelo`, JSON): hasta 5000 palabras, se
  quitan las menos usadas. «Borrar lo aprendido» en Ajustes, con confirmación.
- Con «El Mac está en un campo de contraseña» (simulado; en la app lo dirá el
  Mac) no aprende ni sugiere nada (como CTR-04).

### Versión 2 (2026-11-23), sustituida por la v3

Versión 2 (2026-11-23). Cambia porque el titular, tras probar la v1, dijo:
- «El teclado está mal distribuido… que se distribuya también el QWERTY: los
  números no van abajo, van arriba… que una persona que utiliza el teclado muy
  seguido sepa dónde están las teclas intuitivamente».
- «Tantas veces quiero oprimir la A, pero salta a la siguiente línea… es muy
  sensible el traspaso».

Lo que se decidió:
- **Abanico QWERTY.** Cada rueda es su mitad del teclado tal como está en el
  Mac. Las letras y los números están igual en el QWERTY español y en el JIS.
  - Columnas: de izquierda a derecha como en el teclado, 5 de 30° entre −165° y
    −15° (arriba son −90°).
  - Filas: de cerca a lejos, abajo (Z), inicio (A), arriba (Q) y números, igual
    que de abajo arriba en el teclado.
  - Izquierda: `1–5`, `Q W E R T`, `A S D F G`, `Z X C V B`.
  - Derecha: `6–0`, `Y U I O P`, `H J K L Ñ`, `N M , . -`.
- **Filas** (fracción de R): zona muerta 0–0,20; abajo 0,20–0,42; inicio
  0,42–0,68, la más ancha; arriba 0,68–0,86; números desde 0,86.
- **Histéresis.** Para dejar la fila en la que estás hay que pasar su borde por
  un margen: 0,06 R por defecto, ajustable de 0 a 0,12. La fila de inicio
  retiene 1,5 veces más. Las columnas retienen 6°, y el paso entre la mitad de
  arriba y la de abajo, 12°.
- **Aviso de paso de fila** (titular: «una pequeña vibración para comprender
  cuándo hemos pasado la línea»). Cada vez que la selección cambia de fila, o
  entre las letras y las teclas de abajo, después de aplicar la histéresis, hay
  aviso. Salir de la zona muerta no lo da.
  - Android: `navigator.vibrate(5)`.
  - iPhone: Safari no tiene `vibrate`, y el truco del interruptor de iOS 18 sólo
    vibra dentro de un toque o al soltar, no mientras el dedo se mueve. Por eso
    allí el aviso es visual: la fila nueva destella 0,18 s con un borde grueso,
    y la fila actual queda siempre perfilada. Con el sonido activado, hace
    además un «tic» grave (700 Hz).
  - App nativa (al portar): `UIImpactFeedbackGenerator(style: .light)` en cada
    paso de fila.
  - Se puede apagar en Ajustes («Aviso al cambiar de fila»).
- **Debajo del centro**, tres teclas de uso continuo:
  - Izquierda: ↙ ⇥, ↓ ⇧, ↘ ⋯.
  - Derecha: ↙ ⋯, ↓ ⏎, ↘ ⌫.
- **Rueda flotante.** Se dibuja donde cae el pulgar mientras el abanico quepa;
  la selección cuenta siempre desde el punto tocado. Como sólo la mitad de
  arriba lleva letras, el radio crece: R = 154 px en 812×375, frente a 113 en
  la v1.
- **Zona muerta:** tocar sin salir de ella es la tecla del centro (derecha:
  espacio; izquierda: ⌫, que se repite si se mantiene).
- **Cancelar:** salir de la zona muerta y volver a ella antes de soltar no
  escribe nada.

### Retirado en la v3 (2026-11-23)

- R-12 (abanico QWERTY) y la histéresis de filas de R-13 pasan a anillos (R-15,
  R-16). El historial se conserva: la tabla enseña «v3», «QWERTY» y «A».

### Retirado en la v2 (2026-11-23)

- Variante A, anillos por frecuencia: 8 en el interior y 12–13 en el exterior.
- Variante B, pétalos: 8 grupos de 3, y torcer el trazo elige la letra.
- Las sustituye el abanico QWERTY. Los datos afectados son el historial de
  práctica, que se conserva: la tabla de medias enseña la v2 («QWERTY») y la
  antigua «A».

### Capa «⋯» y modificadores

- La pastilla «⋯» cambia las dos ruedas:
  - Izquierda: en la fila de inicio ⌃ ⌥ ⌘ ⇧ ⇥ (el orden del Mac); F1–F10 arriba; Esc ⌦ F11 F12 ⏎ abajo.
  - Derecha: flechas ← ↑ ↓ → en la fila de inicio y signos según el contexto (los de ⇧ + número arriba).
- También se llega a «⋯» deslizando hacia ↘ (izquierda) o ↙ (derecha).
- Un modificador elegido:
  - Un toque: vale para la siguiente tecla (luz).
  - Dos toques en menos de 0,4 s: queda fijo.
  - Al elegirlo, la rueda vuelve sola a las letras, así ⌘ y luego C sale sin
    buscar la C en otra capa (lo que pedía el titular: «bloquear esas teclas»).
- Acorde: si el pulgar izquierdo se queda sobre ⌘ y el derecho escribe, ⌘ se
  mantiene mientras haya dedo, igual que en la app desde 156bb50.
- Con un modificador activo, cada letra enseña debajo el nombre del atajo
  (C Copiar, V Pegar, Z Deshacer…).

### Adaptativo según el contexto

- Selector de contexto (en la app lo dirá el Mac por la app de delante):
  Texto, Terminal, Finder, Navegador. Cambia:
  - Mayúscula automática: sólo en Texto, al empezar y tras «. ».
  - Los signos de la capa «⋯» derecha. En Terminal: `| ~ / -` y ⌃ con nombres
    de terminal (⌃C Parar, ⌃R Buscar…).
  - Las pastillas de atajos frecuentes.
- Las pastillas aprenden: se ordenan por uso (contador en el navegador).

### Acentos

Mantener el dedo quieto sobre una vocal 0,45 s la cambia por la acentuada
(otra vez, ü). Se ve en la vista previa antes de soltar.

### Háptica

`navigator.vibrate` no existe en Safari de iOS.
- Se usa el `<input type="checkbox" switch>` de iOS 18 o posterior, que vibra
  al pulsarse. Sólo funciona dentro de un gesto del usuario: al soltar.
- En Android se usa `vibrate`.

## Requisitos (objetivo)

| ID | Requisito |
|---|---|
| R-01 | Dos ruedas, una por mitad de pantalla, que se abren donde cae cada pulgar; los dos pulgares a la vez (multitáctil) |
| R-15 | Cada rueda tiene 8 direcciones y dos anillos; las 16 letras frecuentes en el interior, vocales en una mano y consonantes en la otra, con la mano elegible |
| R-16 | Histéresis entre anillos (el interior retiene más) y entre direcciones (6°); aviso al cambiar de anillo |
| R-17 | Centro: espacio en la rueda de consonantes y ⌫ en la de vocales; ⇧ ⋯ ⏎ en la columna central |
| R-18 | Con teclado del Mac JIS, botón あ/A (かな/英数) y ¥ en la capa ⋯ |
| R-19 | Predicción que empieza en blanco y aprende al cerrar palabras; resalta hasta 3 letras probables sin cambiar el dibujo |
| R-20 | Corrección sólo en la franja de borde (8°), con confianza mínima; visible antes de soltar; ⌫ inmediato la deshace y la vuelve más prudente |
| R-21 | Barra de 3 palabras sugeridas; tocar una la completa y añade espacio |
| R-22 | Lo aprendido no sale del navegador, tiene tope, se puede borrar, y no se aprende en un campo de contraseña |
| R-12 | Retirado 2026-11-23 (v3). Lo sustituye R-15. Antes: las teclas de cada rueda siguen el QWERTY: columnas de izquierda a derecha y filas de cerca a lejos (abajo, inicio, arriba, números) |
| R-13 | Retirado 2026-11-23 (v3) para las filas; lo sustituye R-16. Antes: para cambiar de fila o columna hay que pasar el borde por un margen (histéresis); la fila de inicio es la más ancha y retiene más |
| R-14 | Al pasar de una fila a otra (tras la histéresis) hay un aviso corto: vibración donde exista y, siempre, destello de la fila; en la app nativa, `UIImpactFeedbackGenerator` ligero |
| R-02 | Retirado 2026-11-23 (variantes A y B). Lo sustituye R-12; el historial antiguo se conserva |
| R-03 | Mientras el dedo se mueve, se ilumina el sector y una vista previa grande enseña lo que saldrá al soltar; volver al centro cancela |
| R-04 | Tocar el centro: espacio a la derecha y ⌫ a la izquierda (mantener, repite) |
| R-05 | Capa «⋯» con modificadores de un toque, fijos con doble toque, y acorde manteniendo el dedo; tras elegir un modificador, la rueda vuelve a las letras |
| R-06 | Con un modificador activo, las letras enseñan el nombre del atajo según el contexto, y el atajo sale como combinación (registro «→ Mac»), no como letra |
| R-07 | Contexto Texto, Terminal, Finder o Navegador: mayúscula automática, signos y pastillas de atajos frecuentes ordenadas por uso |
| R-08 | Vocal mantenida 0,45 s, acentuada |
| R-09 | Modo práctica con frases en español: ppm, errores (distancia de edición) e historial por variante |
| R-10 | Ajustes: tamaño de rueda, margen entre filas, háptica, sonido, etiquetas en reposo |
| R-11 | Vertical y horizontal; sin zoom, selección de texto ni menú contextual del navegador al teclear |

## Aceptación

| ID | Dado / cuando / entonces | Prueba |
|---|---|---|
| A-01 | Con la variante A, un trazo corto hacia arriba con el pulgar izquierdo escribe «e» y uno largo, la letra del anillo exterior | táctil emulado + titular |
| A-02 | Retirado con R-02 |
| A-08 | Derecha inicio col 1 «h», arriba col 4 «o», inicio col 4 «l», izquierda inicio col 1 «a» → «Hola»; números col 1 izquierda «1», col 5 derecha «0» | táctil emulado + titular |
| A-10 | Abajo → inicio: un aviso; moverse dentro del margen: ninguno; inicio → arriba: otro aviso, con destello y la fila perfilada | táctil emulado + titular (sensación en el iPhone) |
| A-09 | Ir a la A y pasarse hasta 0,74 R (borde a 0,68) sigue siendo «a»; llegar a 0,80 R es «q» | táctil emulado + titular |
| A-11 | Consonantes: toque → espacio, N → «s», NE exterior → «q»; vocales: N → «e», NW → «a», toque → ⌫ | táctil emulado + titular |
| A-12 | Con vocales a la izquierda, N de la izquierda → «e» y NE de la izquierda → «a» (reflejo) | táctil emulado |
| A-13 | Tras aprender «queso» varias veces, «qu» resalta la E; un toque a 3° del borde de la I hacia la E escribe «e» (vista previa «corregido»); en el centro de la I escribe «i» | táctil emulado + titular |
| A-14 | ⌫ justo después de una corrección deja la letra tocada y sube la prudencia | táctil emulado |
| A-15 | Tras escribir «hola mundo», «ho» sugiere «hola»; tocarla deja «hola » | táctil emulado |
| A-16 | En modo contraseña no se aprende ni se sugiere | táctil emulado |
| A-17 | Teclado JIS: aparece あ/A y manda かな/英数; con ISO no aparece | táctil emulado |
| A-03 | Salir y volver al centro antes de soltar no escribe nada | táctil emulado |
| A-04 | «⋯», ⌘, «⋯» se cierra solo, C → registro «⌘C» y las letras muestran «Copiar» mientras ⌘ está activo | táctil emulado |
| A-05 | Mantener ⌘ con el izquierdo y tocar C dos veces con el derecho (en la capa de letras de la derecha no hay C: se usa V) → «⌘V ⌘V» | titular en el iPhone |
| A-06 | Una frase de práctica da ppm y errores, y queda en el historial | táctil emulado + titular |
| A-07 | Escribir una frase en el iPhone en horizontal, sin zoom ni selección accidental | titular en el iPhone |

## Decisiones abiertas

- D-01: Cerrada 2026-11-23: QWERTY en abanico (titular). Abierta: ¿escalonar las columnas como el teclado real?
- D-02: Qué contexto manda el Mac a la app (la app de delante); hace falta un
  mensaje nuevo en el protocolo (CTR-05) al portarlo.
- D-03: Cerrada 2026-11-23: predicción local que aprende (v3). Abierta: ¿proponer una disposición personalizada a partir de lo aprendido?
- D-04: En la app, ¿JSON o SQLite para el modelo?, y mensaje del agente para «campo de contraseña» (`IsSecureEventInputEnabled`) y tipo de teclado (CTR-05).

## Evidencia

| Fecha | Qué | Commit | Entorno | Resultado |
|---|---|---|---|---|
| 2026-11-23 | A-01: trazo corto ↑ izquierda → «E» (mayúscula automática), largo → «f»; toque derecho espacio, toque izquierdo ⌫ | e9927ce | navegador integrado, 812×375, eventos de puntero sintéticos (R = 113 px) | correcto |
| 2026-11-23 | A-02: variante B, pétalo ↑ recto «e», torcer a la derecha «w», a la izquierda «q» | e9927ce | ídem | correcto |
| 2026-11-23 | A-03: salir y volver al centro → no escribe, «Cancelado» | e9927ce | ídem | correcto |
| 2026-11-23 | A-04: «⋯» → ⌘ → capa de letras sola, C → «→ Mac ⌘C Copiar», ⌘ se apaga; con ⌘ activo, las letras en reposo enseñan Copiar, Pegar, Deshacer… | e9927ce | ídem | correcto |
| 2026-11-23 | Acorde: pulgar izquierdo quieto en ⌘ (0,16 s) → rueda derecha de atajos C V Z X A S F ⇥; ↑ → ⌘C, ↗ → ⌘V; al soltar, ⌘ no queda activo | e9927ce | ídem | correcto |
| 2026-11-23 | R-08: O mantenida 0,5 s → «Ó» | e9927ce | ídem | correcto |
| 2026-11-23 | Publicado como artefacto privado: https://claude.ai/artifact/GVPMncfDfSEwsoZAob1c3Y | e9927ce | claude.ai | publicado |
| 2026-11-23 | R-14/A-10: entrar en la fila de abajo, 0 avisos; abajo → inicio, 1 (destello, 5 sectores perfilados, `vibrate(5)`); 0,72 R dentro del margen, sigue en 1; 0,80 R, 2; al soltar, «Q» | este commit | navegador integrado, 812×375, `navigator.vibrate` simulado | correcto |
| 2026-11-23 | v3.1: en 812×375 R = 93 px y las dos ruedas caben enteras; botones encima de cada rueda; ↑ izquierda hasta 0,52 R → «S», hasta 0,56 R → «V»; volver del exterior a 0,49 R sigue en «V», a 0,45 R → «S» | este commit | navegador integrado, 812×375, eventos sintéticos | correcto |
| 2026-11-23 | v3: A-11 «Se qa» (S, e, espacio, q exterior tras 1 aviso de anillo, a) y toque derecho ⌫; A-12 vocales a la izquierda «Ea»; A-13 tras 16 palabras con «qu», p(e) 0,92, E resaltada, toque a 25° (2,5° dentro de la I) → «e» con vista previa «corregido (tocaste i)» y la I con borde discontinuo, centro de la I → «i»; A-14 ⌫ tras la corrección → «qui», rechazos 1; A-15 «ho» → «hola», tocarla → «hola », después sugiere «mundo»; A-16 contraseña: 0 sugerencias, 0 letras aprendidas; A-17 ISO sin あ/A, JIS con él, ⋯ derecha con ¥; sin `localStorage` (página `data:`) no falla | este commit | navegador integrado, 812×375, eventos sintéticos, R = 117 px | correcto |
| 2026-11-23 | v2: A-08 «Hola10», fila de abajo «c», A-09 (pasarse de la A sigue en «a»; 0,80 R es «q»), ↘ izquierda abre «⋯», ⌘ en la fila de inicio y luego C → «⌘C Copiar», ↓ derecha → ⏎ | 64a7f78 | navegador integrado, 812×375, eventos sintéticos, R = 154 px | correcto |

## No ejecutado

- v3: ningún toque real; la carga 47/48 % es una estimación con frecuencias del español, sin medir (la práctica ya la mide: «izq % / der %»). Vertical sin comprobar. El modelo se probó con palabras inyectadas, no tras días de uso.

- Ningún toque real: todo con eventos sintéticos. A-05, A-06 (ppm reales) y A-07 son del titular en el iPhone.
- Vibración en iOS (truco del interruptor de iOS 18, sólo al tocar y soltar): sin probar en el aparato. El aviso de fila en iPhone es sólo visual o sonoro, por la limitación de Safari.
- Vertical: sólo cálculo (en 375 de ancho, R ≈ 88 px y las dos ruedas en reposo no se tocan); sin captura.
- Dos pulgares a la vez de verdad (sólo un acorde sintético).
