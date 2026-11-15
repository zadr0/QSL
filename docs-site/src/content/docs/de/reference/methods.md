---
title: Generierte Methoden
description: Methoden und Konstanten, die QSL für jeden Schematyp generiert, gruppiert danach, was sie vom Aufrufer verlangen.
qslResources: [{"label": "docs.rs", "url": "https://docs.rs/qsl/0.3.0/qsl/"}]
---

Jede Deklaration erzeugt einen Rust-Typ mit den folgenden Methoden. Welche davon es gibt, hängt von den Attributen des Schemas ab. Die vollständige Liste der öffentlichen Items in den Runtime-Crates findest du auf docs.rs.

## Geprüft: sicher bei nicht vertrauenswürdigen Daten

Nur generiert, wenn der Typ `[safety]` hat:
Methode | Signatur (vereinfacht) | Hinweise
---|---|---
`deserialize_checked` | `fn(&[u8]) -> QslResult<Self>` | Validiert beim Dekodieren. Mit `lifetime` ist die Eingabe `&'a [u8]`.
`deserialize_checked_into` | `fn(&[u8], &mut MaybeUninit<Self>) -> QslResult<&mut Self>` | Dekodiert in deinen Speicher. Bei einem Fehler bleibt das Ziel uninitialisiert.
Für jeden Typ generiert:
Methode | Signatur (vereinfacht) | Hinweise
---|---|---
`validate` | `fn(&[u8]) -> QslResult<()>` | Prüft einen ganzen Frame, ohne einen Wert aufzubauen. Vor `deserialize_checked` nicht nötig.
`validate_values` | `fn(&self) -> QslResult<()>` | Prüft Wertebereiche, Längen, UTF-8 fester Strings und das Budget `MAX_DATA_SIZE`.
`serialize_checked` | `fn(&self) -> QslResult<`Vec<u8>`>` | Prüft den Wert und kodiert ihn dann.
`serialize_checked_to` | `fn(&self, &mut [u8]) -> QslResult<usize>` | Kodiert in einen Slice mit mindestens `serialized_size()` Bytes und gibt die Anzahl geschriebener Bytes zurück.
Typen mit einem optionalen numerischen Feld bekommen außerdem:
Methode | Signatur (vereinfacht) | Hinweise
---|---|---
`try_new` | `fn(field, ...) -> QslResult<Self>` | Nimmt alle Felder in Deklarationsreihenfolge entgegen, optionale als `Option<T>`, und prüft die Werte.
`name` | `fn(&self) -> Option<T>` | Liest das optionale Feld `name`.
`set_name` | `fn(&mut self, Option<T>) -> QslResult<()>` | Ändert es. `InvalidValue`, wenn der Wert außerhalb des Bereichs liegt.
Siehe [Optionale Felder in Rust](/language/declarations/#optional-fields-in-rust).

## Vertrauenswürdig: Der Aufrufer garantiert die Gültigkeit

Das sind Methoden des Traits `qsl::Serializable`. Typen mit `lifetime` implementieren stattdessen `qsl::BorrowSerializable<'a>`; dort heißen die Decoder `deserialize_borrowed`, `deserialize_borrowed_into` und `deserialize_borrowed_unsafe`. In beiden Traits ist jede Methode außer `serialized_size` und `get_size` als `unsafe` markiert.
Methode | Was der Aufrufer garantiert
---|---
`serialize`, `serialize_to`, `serialize_into_unsafe`, `serialize_unsafe` | Der Wert ist für das Schema gültig; das Ziel hat `serialized_size()` beschreibbare Bytes.
`deserialize`, `deserialize_unsafe` | Die Eingabe ist genau ein vollständiger, gültiger Frame.
`deserialize_into(ptr, len, out)` | Gültiger Frame; `out` ist beschreibbar, ausgerichtet und überlappt sich nicht mit der Eingabe. Der alte Wert in `out` wird nicht gedroppt.
`deserialize_reuse(input, &mut value)` | Gültiger Frame; jeder wiederverwendete String-Puffer hat genug Kapazität. Behält Puffer, soweit das Layout es erlaubt.
`deserialize_prepared_into(ptr, len, &mut value)` | Dasselbe mit Rohzeiger und Länge.
Sichere Trait-Methoden:
Methode | Rückgabe
---|---
`serialized_size(&self)` | Der Platz, den du für diesen Wert bereitstellen musst: die exakte Frame-Länge oder, bei Werten mit komprimierten Strings, eine Obergrenze.
`get_size()` | Die kleinstmögliche Frame-Länge.
## Spezialisiert
Methode | Wann
---|---
`serialize()` | Mit RT-Pool: sicher, prüft den Wert, wartet auf einen freien Slot und gibt eine Lease zurück. Bei einem Typ mit aktiver `qps`-Packung ohne RT-Pool: sicher, gibt `QuantizedOut` zurück. Siehe [RT-Slot-Pool](/guides/memory/#rt-slot-pool).
`serialize_quantized_to` | Bei aktivem `qps`. Prüft Werte und Kapazität und führt dann den Pack-Kernel aus.
`quant_write_prepared`, `quant_read_padded` | Bei aktivem `qps`. Methoden von `qsl::payload_quant::QuantizedRecord`. `unsafe`: Der Aufrufer garantiert Wertebereiche, Padding sowie disjunkten, ausgerichteten Speicher.
`serialize_qps_to`, `deserialize_qps_into`, `deserialize_qps` | `ByteQps`-Adapter. Nur für Typen mit `[safety]` und `qps` und ohne Strings oder Arrays.
`serialize_cached` | `cache=N` oder `loyal_cache`. Schreibt in einen `CachedFrame`.
`deserialize_transmutate_copy`, `deserialize_overlay` | [`transmutate`](/reference/attributes/#transmutate). Jede hat eine `_unchecked`-Variante, die die Prüfungen überspringt.
## Konstanten
Konstante | Bedeutung
---|---
`SCHEMA_HASH_STRICT: u128` | Hash einschließlich der Namen. Siehe [Schema-Identität](/guides/schema-identity/).
`SCHEMA_HASH_LIGHT: u128` | Hash ohne Namen.
`SCHEMA_HASHES` | Beide, mit Algorithmus, Version und Vollständigkeit. Auch als `<T as qsl::SchemaIdentity>::SCHEMA_HASHES`.
`MAX_DATA_SIZE` | Das Budget, das geprüfte Methoden verwenden.
`NATIVE_BYTES`, `NATIVE_ALIGNMENT` | `size_of` und `align_of` des generierten Structs.
`WIRE_MAX_BYTES: Option<usize>` | Die maximale Frame-Größe oder `None`, wenn sie unbegrenzt ist.
`CACHE_BYTES` | Kapazität für `serialize_cached`. Nur mit `cache=N` oder `loyal_cache`.
`NATIVE_WIRE_COMPATIBLE` | Ob das native Layout dem Wire-Layout entspricht. Nur mit `transmutate`.
`QUANT_PADDING_BYTES` | Zusätzlich lesbare Bytes, die vorbereitete `qps`-Lesevorgänge brauchen. Derzeit `0`.
`REUSES_STORAGE` ist eine Konstante des Traits `Serializable`: `<T as qsl::Serializable>::REUSES_STORAGE` ist `true`, wenn `deserialize_reuse` Puffer an Ort und Stelle aktualisiert.

Der Generator wählt den Ausgabetyp anhand der bewiesenen maximalen Framegröße. Bis einschließlich STACK_LIMIT wird `StackOut<N>` verwendet; eine größere oder unbekannte Grenze wählt alloc::`Vec<u8>`. Übertragene Header zählen mit, die Metadaten des Ausgabeobjekts nicht.
