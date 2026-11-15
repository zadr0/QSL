---
title: Puffer und Wiederverwendung
description: In eigene Puffer schreiben, in vorhandene Werte dekodieren, der statische RT-Slot-Pool und gecachte Ausgabe-Frames.
---

Der Generator wählt den Ausgabetyp anhand der bewiesenen maximalen Framegröße. Bis einschließlich STACK_LIMIT wird `StackOut<N>` verwendet; eine größere oder unbekannte Grenze wählt alloc::`Vec<u8>`. Übertragene Header zählen mit, die Metadaten des Ausgabeobjekts nicht.

String, Vec und Box verwenden alloc innerhalb von no_std. Den Allokator stellt die Anwendung bereit. InlineString liegt direkt im Objekt, wenn seine native Größe in STACK_LIMIT passt, andernfalls in einer Box. Maßgeblich ist die Schemakapazität.

Vorbereitetes unsafe reuse erfordert ausreichende Kapazität vor dem Aufruf und behält geeignete String-, Vec- und boxed InlineString-Puffer. Diese Puffer wachsen dabei nicht. Initialisierte Objekte und uninitialisierte Ziele haben getrennte Verträge.

<a id="reading-into-existing-memory"></a>

## Beispiel {#example}

```qsl
&Record[safety] { u64 id; }
```

```rust
let value = Record { id: 42 };
let mut buffer = [0u8; 64];
let used = value.serialize_checked_to(&mut buffer)?;
let mut destination = core::mem::MaybeUninit::<Record>::uninit();
let restored = Record::deserialize_checked_into(&buffer[..used], &mut destination)?;
assert_eq!(restored.id, 42);
```

[STACK_LIMIT](/language/deq/) · [String](/language/strings/) · [RT](/internals/runtime/)

## QSL_RT {#rt-slot-pool}

LocalRegions · StaticRegions · FramePool · FrameStorage. 1–64 slots; lease → Drop. PoolExhausted.

## StackOut / Vec {#cached-output-frames}

`STACK_LIMIT = 64`: `StackOut<64>`; 65 B → `Vec<u8>`.
