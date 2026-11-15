# binary_opt

Модификатор применяется только к string/str. Поддерживаются false (обычный String), inline (InlineString<N>) и view (&str). По умолчанию выбирается String; aggressive может выбрать StackString<N> для конечного диапазона в STACK_LIMIT. guard и true больше не принимаются.

```qsl
&Text[safety] {
    str(1..100) name|binary_opt=view|;
    str(1..100) owned;
}
```

Границы заданы в байтах UTF-8. &str требует времени жизни исходного кадра. Сжатую переменную строку нельзя выдавать как заимствованный &str: для восстановления нужна память назначения. Fixed native str(N) сохраняет [u8; N]. New/reuse выбирает метод чтения и не является значением binary_opt.
