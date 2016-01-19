#[cfg(feature = "nightly")]

mod rex_bench {
    extern crate test;
    use self::test::Bencher;

    #[bench]
    fn bench_code_chunks(b: &mut Bencher) {
        b.iter(|| {
            test::black_box(vec![1, 2, 3]);
        });
    }

}
