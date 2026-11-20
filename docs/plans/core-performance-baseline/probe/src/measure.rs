use std::{cell::Cell, cmp::Ordering, hint::black_box, time::Instant};

thread_local! {
    static INDEX_EQUALITY_COMPARISONS: Cell<usize> = const { Cell::new(0) };
    static INDEX_ORDER_COMPARISONS: Cell<usize> = const { Cell::new(0) };
}

#[derive(Eq)]
pub struct CountedIndex(pub usize);

impl PartialEq for CountedIndex {
    fn eq(&self, other: &Self) -> bool {
        INDEX_EQUALITY_COMPARISONS.set(INDEX_EQUALITY_COMPARISONS.get() + 1);
        self.0 == other.0
    }
}

impl PartialOrd for CountedIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CountedIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        INDEX_ORDER_COMPARISONS.set(INDEX_ORDER_COMPARISONS.get() + 1);
        self.0.cmp(&other.0)
    }
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

    #[derive(Clone, Copy)]
    enum Tracking {
        Requests(Counts),
        Retained(isize),
    }

    thread_local! {
        static TRACKING: Cell<Option<Tracking>> = const { Cell::new(None) };
    }

    struct Allocator;

    fn request(bytes: usize, live_delta: isize) {
        let _ = TRACKING.try_with(|cell| match cell.get() {
            Some(Tracking::Requests(mut counts)) => {
                counts.requests += 1;
                counts.bytes += bytes;
                cell.set(Some(Tracking::Requests(counts)));
            }
            Some(Tracking::Retained(total)) => {
                cell.set(Some(Tracking::Retained(total + live_delta)));
            }
            None => {}
        });
    }

    fn release(bytes: usize) {
        let _ = TRACKING.try_with(|cell| {
            if let Some(Tracking::Retained(total)) = cell.get() {
                cell.set(Some(Tracking::Retained(total - bytes as isize)));
            }
        });
    }

    // Forward each request unchanged. The thread-local counter never allocates.
    unsafe impl GlobalAlloc for Allocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let ptr = unsafe { System.alloc(layout) };
            let delta = if ptr.is_null() {
                0
            } else {
                layout.size() as isize
            };
            request(layout.size(), delta);
            ptr
        }

        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            let ptr = unsafe { System.alloc_zeroed(layout) };
            let delta = if ptr.is_null() {
                0
            } else {
                layout.size() as isize
            };
            request(layout.size(), delta);
            ptr
        }

        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            release(layout.size());
            unsafe { System.dealloc(ptr, layout) }
        }

        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
            let ptr = unsafe { System.realloc(ptr, layout, size) };
            let delta = if ptr.is_null() {
                0
            } else {
                size as isize - layout.size() as isize
            };
            request(size, delta);
            ptr
        }
    }

    #[global_allocator]
    static ALLOCATOR: Allocator = Allocator;

    pub fn measure<T>(call: impl FnOnce() -> T) -> (usize, usize) {
        TRACKING.set(Some(Tracking::Requests(Counts::default())));
        drop(black_box(call()));
        let Some(Tracking::Requests(counts)) = TRACKING.replace(None) else {
            unreachable!("allocation measurement was interrupted");
        };
        (counts.requests, counts.bytes)
    }

    pub fn retained<T>(call: impl FnOnce() -> T) -> isize {
        TRACKING.set(Some(Tracking::Retained(0)));
        let result = black_box(call());
        let Some(Tracking::Retained(bytes)) = TRACKING.replace(None) else {
            unreachable!("retained allocation measurement was interrupted");
        };
        drop(result);
        bytes
    }

    #[cfg(test)]
    #[test]
    fn retained_bytes_follow_the_live_allocation_after_growth_and_shrink() {
        fn build() -> Vec<u8> {
            drop(black_box(Vec::<u8>::with_capacity(32)));
            let mut output = Vec::with_capacity(8);
            output.push(1);
            output.reserve_exact(63);
            output.shrink_to_fit();
            output
        }

        assert_eq!(retained(build), 1);
        let (_, requested_bytes) = measure(build);
        assert!(requested_bytes > 32 + 8 + 64);
        assert_eq!(retained(|| ()), 0);
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
    pub(crate) fn id(&self, api: &str) -> String {
        format!(
            "{},{},{},{},{},{}",
            self.system, self.workload, self.sites, self.factors, self.order, api
        )
    }
}

pub enum Mode {
    Time,
    Allocations,
    Retained,
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
            Mode::Retained => println!("{fields},retained_bytes"),
            Mode::Diagnostics => println!("{fields},metric,value"),
        }
    }

    pub fn measure<T>(&self, case: &Case, api: &str, mut call: impl FnMut() -> T) {
        self.measure_batches(
            case,
            api,
            65_536,
            |_| (),
            |&(), iterations| {
                for _ in 0..iterations {
                    drop(black_box(call()));
                }
            },
        );
    }

    pub fn measure_prepared<I, T>(
        &self,
        case: &Case,
        api: &str,
        prepare: impl Fn() -> I,
        call: impl Fn(&I) -> T,
    ) {
        // Bound the number of fresh inputs held at once. Build and drop them
        // outside the timing and allocation windows.
        self.measure_batches(
            case,
            api,
            256,
            |iterations| (0..iterations).map(|_| prepare()).collect::<Vec<_>>(),
            |inputs, _| {
                for input in inputs {
                    drop(black_box(call(input)));
                }
            },
        );
    }

    pub fn measure_construction<T>(&self, case: &Case, api: &str, mut build: impl FnMut() -> T) {
        if matches!(self.mode, Mode::Retained) {
            #[cfg(feature = "allocations")]
            {
                let id = case.id(api);
                if id.contains(&self.filter) {
                    drop(black_box(build()));
                    let bytes = allocations::retained(build);
                    println!("{id},{bytes}");
                }
            }
            #[cfg(not(feature = "allocations"))]
            unreachable!("validated mode");
        } else {
            self.measure(case, api, &mut build);
        }
    }

    fn measure_batches<I>(
        &self,
        case: &Case,
        api: &str,
        max_iterations: usize,
        mut prepare: impl FnMut(usize) -> I,
        mut call_batch: impl FnMut(&I, usize),
    ) {
        if matches!(self.mode, Mode::Diagnostics | Mode::Retained) {
            return;
        }
        let id = case.id(api);
        if !id.contains(&self.filter) {
            return;
        }
        call_batch(&prepare(1), 1);
        match self.mode {
            Mode::Time => {
                let (iterations, wall, cpu) = timings(max_iterations, |iterations| {
                    let inputs = prepare(iterations);
                    elapsed(|| call_batch(&inputs, iterations))
                });
                print!("{id},{iterations},{:.3},{:.3}", median(wall), median(cpu));
                for sample in wall.into_iter().chain(cpu) {
                    print!(",{sample:.3}");
                }
                println!();
            }
            Mode::Allocations => {
                #[cfg(feature = "allocations")]
                {
                    let input = prepare(1);
                    let (requests, bytes) = allocations::measure(|| call_batch(&input, 1));
                    println!("{id},{requests},{bytes}");
                }
                #[cfg(not(feature = "allocations"))]
                unreachable!("validated mode");
            }
            Mode::Diagnostics | Mode::Retained => unreachable!("handled before warmup"),
        }
    }

    pub fn diagnose(&self, case: &Case, api: &str, metric: &str, check: impl FnOnce() -> usize) {
        if let Some(id) = self.diagnostic_id(case, api) {
            println!("{id},{metric},{}", check());
        }
    }

    pub fn diagnose_index_comparisons<T>(&self, case: &Case, api: &str, call: impl FnOnce() -> T) {
        if let Some(id) = self.diagnostic_id(case, api) {
            INDEX_EQUALITY_COMPARISONS.set(0);
            INDEX_ORDER_COMPARISONS.set(0);
            drop(black_box(call()));
            println!(
                "{id},index_comparisons,{}",
                INDEX_EQUALITY_COMPARISONS.get()
            );
            println!(
                "{id},index_order_comparisons,{}",
                INDEX_ORDER_COMPARISONS.get()
            );
        }
    }

    fn diagnostic_id(&self, case: &Case, api: &str) -> Option<String> {
        if !matches!(self.mode, Mode::Diagnostics) {
            return None;
        }
        let id = case.id(api);
        id.contains(&self.filter).then_some(id)
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

fn elapsed(call: impl FnOnce()) -> (f64, u64) {
    let cpu_start = thread_cpu_ns();
    let wall_start = Instant::now();
    call();
    let wall_ns = wall_start.elapsed().as_secs_f64() * 1e9;
    (wall_ns, thread_cpu_ns() - cpu_start)
}

fn timings(
    max_iterations: usize,
    mut sample: impl FnMut(usize) -> (f64, u64),
) -> (usize, [f64; 7], [f64; 7]) {
    let mut iterations = 1;
    while sample(iterations).1 < 10_000_000 && iterations < max_iterations {
        iterations *= 2;
    }
    let mut wall = [0.0; 7];
    let mut cpu = [0.0; 7];
    for (w, c) in wall.iter_mut().zip(&mut cpu) {
        let (wall_ns, cpu_ns) = sample(iterations);
        *w = wall_ns / iterations as f64;
        *c = cpu_ns as f64 / iterations as f64;
    }
    (iterations, wall, cpu)
}
