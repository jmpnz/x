#![feature(test)]
extern crate test;
use std::collections::HashMap;

fn main() {
    println!("Answering the question of when does a HashMap beat binary search!");
}

#[allow(dead_code)]
fn random_str() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    const STR_LENGTH: usize = 55;
    let mut rng = rand::thread_rng();

    let str: String = (0..STR_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    str
}

#[allow(dead_code)]
#[inline]
fn find(v: &[String], s: &String) {
    assert!(v.binary_search(s).is_ok())
}

#[allow(dead_code)]
#[inline]
fn hashmap_find(v: &HashMap<String, usize>, s: &String) {
    assert!(v.get(s).is_some())
}

#[cfg(test)]
mod tests {
    use test::{black_box, Bencher};

    use super::*;

    macro_rules! bench_binary_search {
        ($name: ident, $size: expr) => {
            #[bench]
            fn $name(b: &mut Bencher) {
                let mut pool = Vec::with_capacity($size);
                let mut searches = Vec::with_capacity($size);

                for _ in 0..$size {
                    let s = random_str();
                    pool.push(s.clone());
                    searches.push(s);
                }

                pool.sort();

                b.iter(|| {
                    for s in &searches {
                        black_box(find(&pool, &s))
                    }
                });
            }
        };
    }

    macro_rules! bench_hashmap_lookup {
        ($name: ident, $size: expr) => {
            #[bench]
            fn $name(b: &mut Bencher) {
                let mut pool = HashMap::with_capacity($size);
                let mut searches = Vec::with_capacity($size);

                for pos in 0..$size {
                    let s = random_str();
                    pool.insert(s.clone(), pos);
                    searches.push(s);
                }

                b.iter(|| {
                    for s in &searches {
                        black_box(hashmap_find(&pool, &s))
                    }
                });
            }
        };
    }

    bench_binary_search!(bench_search_100, 100);
    bench_binary_search!(bench_search_1k, 1000);
    bench_binary_search!(bench_search_10k, 10000);

    bench_hashmap_lookup!(bench_lookup_100, 100);
    bench_hashmap_lookup!(bench_lookup_1k, 1000);
    bench_hashmap_lookup!(bench_lookup_10k, 10000);
}
