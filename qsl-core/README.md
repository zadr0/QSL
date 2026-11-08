# qsl-core

Shared schema contracts and portable runtime foundations for QSL binary codecs. This crate owns errors, schema types, generation settings, wire constants and analyses, schema hashes, bit/byte I/O, UTF-8 containers, checked validation and reusable output storage. It does not parse QSL source or generate Rust and has no dependency on qsl-lexer or qsl-interpreter.

Applications normally depend on qsl 0.3.0, which re-exports these same definitions and supplies the compiler facade and attribute macros. Frontend/backend integrations can share qsl-core without creating a compiler dependency cycle.

Variable UTF-8 storage is String, &str, `StackString<N>` or `InlineString<N>`. New/reuse are destination modes; borrowed text requires a live immutable input frame. Ordinary inline storage respects STACK_LIMIT (256 B by default). Checked APIs validate bounds, domains, UTF-8 and restoration budgets. Unchecked APIs require callers to satisfy the documented validity, capacity, alignment and lifetime obligations.

Requires Rust 1.98 or later. MIT licensed.

[English guide](https://github.com/zadr0/QSL/blob/v0.3.0/docs/guide.en.md) · [Architecture](https://github.com/zadr0/QSL/blob/v0.3.0/docs/architecture.en.md) · [Migration](https://github.com/zadr0/QSL/blob/v0.3.0/docs/migration-0.3.0.en.md).
