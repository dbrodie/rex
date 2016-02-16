use super::super::util::split_vec::SplitVec;

use super::test::Bencher;

fn create_large_vecs() -> Vec<Vec<u8>> {
    let mut v = vec![];
    for i in 0..100 {
        v.push(vec![1; 4*1024]);
    }
    v
}

#[bench]
fn bench_splitvec(b: &mut Bencher) {
    let mut sv = SplitVec::from_vecs(create_large_vecs());
    b.iter(move || {
        sv.splice(2000..4000, &vec![100; 4*1024]);
    });
}
