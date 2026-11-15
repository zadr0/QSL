---
title: 'Arrays und Bitfenster'
description: 'Arraylängen zählen Elemente, Zeichenkettenlängen UTF-8-Bytes. InlineArray hält begrenzte Kapazität im Objekt, Vec besitzt eine eigene Allokation. STACK_LIMIT begrenzt eingebettete Speicherung strikt. Ein Vec-Deskriptor ist nicht der Payload.'
---

Arraylängen zählen Elemente; Stringlängen zählen UTF-8-Bytes. `array<u16>(0..16)` erlaubt null bis sechzehn Elemente, `array<u16>(16)` verlangt genau sechzehn.

Vec besitzt über alloc einen separaten Puffer. InlineArray enthält begrenzte Kapazität im Objekt; feste Arrays können `[T;N]` verwenden. STACK_LIMIT begrenzt eingebetteten Speicher. Der Vec-Deskriptor ist nicht der Payload.

$$
B_{\mathrm{array}}=H(n)+\sum_{i=1}^{n} w_i.
$$

H sind die Längenmetadaten in Bits, n die Elementzahl und w die jeweilige Codecbreite. Ausrichtung wird zusätzlich gerechnet. Eine Länge erlaubt kein Lesen außerhalb des Frames. Bis zu 32 Verschachtelungsebenen sind möglich; Domänen und Längen werden separat geprüft. Das Leseregister hängt von der Bitposition ab und ist nicht stets 128 Bit breit.

Explizites `aggressive[shift=true]` kann für InlineArray eine passende NonZero-Länge wählen. Standard ist u32. Diese physische Optimierung erhält das Wireformat und unterscheidet None von Some(empty).
[Read windows](/internals/optimizer/) · [Memory](/guides/memory/)

no_std schließt std aus, nicht alloc. qsl-core verwendet immer #![no_std]. Das standardmäßig aktive alloc-Feature bietet alloc String/Vec, SerOut und Wiederherstellung im Heap. Der Allokator kommt von der Anwendung; QSL definiert keinen globalen Allokator. Ohne Standardfeatures werden nur Puffer des Aufrufers verwendet.
