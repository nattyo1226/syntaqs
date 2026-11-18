use std::{cell::Cell, hint::black_box, time::Instant};

thread_local! {
    static INDEX_COMPARISONS: Cell<usize> = const { Cell::new(0) };
}

#[derive(Eq)]
pub struct CountedIndex(pub usize);

impl PartialEq for CountedIndex {
    fn eq(&self, other: &Self) -> bool {
        INDEX_COMPARISONS.set(INDEX_COMPARISONS.get() + 1);
        self.0 == other.0
    }
}

pub fn count_comparisons<T>(call: impl FnOnce() -> T) -> usize {
    INDEX_COMPARISONS.set(0);
    drop(black_box(call()));
    INDEX_COMPARISONS.get()
}

#[cfg(feature = "allocations")]
mod allocations {
    use std::{
        alloc::{GlobalAlloc, Layout, System},
        cell::Cell,
        hint::black_box,
    };

    #[derive(Clone, Copy, Default)]
    struct Counts {
        requests: usize,
        bytes: usize,
    }

    thread_local! {
        static COUNTS: Cell<Option<Counts>> = const { Cell::new(None) };
    }

    struct Allocator;

    fn count(bytes: usize) {
        let _ = COUNTS.try_with(|cell| {
            if let Some(mut counts) = cell.get() {
                counts.requests += 1;
                counts.bytes += bytes;
                cell.set(Some(counts));
            }
        });
    }

    // Forward each request unchanged. The thread-local counter never allocates.
    unsafe impl GlobalAlloc for Allocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            count(layout.size());
            unsafe { System.alloc(layout) }
        }

        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            count(layout.size());
            unsafe { System.alloc_zeroed(layout) }
        }

        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            unsafe { System.dealloc(ptr, layout) }
        }

        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
            count(size);
            unsafe { System.realloc(ptr, layout, size) }
        }
    }

    #[global_allocator]
    static ALLOCATOR: Allocator = Allocator;

    pub fn measure<T>(call: impl FnOnce() -> T) -> (usize, usize) {
        COUNTS.set(Some(Counts::default()));
        drop(black_box(call()));
        let counts = COUNTS.replace(None).unwrap();
        (counts.requests, counts.bytes)
    }
}

pub struct Case {
    pub system: &'static str,
    pub workload: &'static str,
    pub sites: usize,
    pub factors: usize,
    pub order: usize,
}

impl Case {
    fn id(&self, api: &str) -> String {
        format!(
            "{},{},{},{},{},{}",
            self.system, self.workload, self.sites, self.factors, self.order, api
        )
    }
}

pub enum Mode {
    Time,
    Allocations,
    Diagnostics,
}

pub struct Runner {
    pub mode: Mode,
    pub filter: String,
}

impl Runner {
    pub fn print_header(&self) {
        let fields = "system,workload,sites,factors,order,api";
        match self.mode {
            Mode::Time => {
                print!("{fields},iterations,median_wall_ns,median_cpu_ns");
                for clock in ["wall", "cpu"] {
                    for i in 1..=7 {
                        print!(",{clock}_{i}_ns");
                    }
                }
                println!();
            }
            Mode::Allocations => println!("{fields},requests,requested_bytes"),
            Mode::Diagnostics => println!("{fields},metric,value"),
        }
    }

    pub fn measure<T>(&self, case: &Case, api: &str, mut call: impl FnMut() -> T) {
        if matches!(self.mode, Mode::Diagnostics) {
            return;
        }
        let id = case.id(api);
        if !id.contains(&self.filter) {
            return;
        }
        drop(black_box(call()));
        match self.mode {
            Mode::Time => {
                let (iterations, wall, cpu) = timings(&mut call);
                print!("{id},{iterations},{:.3},{:.3}", median(wall), median(cpu));
                for sample in wall.into_iter().chain(cpu) {
                    print!(",{sample:.3}");
                }
                println!();
            }
            Mode::Allocations => {
                #[cfg(feature = "allocations")]
                {
                    let (requests, bytes) = allocations::measure(call);
                    println!("{id},{requests},{bytes}");
                }
                #[cfg(not(feature = "allocations"))]
                unreachable!("validated mode");
            }
            Mode::Diagnostics => unreachable!("handled before warmup"),
        }
    }

    pub fn diagnose(&self, case: &Case, api: &str, metric: &str, check: impl FnOnce() -> usize) {
        if matches!(self.mode, Mode::Diagnostics) {
            let id = case.id(api);
            if id.contains(&self.filter) {
                println!("{id},{metric},{}", check());
            }
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn thread_cpu_ns() -> u64 {
    let mut timestamp = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // The function fills the initialized timespec through its valid pointer.
    let status = unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut timestamp) };
    assert_eq!(status, 0, "cannot read the thread CPU clock");
    timestamp.tv_sec as u64 * 1_000_000_000 + timestamp.tv_nsec as u64
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn thread_cpu_ns() -> u64 {
    panic!("timings require the thread CPU clock on macOS or Linux");
}

fn median(mut samples: [f64; 7]) -> f64 {
    samples.sort_by(f64::total_cmp);
    samples[3]
}

fn timings<T>(call: &mut impl FnMut() -> T) -> (usize, [f64; 7], [f64; 7]) {
    fn elapsed<T>(call: &mut impl FnMut() -> T, iterations: usize) -> (f64, u64) {
        let cpu_start = thread_cpu_ns();
        let wall_start = Instant::now();
        for _ in 0..iterations {
            drop(black_box(call()));
        }
        let wall_ns = wall_start.elapsed().as_secs_f64() * 1e9;
        (wall_ns, thread_cpu_ns() - cpu_start)
    }

    let mut iterations = 1;
    while elapsed(call, iterations).1 < 10_000_000 && iterations < 65_536 {
        iterations *= 2;
    }
    let mut wall = [0.0; 7];
    let mut cpu = [0.0; 7];
    for (w, c) in wall.iter_mut().zip(&mut cpu) {
        let (wall_ns, cpu_ns) = elapsed(call, iterations);
        *w = wall_ns / iterations as f64;
        *c = cpu_ns as f64 / iterations as f64;
    }
    (iterations, wall, cpu)
}
