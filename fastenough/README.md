# When does a hashmap out perform binary search (for `String`) ?

I was re-thinking the design of a symbol table in one of my compilers and got
to thinking about how `HashMap` compares to `Vec` in the case of lookups on
small strings (~128 bytes or less), unlike in the case of primitive types
which all end up in cache, `String` is a 16-byte fat pointer; thus each access
will need to read from main memory, flushing the previously read pointers on
each iteration so the impact of binary search behavior is more noticeable.

P.S: None of these benchmarks are representative, please run your own.

```sh

-- Hashmap lookup.

test tests::bench_lookup_100 ... bench:       2,587 ns/iter (+/- 12)
test tests::bench_lookup_1k  ... bench:      25,590 ns/iter (+/- 248)
test tests::bench_lookup_10k ... bench:     266,122 ns/iter (+/- 1,600)

-- Binary search.

test tests::bench_search_100 ... bench:       4,474 ns/iter (+/- 49)
test tests::bench_search_1k  ... bench:      84,827 ns/iter (+/- 715)
test tests::bench_search_10k ... bench:   1,524,677 ns/iter (+/- 2,113)

```
