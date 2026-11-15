---
title: "Limits and source map"
description: "QSL documentation: Limits and source map."
---
Limit | Meaning
---|---
MAX_DATA_SIZE, default 65535 | Input/restored frame, aggregate decoded-value and allocation/work guards; not exact process RSS
STACK_LIMIT, default 256 B | Strict inline string/array policy; impossible explicit storage is diagnosed
64 compiler-assigned field tags | Optional dispatch/bitmap domain
32 text array nesting levels | Parser complexity bound
Explicit unsafe methods | Caller must establish all data/memory/CPU/lifetime conditions
Unsupported option combinations | Build error, not an implicit new wire contract
LLVM backend / QBProto handshake | Future work
Exact serializer sizes are checked before allocation/writing. LZ capacity is an upper bound, so value/padding budgets and actual output size are checked separately. Native allocation guards use the Rust element size and bound zero-sized work; a tiny wire element cannot authorize a huge Vec allocation. These are data/storage guards, not exact allocator overhead accounting.

Sources: [schema/types](/source/qsl-lexer/src/types.rs), [domains](/source/qsl-core/src/domain.rs), [analysis/IR](/source/qsl-lexer/src/analysis/compilation.rs), [layout](/source/qsl-lexer/src/layout.rs), [register planner](/source/qsl-lexer/src/optimizer/registers.rs), [kernels](/source/qsl-core/src/optimizer/kernels.rs), [parser](/source/qsl-lexer/src/parser.rs), [Builder](/source/qsl-lexer/src/builder.rs), [Rust backend](/source/qsl-interpreter/src/interpreter.rs), [checked codegen](/source/qsl-interpreter/src/codegen_checked.rs), [validation](/source/qsl-core/src/validation.rs), [runtime buffers](/source/qsl-core/src/serialize.rs), [RT](/source/qsl-rt/src/lib.rs).
