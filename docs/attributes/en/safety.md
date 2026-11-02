# safety

[Current architecture and full contract](../../architecture.en.md).

Generates validate, validate_values, deserialize_checked and checked final-destination decoding. Validates before trusted decode. Does not inject checks into every unsafe call; custom codecs need a validator.
