---
title: Strings und Ownership
description: Welche Rust-Typen aus einem QSL-String werden können, wann welcher gewählt wird und was das Dekodieren mit dem Speicher macht.
---

UTF-8 wird in Bytes gezählt: «Яa» belegt 3 B. String besitzt dynamischen Speicher, &str leiht die lebende Eingabe und StackString/InlineString haben begrenzte Kapazität. QSL-LZ komprimiert nur bei tatsächlicher Verkleinerung, sonst bleibt raw. Das Diagramm zeigt Verteilung, Literale und Rückverweise.

## Speicher und Länge

InlineString behält seinen Codec im Objekt oder in einer Box auf dem Heap. Große Heap-Strings werden direkt am Ziel wiederhergestellt. StackString und InlineArray enthalten ihre begrenzte Kapazität. aggressive[shift=true] kann eine NonZero-Länge wählen; None und Some(empty) bleiben verschieden.

## QSL-LZ {#compressed-strings}

Jedes expandierte Token, Rückverweisabstand, UTF-8 und Gesamtbudget werden geprüft. Eine gespeicherte Länge erlaubt keine größere Wiederherstellung. Längen sind UTF-8-Byte, nicht Unicode-Zeichen.
[Arrays](/language/arrays/) · [Limits](/reference/limits/) · [Memory](/guides/manual-layout/) · [QPS](/guides/smaller-frames/)

no_std schließt std aus, nicht alloc. qsl-core verwendet immer #![no_std]. Das standardmäßig aktive alloc-Feature bietet alloc String/Vec, SerOut und Wiederherstellung im Heap. Der Allokator kommt von der Anwendung; QSL definiert keinen globalen Allokator. Ohne Standardfeatures werden nur Puffer des Aufrufers verwendet.

Der Generator wählt den Ausgabetyp anhand der bewiesenen maximalen Framegröße. Bis einschließlich STACK_LIMIT wird `StackOut<N>` verwendet; eine größere oder unbekannte Grenze wählt alloc::`Vec<u8>`. Übertragene Header zählen mit, die Metadaten des Ausgabeobjekts nicht.
