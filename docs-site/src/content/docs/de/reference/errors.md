---
title: Fehler
description: Die Varianten von QslError und wann welche zurückgegeben wird.
---

Generierte Methoden geben `qsl::QslResult<T>` zurück, einen Alias für `Result<T, QslError>`. `QslError` hat 11 Varianten und implementiert `std::error::Error`. Die `Display`-Ausgabe lautet `QSL error: <Variant>`, zum Beispiel `QSL error: InvalidUtf8`.
Variante | Wann sie zurückgegeben wird
---|---
`UnexpectedEof` | Die Eingabe endet vor dem Frame.
`BufferTooSmall` | Ein übergebener Puffer ist zu klein für die Ausgabe.
`LengthOverflow` | Eine Längenberechnung würde überlaufen.
`InvalidWidth` | Eine Breite oder ein Quantum wird nicht unterstützt.
`InvalidValue` | Ein Wert, eine Länge oder ein Metadatenfeld liegt außerhalb des erlaubten Bereichs.
`InvalidUtf8` | String-Bytes sind kein gültiges UTF-8.
`InvalidSchema` | Das Schema ist ungültig, oder die Operation ist darauf nicht anwendbar. Außerdem das Standardergebnis der `QslCustom`-Validatoren.
`InvalidPadding` | Padding-Bits oder -Bytes sind nicht null.
`PoolExhausted` | Alle RT-Slots sind belegt. Kommt nur vom nicht wartenden `try_acquire`.
`SizeLimitExceeded` | Frame oder dekodierte Daten überschreiten `MAX_DATA_SIZE`.
`AllocationFailed` | Eine Allokation mit Fehlerrückgabe ist fehlgeschlagen.
Datei- und Socket-Operationen können außerdem `std::io::Error` zurückgeben.

Eine erfolgreiche Validierung bedeutet, dass die Bytes zur Form des Schemas passen. Sie sagt nichts darüber, wer sie gesendet hat oder mit welchem Schema sie geschrieben wurden.
