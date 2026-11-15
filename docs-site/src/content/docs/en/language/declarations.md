---
title: "Declarations and field order"
description: "QSL documentation: Declarations and field order."
---

A schema names a message and declares its fields in wire order. In the ordinary format, required fields have no tag; optional fields must follow them. The compiler assigns tags.

```qsl
&User[safety] {
    u64 id;
    str(1..100) name;
    u8? age;
}
```

`?` permits absence. An empty string differs from an absent string. Object initializers do not replace absent wire values.

Schema attributes follow the name in brackets; field modifiers affect that field. Additional mechanisms are optional; required → optional order is mandatory.

[Types](/language/types/) · [Builder arguments](/language/deq/) · [Attribute combinations](/reference/attributes/)
