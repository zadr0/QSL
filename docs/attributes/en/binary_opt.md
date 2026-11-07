# binary_opt

This modifier applies to string/str only. Supported values are false (ordinary String), inline (InlineString<N>) and view (&str). Default storage is String; aggressive can select StackString<N> for a finite range within STACK_LIMIT. guard and true are no longer accepted.

```qsl
&Text[safety] {
    str(1..100) name|binary_opt=view|;
    str(1..100) owned;
}
```

Bounds count UTF-8 bytes. &str requires its source frame to remain alive. A compressed variable string cannot be returned as a borrowed &str because restoration requires destination storage. Fixed native str(N) retains [u8; N]. New/reuse selects a decoding method and is not a binary_opt value.
