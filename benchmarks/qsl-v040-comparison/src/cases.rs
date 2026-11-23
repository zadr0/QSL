#[macro_export]
macro_rules! visit_cases {
    ($callback:ident,$context:expr) => {
        $callback::<$crate::QNumDefault>($context, 0);
        $callback::<$crate::QNumCompact>($context, 0);
        $callback::<$crate::QNumAligned>($context, 0);
        $callback::<$crate::QNumPointing>($context, 0);
        $callback::<$crate::QNumBmi2>($context, 0);
        $callback::<$crate::PNumVar>($context, 0);
        $callback::<$crate::PNumFixed>($context, 0);
        $callback::<$crate::BNumVar>($context, 0);
        $callback::<$crate::BNumFixed>($context, 0);
        $callback::<$crate::BNumNarrow>($context, 0);
        $callback::<$crate::CNum>($context, 0);
        $callback::<$crate::CNumNarrow>($context, 0);
        $callback::<$crate::QOptDefault>($context, 0);
        $callback::<$crate::QOptTable>($context, 0);
        $callback::<$crate::QOptPredict>($context, 0);
        $callback::<$crate::QOptPredictTable>($context, 0);
        $callback::<$crate::QOptBitmap>($context, 0);
        $callback::<$crate::QOptBitmapTable>($context, 0);
        $callback::<$crate::POptionals>($context, 0);
        $callback::<$crate::BOptVar>($context, 0);
        $callback::<$crate::BOptFixed>($context, 0);
        $callback::<$crate::COptionals>($context, 0);
        for length in [32usize, 256, 1024] {
            $callback::<$crate::QTextHeap>($context, length);
            $callback::<$crate::QTextInline>($context, length);
            $callback::<$crate::QTextBorrow>($context, length);
            $callback::<$crate::PText>($context, length);
            $callback::<$crate::BTextVar>($context, length);
            $callback::<$crate::BTextFixed>($context, length);
            $callback::<$crate::BTextBorrowVar>($context, length);
            $callback::<$crate::BTextBorrowFixed>($context, length);
            $callback::<$crate::CText>($context, length);
            $callback::<$crate::CTextBorrow>($context, length);
        }
    };
}
pub fn verify<C: crate::Codec>(length: usize) {
    let mut scratch = vec![0; crate::CAPACITY];
    let mut buffers = crate::Buffers::new(&mut scratch);
    for i in 0..crate::FIXTURES {
        let mut fixture = crate::Fixture::new(i, length);
        if i == 0 {
            fixture.numbers = [0, 1, 4094, 4095]
        }
        let input = C::prepare(&fixture);
        let size = C::encode(&input, &mut buffers).unwrap();
        C::read(&buffers.out[..size], |actual| {
            assert_eq!(
                actual,
                fixture.view(C::KIND),
                "{} fixture {} length {}",
                C::NAME,
                i,
                length
            )
        })
        .unwrap();
    }
}
