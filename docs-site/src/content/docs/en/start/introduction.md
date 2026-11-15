---
title: "Purpose and invariants"
description: "QSL documentation: Purpose and invariants."
---

:::note[Local edition]
The dev-qsl working tree on November 14, 2026. New APIs differ from the published qsl 0.3.0 crate.
:::

QSL exchanges explicitly described data between peers that control the schema, value bounds and memory contract. Build-time analysis produces concrete Rust types and specialized binary methods. The project develops as an independent implementation based on practical experience and a logical model.

Its scope includes internal protocols, telemetry, repeated records and applications with known data bounds. Specialization requires precise ranges, lengths, ownership, optional states and peer compatibility.

Invariants:

1. Additional attributes/mechanisms are optional; required fields must precede optional fields.
2. Values restore exactly within their declared domains.
3. An absent optional has no value payload and does not acquire an initializer/default.
4. Optimization never authorizes a read of unavailable bytes or uninitialized slots.
5. The schema and wire-affecting configuration define the format; physical Self layout is separate.
6. Checked APIs validate; explicit unsafe APIs retain caller obligations.

Version negotiation, routing, encryption, authentication and delivery policy belong to the application/transport. QBProto and an automatic schema-set handshake remain unimplemented.
