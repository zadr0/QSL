---
title: 'Runtime und Puffer'
description: 'SerOut und vorbereitete Puffer steuern Allokationen und Wiederverwendung. QSL_RT erzeugt statische Slots nur für geeignete begrenzte Schemas, standardmäßig 16 je Schema. Bei Erschöpfung folgt ein Fehler ohne versteckte Allokation. Plattformspezifische Sockets und Speicherregionen gehören nicht zur aktuellen Runtime.'
---

no_std schließt std aus, nicht alloc. qsl-core verwendet immer #![no_std]. Das standardmäßig aktive alloc-Feature bietet alloc String/Vec, SerOut und Wiederherstellung im Heap. Der Allokator kommt von der Anwendung; QSL definiert keinen globalen Allokator. Ohne Standardfeatures werden nur Puffer des Aufrufers verwendet.

Statische no_std-Regionen implementieren FramePool/FrameStorage. Ein voller Pool liefert sofort PoolExhausted.
API                             | Contract
------------------------------- | -----------------------------------------------------------
SliceOut                        | caller-owned initialized buffer
SerOut                          | alloc-backed reusable output
FrameStorage / FramePool        | core memory-provider contracts
StaticRegions / RegionLease     | qsl-rt; 1–64 exclusive slots, release on Drop
QuantizedOut / QuantizedRegions | scalar QPS output; initialized padding outside wire
RawSocket<S> / ByteTransport    | u32 LE length + payload; application transport, no std::net
InlineString behält seinen Codec im Objekt oder in einer Box auf dem Heap. Große Heap-Strings werden direkt am Ziel wiederhergestellt. StackString und InlineArray enthalten ihre begrenzte Kapazität. aggressive[shift=true] kann eine NonZero-Länge wählen; None und Some(empty) bleiben verschieden.

Build-Skripte und Tests dürfen std nutzen, ohne es in die Ziel-Runtime einzubinden. Echte generierte Schemata laufen ohne Allokator; eine separate no_std-Bibliothek prüft alloc-Container. Beide Kernprofile und qsl-rt werden auch für wasm32-wasip2 gebaut.

[Memory](/guides/memory/) · [Safety](/guides/safety/)

Der Generator wählt den Ausgabetyp anhand der bewiesenen maximalen Framegröße. Bis einschließlich STACK_LIMIT wird `StackOut<N>` verwendet; eine größere oder unbekannte Grenze wählt alloc::`Vec<u8>`. Übertragene Header zählen mit, die Metadaten des Ausgabeobjekts nicht.
