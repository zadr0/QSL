---
title: Schema-Identität
description: Strict- und Light-Hash eines Schemas, was in sie einfließt und wie du sie in einem Handshake vergleichst.
---

QSL berechnet beim Analysieren des Schemas zwei Hashes für jeden Typ und legt sie als Konstanten im generierten Code ab. Kodieren und Dekodieren berechnen sie nie, und Frames enthalten sie nicht.

```rust
let strict: u128 = Packet::SCHEMA_HASH_STRICT;
let light: u128 = Packet::SCHEMA_HASH_LIGHT;
let h = Packet::SCHEMA_HASHES; // auch <Packet as qsl::SchemaIdentity>::SCHEMA_HASHES
println!("{} strict={} light={}", h.algorithm.name(), h.strict_hex(), h.light_hex());
```
Hash | Umfasst
---|---
Strict | Typ- und Feldnamen, Feldreihenfolge, Typen, Wertebereiche, Optionalität und Formatoptionen.
Light | Dasselbe ohne Typ- und Feldnamen.
`&A { u32 id; }` und `&B { u32 sequence; }` haben denselben Light-Hash und unterschiedliche Strict-Hashes. Vertauschst du zwei Felder unterschiedlichen Typs, ändern sich beide.

`SCHEMA_HASHES` ist ein Wert vom Typ `qsl::SchemaHashes`:
Feld | Bedeutung
---|---
`version` | Version des Eingabeformats für den Hash, derzeit 1.
`algorithm` | `Xxh3_64` oder `Xxh3_128`.
`strict`, `light` | Die Hashes. Bei XXH3-64 sind die oberen 64 Bits null.
`complete` | `false`, wenn der Typ einen [`QslCustom`](/guides/custom-types/#qslcustom)-Typ verwendet. Der Hash umfasst dann nur den Rust-Pfad und die Größe des eigenen Typs, nicht seinen Codec.
Vergleiche `version`, `algorithm` und `complete` zusammen mit dem Hash. `strict_hex()` und `light_hex()` formatieren einen Hash je nach Algorithmus als 16 oder 32 Hex-Ziffern.

## Algorithmus

Standard ist XXH3-64. Für 128 Bits stellst du das in einer [DEQ-Datei](/language/deq/) ein:

```deq
!config USE_HASH = xxHash128;
```

Die Einstellung gilt für den ganzen Build. Beide Algorithmen sind nicht kryptografisch: Sie erkennen Abweichungen, aber keine Manipulation.

## Was den Hash ändert

- Die Feldfolge, nachdem Konstanten ausgewertet und Rust-Templates expandiert wurden.
- Typen, Wertebereiche, Grenzen von Strings und Arrays und welche Felder optional sind.
- Optionen des Wire-Formats: `compact`, `alignment` zusammen mit `compact`, `optionals=bitmap`, `predict_seq`, `predict_seq` mit `opt_table`, `opt_table=legacy`, `state_bits`.
- String-Codecs: `decompress=true` und `decompress=true`.
- `qps` und sein Quantum. `[qps]`, `[qps=Q16]` und `[quant=Q16]` sind gleichwertig.
- Im Standardformat für optionale Felder: die Tags der optionalen Felder und die Tag-Breite.

## Was ihn nicht ändert

`safety`, `lifetime`, `inline` mit seinen Varianten, `default`, `opt_table` allein, `alignment` ohne `compact`, `aggressive`, `transmutate`, `cache` und `loyal_cache`, `QSL_RT` und die Anzahl der Slots. Sie ändern den generierten Rust-Code oder den Umgang mit Speicher, nicht die Bytes.

`String`, `&str`, `StackString` und `lifetime` erzeugen bei gleichen Grenzen dieselben Bytes und haben deshalb denselben Hash.

## Verwendung

Die Hashes beschreiben das Format konservativ. Sie beweisen nicht, dass zwei verschiedene Schemas nicht dieselben Bytes dekodieren können. Damit du sicher bist, dass beide Seiten übereinstimmen, tausche die Hashes in einem eigenen Handshake aus, bevor du Daten sendest. QSL bringt keinen Handshake mit.
