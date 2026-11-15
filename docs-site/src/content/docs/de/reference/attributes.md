---
title: Attribute
description: Alle Typattribute und Feldmodifikatoren von QSL, was sie generieren und ob sie das Wire-Format ändern.
---

Typattribute stehen in eckigen Klammern nach dem Namen: `&Packet[safety,compact]`. Feldmodifikatoren stehen zwischen senkrechten Strichen nach dem Feldnamen: `str(1..100) name;`. Alle sind optional, und jedes darf nur einmal vorkommen.

**Format** gibt an, ob das Attribut die Bytes ändert und damit den [Schema-Hash](/guides/schema-identity/). Wenn ja, müssen beide Seiten dieselbe Einstellung verwenden.

## Übersicht
Attribut | Wo | Format | Wirkung
---|---|---|---
[`safety`](#safety) | Typ | nein | Generiert geprüfte Decode-Methoden.
[`compact`](#compact) | Typ | ja | Speichert Werte mit Bereich in der minimalen Anzahl Bits.
[`alignment`](#alignment) | Typ | mit `compact` | Lässt jedes kompakte Feld an einer Bytegrenze beginnen.
[`optionals=bitmap`](#optionalsbitmap) | Typ | ja | Präsenzmaske über die optionalen Felder.
[`predict_seq`](#predict_seq) | Typ | ja | Vorhandensein aus der Länge der Nutzdaten abgeleitet.
[`opt_table`](#opt_table) | Typ | nur `legacy` | Statische Tabelle der Reader für optionale Felder.
[`qps`, `quant`](#qps-quant) | Typ | ja | Umkehrbare Packung numerischer Felder.
[`aggressive`](#aggressive) | Typ | nein | Inline-Speicher für begrenzte Strings und Arrays.
[`lifetime`](#lifetime) | Typ | nein | Leiht Strings und Bytes aus der Eingabe aus.
[`transmutate`](#transmutate) | Typ | nein | Natives Layout gleich dem Wire-Layout.
[`inline`](#inline) | Typ | nein | `#[inline]`-Regel für generierte Methoden.
[`slots=N`](#slotsn) | Typ | nein | Anzahl der RT-Slots für diesen Typ.
[`cache=N`, `loyal_cache`](#cachen-loyal_cache) | Typ | nein | Ausgabe in einen `CachedFrame` des Aufrufers.
[`state_bits=Qn`](#state_bitsqn) | Typ | ja | Legacy-Format für den Präsenzindex.
[`decompress=true`](#decompresstrue) | Feld | ja | Begrenzte LZ-Kompression für einen String.
[`default=N`](#defaultn) | Feld | nein | Generiert `impl Default`.
## Typattribute

### safety {#safety}

Generiert `deserialize_checked` und `deserialize_checked_into`, die nicht vertrauenswürdige Eingaben dekodieren. Sie prüfen Grenzen, Wertebereiche, UTF-8, die Metadaten optionaler Felder, Padding, Dekompression und das Gesamtbudget `MAX_DATA_SIZE`, bevor sie einen Wert zurückgeben. Die ungeprüften Methoden bleiben verfügbar. `serialize_checked`, `serialize_checked_to`, `validate` und `validate_values` werden mit und ohne `safety` generiert. Siehe [Eingaben validieren](/guides/safety/).

### compact {#compact}

Speichert jeden Integer mit Wertebereich als `x - min` in `bit_length(max - min)` Bits, gepackt über Bytegrenzen hinweg. `u8(0..7)`, `u8(0..31)` und `u16(0..1023)` belegen 3, 5 und 10 Bits. String- und Array-Längen werden innerhalb ihrer Grenzen genauso gespeichert. Funktioniert mit `opt_table` und `optionals=bitmap`. Nicht kombinierbar mit `predict_seq`, `state_bits` oder `qps`. Siehe [Kleinere Frames](/guides/smaller-frames/).

### alignment {#alignment}

Lässt mit `compact` jedes Feld an einer Bytegrenze beginnen. Der Frame kann dadurch größer werden. Ohne `compact` ändert es das Format nicht. Gemeint ist nicht die Speicherausrichtung von Puffern. Nicht kombinierbar mit `qps`.

### optionals=bitmap {#optionalsbitmap}

Schreibt eine Maske mit einem Bit pro optionalem Feld, je nach Anzahl 0 bis 16 Bytes breit. Unterstützt bis zu 128 optionale Felder. Inkompatibel mit der Maske, die vor 0.2.0 verwendet wurde. Nicht kombinierbar mit `state_bits`, `opt_table=legacy` oder `qps`. Siehe [Optionale Felder](/language/optionals/#bitmap).

### predict_seq {#predict_seq}

Leitet aus der Länge der Nutzdaten ab, welche optionalen Felder vorhanden sind. Setzt voraus, dass jede Kombination vorhandener Felder eine eigene Länge ergibt. Wird ignoriert, wenn `optionals=bitmap` gesetzt ist. Nicht kombinierbar mit `compact`. Siehe [Optionale Felder](/language/optionals/#length-index-predict_seq).

### opt_table {#opt_table}

`opt_table` (oder `opt_table=static`) ruft die Reader für optionale Felder über eine statische Tabelle statt über ein `match` auf. Gleiche Bytes, gleicher Hash. Zusammen mit `predict_seq` wählt es das Längengraph-Format, das `[opt_table]` vor 0.2.0 erzeugt hat.

`opt_table=legacy` ist ein anderes Format. Nach den Pflichtfeldern schreibt es ein Byte mit der Anzahl und ein Little-Endian-`u32` pro optionalem Feld: `0` für ein fehlendes Feld, sonst den Offset des Werts in den optionalen Nutzdaten plus 1. Es lässt sich nicht mit einfachem `opt_table`, `predict_seq`, `state_bits`, `optionals=bitmap` oder `qps` kombinieren.

### qps, quant {#qps-quant}

`qps` bedeutet `qps=Q16`. `qps=Q2`, `Q4`, `Q8` und `Q16` legen das Quantum fest. `quant=Qn` ist ein Alias. `quant` ohne Wert ist ein Fehler, ebenso die Verwendung beider Namen. Packt nur numerische Felder: Strings, Arrays und ihre Längen behalten ihre Codecs, und numerische Felder müssen vor ihnen stehen. Nicht kombinierbar mit `compact`, `alignment`, `optionals=bitmap` oder `opt_table=legacy`.

Ohne RT-Pool bekommt ein Typ mit aktiver Packung ein sicheres `serialize()`, das `QslResult<QuantizedOut>` zurückgibt. Siehe [Kleinere Frames](/guides/smaller-frames/).

### aggressive {#aggressive}

Ändert die Rust-Speicherform, nicht die Bytes und nicht, welche Eingaben akzeptiert werden:

- `str(min..max)` wird zu `StackString<max>` und `array<T>(min..max)` zu `InlineArray<T, max>`, wenn das Maximum in `STACK_LIMIT` passt. Größere Felder bleiben `String` bzw. `Vec`.
- Ein optionales numerisches Feld wird als `NonZero`-Integer der kleinsten Breite gespeichert, die seinen Bereich fasst, sofern diese nicht breiter ist als der Typ des Felds.

### lifetime {#lifetime}

Gibt dem Typ eine Lifetime `'a` und leiht aus der Eingabe aus: Strings werden zu `&'a str` und `array<u8>` zu `&'a [u8]`. Die Eingabe muss länger leben als der Wert. Arrays mit anderen Elementtypen und komprimierte Strings lassen sich nicht ausleihen und führen zu einem Build-Fehler, ebenso ein Typ mit `lifetime`, der nichts auszuleihen hat.

### transmutate {#transmutate}

Macht das native Layout des Typs gleich seinem Wire-Layout. Jedes Feld muss eine feste Größe haben: `compact`, `qps`, optionale Felder sowie Strings oder Arrays mit Bereich sind Build-Fehler. Die Frame-Größe muss der Größe des Structs entsprechen. Füge deshalb `padding(N)`-Felder dort ein, wo Rust Padding einfügt.

Ohne `lifetime` kopiert `deserialize_transmutate_copy` den Frame in einem Block in einen eigenständigen Wert, wenn `NATIVE_WIRE_COMPATIBLE` wahr ist, und dekodiert sonst Feld für Feld. Mit `[safety]` ist die Methode sicher und validiert das Ergebnis. Ohne `[safety]` ist sie `unsafe`.

Mit `lifetime` gibt `deserialize_overlay` eine Referenz `&Self` zurück, die in die Eingabe zeigt. Der Build prüft, dass jeder Offset im Frame dem nativen Offset entspricht, und schlägt auf Big-Endian-Zielen fehl. Mit `[safety]` prüft `deserialize_overlay` die Konstante `NATIVE_WIRE_COMPATIBLE`, Länge und Ausrichtung der Eingabe und danach die Werte, und `deserialize_checked` tut dasselbe. Ohne `[safety]` ist `deserialize_overlay` `unsafe` und prüft nichts.

Beide haben `_unchecked`-Varianten, die die Prüfungen überspringen.

### inline {#inline}

`inline` (gleichbedeutend mit `inline=hint`), `inline=always` oder `inline=never` setzt das entsprechende Attribut an die generierten Methoden. Ohne diese Angabe tragen generierte Methoden kein Inline-Attribut, und der Compiler entscheidet. `always` kann den Code vergrößern; ein Geschwindigkeitsgewinn ist nicht garantiert.

### slots=N {#slotsn}

Überschreibt die Anzahl der RT-Slots für diesen Typ, 1 bis 64. Wirkt nur mit `!config QSL_RT = true;`. Siehe [RT-Slot-Pool](/guides/memory/#rt-slot-pool).

### cache=N, loyal_cache {#cachen-loyal_cache}

Generiert `CACHE_BYTES` und `serialize_cached`, das in einen vom Aufrufer bereitgestellten `qsl::modernization::CachedFrame<N>` schreibt. `loyal_cache` setzt `CACHE_BYTES` auf die maximale Frame-Größe. Der Typ braucht eine endliche maximale Frame-Größe; ein kleineres `N` führt zu einem Build-Fehler. Das bereitet Ausgabespeicher vor, cacht aber keine Werte. Siehe [Gecachte Ausgabe-Frames](/guides/memory/#cached-output-frames).

### state_bits=Qn {#state_bitsqn}

`Q1` (Standard), `Q2`, `Q4`, `Q8` oder `Q16`. Wählt ein Legacy-Format für den Präsenzindex. Es ist kein Kompressionsmodus. Nicht kombinierbar mit `optionals=bitmap`, `compact` oder `opt_table=legacy`.

## Feldmodifikatoren

### binary_opt {#binary_opt}

binary_opt wurde vollständig entfernt. Nutzen Sie [lifetime] für geliehenes &str, [aggressive] für begrenztes StackString und decompress=true für InlineString. STACK_LIMIT gilt für alle Strings und Arrays.

### decompress=true {#decompresstrue}

Kodiert den String mit dem begrenzten LZ-Codec von QSL. Strings unter 32 Bytes und Strings, die durch Kompression nicht kürzer würden, werden roh gespeichert. Jedes dekodierte Token wird gegen die wiederhergestellte Größe, die Kapazität des Ziels und `MAX_DATA_SIZE` geprüft. Der String braucht eine Obergrenze von höchstens 65535 Bytes. Ein String mit Bereich wird zu `InlineString<max>`, oberhalb von `STACK_LIMIT` zu `String`. Nicht kombinierbar mit `lifetime` oder `lifetime`. Siehe [Komprimierte Strings](/language/strings/#compressed-strings).

### default=N {#defaultn}

Generiert `impl Default` für den Typ. Dieses Feld startet mit `N` bzw. mit `Some(N)`, wenn es optional ist. Andere Felder starten mit ihrem kleinsten gültigen Wert. `N` muss im Wertebereich des Felds liegen, und das Feld muss numerisch sein. Hat der Typ zusätzlich ein Feld mit eigenem Typ, schlägt der Build fehl. Bytes und Hash ändern sich nicht.
