use std::fmt::Display;

fn main() {
    baseline::run();
    btreemap::run();
    mapped_file::run();
}

#[derive(Debug, Clone, Copy)]
struct Stats {
    count: u64,
    min: f64,
    max: f64,
    total: f64,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            count: 0,
            min: f64::MAX,
            max: f64::MIN,
            total: 0.0,
        }
    }
}

impl Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let avg = self.total / (self.count as f64);
        write!(f, "{:.1}/{:.1}/{:.1}", self.min, avg, self.max)
    }
}

impl Stats {
    fn update(&mut self, val: f64) {
        self.min = self.min.min(val);
        self.max = self.max.max(val);
        self.count += 1;
        self.total += val;
    }

    fn merge(&mut self, other: &Self) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
        self.count += other.count;
        self.total += other.total;
    }
}
/// Improving on the baseline implementation by using `mmap` and
/// `HashMap` and map reduce on number of available cores.
///
/// This only works on Unix/Linux, on Windows use `VirtualAlloc`.
#[cfg(target_family = "unix")]
mod mapped_file {
    use crate::Stats;
    use std::collections::HashMap;
    use std::io::BufRead;
    use std::{fs::File, os::fd::AsRawFd};

    fn mmap(file: &File) -> &'static [u8] {
        extern "C" {
            fn mmap(
                addr: *mut u8,
                length: usize,
                prot: i32,
                flags: i32,
                fd: i32,
                offset: usize,
            ) -> *mut u8;
        }

        unsafe {
            // Allocate read only shared memory from a file descriptor.
            //
            // PROT_READ = 1
            // MAP_SHARED = 1
            let fd = file.as_raw_fd();
            let size = file.metadata().unwrap().len() as usize;
            let ret = mmap(0 as *mut u8, size, 1, 1, fd, 0);
            assert!(!ret.is_null());

            std::slice::from_raw_parts(ret, size)
        }
    }

    /// Offsets of chunks in the mapped file.
    #[derive(Debug, Clone, Copy)]
    struct ChunkOffsets(usize, usize);

    /// Virtually chunk the memory mapped file by calculating chunk offsets
    /// assuming `std::thread::available_parallelism()`.
    fn chunkify(mapped_file: &[u8]) -> Vec<ChunkOffsets> {
        let num_cores: usize = std::thread::available_parallelism()
            .expect("failed to estimate number of cores")
            .into();
        let chunk_size = mapped_file.len() / num_cores;
        let mut chunk_offsets = vec![];

        let mut start = 0;
        for _ in 0..num_cores {
            let end = (start + chunk_size).min(mapped_file.len());
            let mut window = mapped_file[end..].windows(1);
            let offset_to_newline = match window.position(|ch| ch == b"\n") {
                Some(v) => v,
                None => {
                    assert_eq!(end, mapped_file.len());
                    0
                }
            };
            let end = end + offset_to_newline;
            chunk_offsets.push(ChunkOffsets(start, end));
            start = end + 1;
        }
        chunk_offsets
    }

    pub fn run() {
        let f = File::open("measurements.txt").expect("Unable to open measurements file");
        let mapped_file = mmap(&f);
        let chunks = chunkify(mapped_file);
        let mut threads = vec![];

        for chunk in chunks {
            let start = chunk.0;
            let end = chunk.1;
            threads.push(std::thread::spawn(move || -> HashMap<String, Stats> {
                let mut local_stats: HashMap<String, Stats> = HashMap::with_capacity(512);
                let data = &mapped_file[start..end];

                for line in data.lines() {
                    if line.is_ok() {
                        let line = line.unwrap();
                        let line = line.split(";").collect::<Vec<_>>();
                        let city = line[0];
                        let temp = line[1].parse::<f64>().unwrap();

                        if let Some(stats) = local_stats.get_mut(city) {
                            stats.update(temp);
                        } else {
                            let mut stats = Stats::default();
                            stats.update(temp);
                            // Copy is needed to store in a hash map.
                            local_stats.insert(city.to_string(), stats);
                        }
                    }
                }

                local_stats
            }));
        }

        let mut global_stats: HashMap<String, Stats> = HashMap::with_capacity(512);
        let _ = threads
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .for_each(|local_stats| {
                for (k, v) in local_stats {
                    global_stats.entry(k).or_default().merge(&v);
                }
            });

        for (city, stats) in global_stats {
            println!("{city}={stats}");
        }
    }
}

/// Swapping `HashMap` for `BTreeMap`, yields worst performance which is predictable.
mod btreemap {
    use crate::Stats;
    use std::collections::BTreeMap;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    pub fn run() {
        let f = File::open("measurements.txt").expect("Unable to open measurements file");
        let reader = BufReader::new(f);

        let mut global_stats: BTreeMap<String, Stats> = BTreeMap::new();

        for line in reader.lines() {
            if line.is_ok() {
                let line = line.unwrap();
                let line: Vec<_> = line.split(";").collect();
                let city = line[0];
                let temp = line[1].parse::<f64>().unwrap();
                if let Some(stats) = global_stats.get_mut(city) {
                    stats.update(temp);
                } else {
                    let mut stats = Stats::default();
                    stats.update(temp);
                    // Copy is needed to store in a hash map.
                    global_stats.insert(city.to_string(), stats);
                }
            }
        }

        for (city, stats) in global_stats {
            println!("{city}={stats}");
        }
    }
}

/// Baseline implementation uses a naive approach reading line by line
/// and using a `HashMap`.
mod baseline {
    use crate::Stats;
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    pub fn run() {
        let f = File::open("measurements.txt").expect("Unable to open measurements file");
        let reader = BufReader::new(f);

        // Preallocate the hashmap.
        let mut global_stats: HashMap<String, Stats> = HashMap::with_capacity(512);

        for line in reader.lines() {
            if line.is_ok() {
                let line = line.unwrap();
                let line: Vec<_> = line.split(";").collect();
                let city = line[0];
                let temp = line[1].parse::<f64>().unwrap();
                if let Some(stats) = global_stats.get_mut(city) {
                    stats.update(temp);
                } else {
                    let mut stats = Stats::default();
                    stats.update(temp);
                    // Copy is needed to store in a hash map.
                    global_stats.insert(city.to_string(), stats);
                }
            }
        }

        for (city, stats) in global_stats {
            println!("{city}={stats}");
        }
    }
}
