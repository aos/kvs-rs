use criterion::{criterion_group, criterion_main, Criterion};
use kvs::{KvStore, KvsEngine};
use rand::distributions::Alphanumeric;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use tempfile::TempDir;

fn criterion_bench(c: &mut Criterion) {
    c.bench_function("kvs_write", |b| {
        let mut r = StdRng::seed_from_u64(42);
        let mut v: Vec<(String, String)> = Vec::with_capacity(100);
        for _ in 0..1 {
            let key_len = r.gen_range(0, 100_001);
            let val_len = r.gen_range(0, 100_001);
            let key: String = r
                .sample_iter(&Alphanumeric)
                .take(key_len)
                .map(char::from)
                .collect();
            let val: String = r
                .sample_iter(&Alphanumeric)
                .take(val_len)
                .map(char::from)
                .collect();
            v.push((key, val));
        }
        let temp_dir = TempDir::new().expect("unable to create temporary working directory");

        b.iter(|| {
            let mut store = KvStore::open(temp_dir.path()).expect("unable to open KvStore");
            for (key, val) in v.clone() {
                store.set(key, val).expect("Store should not fail");
            }
        });

        b.iter(|| {
            let mut store = KvStore::open(temp_dir.path()).expect("unable to open KvStore");
            for (key, val) in v.clone() {
                let store_val = store
                    .get(key)
                    .expect("Store should not fail")
                    .expect("value should be available");
                assert_eq!(val, store_val);
            }
        })
    });
}

criterion_group!(benches, criterion_bench);
criterion_main!(benches);
