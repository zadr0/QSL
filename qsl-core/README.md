# qsl-core

Shared schema contracts and portable runtime foundations for QSL. Errors, schema types, wire constants, bit and byte I/O, checked validation, UTF-8 containers and reusable output storage are owned here. This crate does not depend on qsl-lexer or qsl-interpreter, parse source, or emit Rust. The qsl facade keeps generated ::qsl paths stable. Unchecked APIs retain their documented caller obligations. MIT; Rust 1.98+.
