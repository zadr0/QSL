# binary_opt: string ownership

[Architecture](../../architecture.en.md) · [safety](safety.md)

| Modifier | Generated Rust field | Storage |
|---|---|---|
| absent / false | String, or &str with lifetime | Normal schema ownership |
| true | &'a [u8] | Borrowed UTF-8 bytes; lifetime required |
| view | Utf8Guard<'a> | Borrowed string-like view; lifetime required |
| guard | Utf8CopyGuard | Copies bytes to owned/reusable SerOut |
| inline | InlineString<MAX> | Inline bounded capacity and length |

Fixed str(N), including equal bounds, always uses `[u8; N]`. Bounds count UTF-8 bytes, not Unicode characters. The modifier applies only to string/str fields.

```qsl
&Bytes[lifetime,safety] { str(0..4096) text|binary_opt=true|; }
&View[lifetime,safety] { str(0..4096) text|binary_opt=view|; }
&Copy[safety] { str(0..4096) text|binary_opt=guard|; }
&Inline[safety] { str(0..4096) text|binary_opt=inline|; }
```

CopyGuard avoids creating a String object but still needs storage. First creation can allocate; prepared reuse avoids repeated storage preparation when capacity suffices. It does not borrow the frame. Borrowed modes require the source to remain alive and immutable. Decompression needs a destination for restored bytes; unsupported borrowed/guard combinations are rejected. Checked decoding validates UTF-8; trusted APIs retain their preconditions.

Historical string measurements are not part of the current numeric QPS/control runs. No new claim about 1024-character latency is made here.
