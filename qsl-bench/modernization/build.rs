fn main() {
    let schemas=qsl::parse_qsl("&Text[inline,safety]{str(0..2048) text;} &StackText[inline,safety,aggressive]{str(0..2048) text;} &Raw[inline,safety]{u64(0..1000) id;i64(-100..100) signed;u64(0..65535) value;} &Packed[inline,qps,safety]{u64(0..1000) id;i64(-100..100) signed;u64(0..65535) value;}").unwrap();
    let mut g = qsl::QslInterpreter::new();
    g.config.stack_limit = 2048;
    g.generate(&schemas).unwrap();
    std::fs::write(
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("qsl_gen.rs"),
        g.output,
    )
    .unwrap();
}
