use num_complex::Complex64 as C;
use std::{
    alloc::{GlobalAlloc, Layout, System},
    hint::black_box,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed},
    time::Instant,
};
use syntaqs_core::{Basis, HilbertSpace, LinearCombination, OperatorOf, State, StateOf};
use syntaqs_geometry::{Boundary, Chain};
use syntaqs_spin_half::{
    CartesianBasis, SpinHalf, SpinHalfOperator as O, SpinHalfState as S, SzBasis,
};
type Q = SpinHalf<C>;
type Models = syntaqs_spin_half::models::SpinHalfModels<Q>;
type Op = OperatorOf<Q, usize>;
type St = StateOf<Q, usize>;
static COUNT: AtomicBool = AtomicBool::new(false);
static ALLOCS: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);
struct Alloc;
unsafe impl GlobalAlloc for Alloc {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        if COUNT.load(Relaxed) {
            ALLOCS.fetch_add(1, Relaxed);
            BYTES.fetch_add(l.size(), Relaxed);
        }
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        if COUNT.load(Relaxed) {
            ALLOCS.fetch_add(1, Relaxed);
            BYTES.fetch_add(n, Relaxed);
        }
        unsafe { System.realloc(p, l, n) }
    }
}
#[global_allocator]
static GLOBAL: Alloc = Alloc;
fn bench<T>(name: &str, mut f: impl FnMut() -> T) {
    drop(black_box(f()));
    let mut times = Vec::new();
    for _ in 0..5 {
        let start = Instant::now();
        drop(black_box(f()));
        times.push(start.elapsed().as_secs_f64() * 1000.);
    }
    times.sort_by(f64::total_cmp);
    ALLOCS.store(0, Relaxed);
    BYTES.store(0, Relaxed);
    COUNT.store(true, Relaxed);
    drop(black_box(f()));
    COUNT.store(false, Relaxed);
    println!(
        "{name},median_ms={:.6},allocs={},allocated_bytes={}",
        times[2],
        ALLOCS.load(Relaxed),
        BYTES.load(Relaxed)
    );
}
fn model(n: usize) -> (Op, St, HilbertSpace<usize>, Vec<HilbertSpace<usize>>) {
    let sites: Vec<_> = (0..n).map(HilbertSpace::factor).collect();
    let chain = Chain::new(n, Boundary::Open);
    let h = Models::heisenberg(&chain, &sites, 1.).unwrap();
    let psi = Models::neel(&chain, &sites, S::Up).unwrap();
    (
        h,
        psi,
        HilbertSpace::tensor_all(sites.clone()).unwrap(),
        sites,
    )
}
static BASIS_CALLS: AtomicUsize = AtomicUsize::new(0);
static ID_CALLS: AtomicUsize = AtomicUsize::new(0);
struct CountBasis;
impl Basis<O, C> for CountBasis {
    fn expand(&self, p: &O) -> LinearCombination<O, C> {
        BASIS_CALLS.fetch_add(1, Relaxed);
        if *p == O::I {
            ID_CALLS.fetch_add(1, Relaxed);
        }
        <CartesianBasis as Basis<O, C>>::expand(&CartesianBasis, p)
    }
    fn index(&self, p: &O) -> Option<usize> {
        <CartesianBasis as Basis<O, C>>::index(&CartesianBasis, p)
    }
    fn elements(&self) -> impl Iterator<Item = O> + '_ {
        <CartesianBasis as Basis<O, C>>::elements(&CartesianBasis)
    }
}
fn original() {
    println!("single-spin rotation R=cos(0.1)I-i sin(0.1)Y");
    let r = Q::operator(O::I).scaled(C::new(0.1_f64.cos(), 0.))
        + Q::operator(O::Sy).scaled(C::new(0., -0.1_f64.sin()));
    let initial = Q::state(S::Up);
    for k in [8, 12, 16] {
        let mut psi = initial.clone();
        for _ in 0..k {
            psi = psi.act(&r);
        }
        let raw = psi.coordinates(&SzBasis).unwrap();
        let mut reduced = initial.clone();
        for _ in 0..k {
            reduced = reduced.act(&r).expand(&SzBasis).unwrap();
        }
        let short = reduced.coordinates(&SzBasis).unwrap();
        for ((a, c), (b, d)) in raw.entries().zip(short.entries()) {
            assert_eq!(a, b);
            assert!((*c - *d).norm() < 1e-10);
        }
        assert_eq!(raw.len(), 2);
        for (digits, c) in raw.entries() {
            let expected = if digits[0] == 0 {
                (0.1 * k as f64).cos()
            } else {
                (0.1 * k as f64).sin()
            };
            assert!((*c - C::new(expected, 0.)).norm() < 1e-10);
        }
        bench(&format!("rotation k={k} evaluate"), || psi.evaluate());
        bench(&format!("rotation k={k} coordinates"), || {
            psi.coordinates(&SzBasis).unwrap()
        });
        bench(&format!("rotation k={k} expand_each_step"), || {
            let mut psi = initial.clone();
            for _ in 0..k {
                psi = psi.act(&r).expand(&SzBasis).unwrap();
            }
            psi
        });
    }
    for n in [16, 32, 64, 128] {
        let (h, psi, space, sites) = model(n);
        bench(&format!("H n={n} evaluate_on_Neel"), || {
            psi.act(&h).unwrap().evaluate().unwrap()
        });
        bench(&format!("H n={n} coordinates"), || {
            h.coordinates(&CartesianBasis).unwrap()
        });
        BASIS_CALLS.store(0, Relaxed);
        ID_CALLS.store(0, Relaxed);
        let out = h.coordinates(&CountBasis).unwrap();
        println!(
            "H n={n} nnz={} basis_calls={} identity_calls={}",
            out.len(),
            BASIS_CALLS.load(Relaxed),
            ID_CALLS.load(Relaxed)
        );
        if n <= 64 {
            let hh = (h.clone() * &h).unwrap();
            bench(&format!("H2 n={n} coordinates"), || {
                hh.coordinates(&CartesianBasis).unwrap()
            });
            println!(
                "H2 n={n} nnz={}",
                hh.coordinates(&CartesianBasis).unwrap().len()
            );
        }
        // Keep product and coordinate construction outside the timing above.
        black_box((space, sites));
    }
    for k in [1, 2, 3, 4] {
        let (h, mut psi, _, _) = model(6);
        for _ in 0..k {
            psi = psi.act(&h).unwrap();
        }
        bench(&format!("Hpow Neel n=6 k={k} evaluate"), || {
            psi.evaluate().unwrap()
        });
        bench(&format!("Hpow Neel n=6 k={k} coordinates"), || {
            psi.coordinates(&SzBasis).unwrap()
        });
        println!(
            "Hpow n=6 k={k} nnz={}",
            psi.coordinates(&SzBasis).unwrap().len()
        );
    }
    for n in [10, 14, 18] {
        let sites: Vec<_> = (0..n).map(HilbertSpace::factor).collect();
        let space = HilbertSpace::tensor_all(sites.clone()).unwrap();
        let plus = (Q::state(S::Up) + Q::state(S::Down)).scaled(C::new(1. / 2_f64.sqrt(), 0.));
        let psi = State::tensor_in(&space, sites.iter().map(|s| plus.at(s).unwrap())).unwrap();
        assert!((psi.norm_squared().unwrap() - 1.).abs() < 1e-12);
        bench(&format!("plus n={n} norm_squared"), || {
            psi.norm_squared().unwrap()
        });
        bench(&format!("plus n={n} coordinates"), || {
            psi.coordinates(&SzBasis).unwrap()
        });
        if n <= 14 {
            bench(&format!("plus n={n} expand"), || {
                psi.expand(&SzBasis).unwrap()
            });
        }
        println!(
            "plus n={n} nnz={}",
            psi.coordinates(&SzBasis).unwrap().len()
        );
    }
}

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_default();
    if mode == "count" {
        count_w();
        return;
    }
    if mode == "h2-count" {
        let (h, _, _, _) = model(64);
        let hh = (h.clone() * &h).unwrap();
        let c = hh.coordinates(&CountBasis).unwrap();
        println!(
            "H2 n=64 nnz={} basis_calls={} identity_calls={}",
            c.len(),
            BASIS_CALLS.load(Relaxed),
            ID_CALLS.load(Relaxed)
        );
        return;
    }
    if mode != "extra" {
        original();
        return;
    }
    for n in [4, 6, 8] {
        let sites: Vec<_> = (0..n).map(HilbertSpace::factor).collect();
        let space = HilbertSpace::tensor_all(sites.clone()).unwrap();
        let gate = Q::operator(O::Sx) + Q::operator(O::Sz);
        let local = Op::product(sites.iter().map(|s| gate.at(s).unwrap()))
            .unwrap()
            .bind(&space)
            .unwrap();
        let placed = Op::product(sites.iter().map(|s| {
            (Q::operator(O::Sx).at(s).unwrap() + Q::operator(O::Sz).at(s).unwrap()).unwrap()
        }))
        .unwrap()
        .bind(&space)
        .unwrap();
        let local = (local.adjoint() * &local)
            .unwrap()
            .scaled(C::new(0.5_f64.powi(n as i32), 0.));
        let placed = (placed.adjoint() * &placed)
            .unwrap()
            .scaled(C::new(0.5_f64.powi(n as i32), 0.));
        let a = local.coordinates(&CartesianBasis).unwrap();
        let b = placed.coordinates(&CartesianBasis).unwrap();
        assert_eq!(a.len(), 1);
        assert_eq!(b.len(), 1);
        assert_eq!(*a.entries().next().unwrap().1, C::new(1., 0.));
        assert_eq!(*b.entries().next().unwrap().1, C::new(1., 0.));
        bench(
            &format!("Hadamard_layer_dagger_layer n={n} local_sum"),
            || local.coordinates(&CartesianBasis).unwrap(),
        );
        bench(
            &format!("Hadamard_layer_dagger_layer n={n} placed_sum"),
            || placed.coordinates(&CartesianBasis).unwrap(),
        );
    }
    for k in [2, 3, 4] {
        let (h, initial, _, _) = model(6);
        let mut raw = initial.clone();
        for _ in 0..k {
            raw = raw.act(&h).unwrap();
        }
        let raw = raw.coordinates(&SzBasis).unwrap();
        let mut reduced = initial.clone();
        for _ in 0..k {
            reduced = reduced.act(&h).unwrap().expand(&SzBasis).unwrap();
        }
        let short = reduced.coordinates(&SzBasis).unwrap();
        assert_eq!(raw.len(), short.len());
        for ((a, c), (b, d)) in raw.entries().zip(short.entries()) {
            assert_eq!(a, b);
            assert_eq!(c, d);
        }
        bench(&format!("Hpow Neel n=6 k={k} expand_each_step"), || {
            let mut psi = initial.clone();
            for _ in 0..k {
                psi = psi.act(&h).unwrap().expand(&SzBasis).unwrap();
            }
            psi
        });
    }
    for n in [8, 16, 32, 64] {
        let sites: Vec<_> = (0..n).map(HilbertSpace::factor).collect();
        let space = HilbertSpace::tensor_all(sites.clone()).unwrap();
        let w = State::sum(
            &space,
            (0..n).map(|i| {
                State::tensor_in(
                    &space,
                    sites.iter().enumerate().map(|(j, s)| {
                        Q::state(if i == j { S::Down } else { S::Up })
                            .at(s)
                            .unwrap()
                    }),
                )
                .unwrap()
            }),
        )
        .unwrap()
        .scaled(C::new(1. / (n as f64).sqrt(), 0.));
        assert!((w.norm_squared().unwrap() - 1.).abs() < 1e-12);
        bench(&format!("W n={n} norm_squared"), || {
            w.norm_squared().unwrap()
        });
        bench(&format!("W n={n} coordinates"), || {
            w.coordinates(&SzBasis).unwrap()
        });
        let coordinates = w.coordinates(&SzBasis).unwrap();
        assert_eq!(coordinates.len(), n);
        bench(&format!("W n={n} coordinate_norm_only"), || {
            coordinates
                .entries()
                .map(|(_, v)| v.norm_sqr())
                .sum::<f64>()
        });
    }
}

static INNER_CALLS: AtomicUsize = AtomicUsize::new(0);
struct CountState(S);
impl syntaqs_core::InnerProduct<C> for CountState {
    fn inner_product(&self, rhs: &Self) -> C {
        INNER_CALLS.fetch_add(1, Relaxed);
        <S as syntaqs_core::InnerProduct<C>>::inner_product(&self.0, &rhs.0)
    }
}
impl syntaqs_core::Act<C> for CountState {
    type Operator = O;
    type Term = Self;
    fn act(&self, op: &O) -> LinearCombination<Self, C> {
        <S as syntaqs_core::Act<C>>::act(&self.0, op).map(CountState)
    }
}
fn count_w() {
    type CQ = syntaqs_core::ExpressionFactory<CountState, O, C>;
    for n in [8, 16, 32, 64] {
        let sites: Vec<_> = (0..n).map(HilbertSpace::factor).collect();
        let space = HilbertSpace::tensor_all(sites.clone()).unwrap();
        let w = State::sum(
            &space,
            (0..n).map(|i| {
                State::tensor_in(
                    &space,
                    sites.iter().enumerate().map(|(j, s)| {
                        CQ::state(CountState(if i == j { S::Down } else { S::Up }))
                            .at(s)
                            .unwrap()
                    }),
                )
                .unwrap()
            }),
        )
        .unwrap()
        .scaled(C::new(1. / (n as f64).sqrt(), 0.));
        INNER_CALLS.store(0, Relaxed);
        assert!((w.norm_squared().unwrap() - 1.).abs() < 1e-12);
        println!(
            "W n={n} local_inner_product_calls={}",
            INNER_CALLS.load(Relaxed)
        );
        assert_eq!(INNER_CALLS.load(Relaxed), n * n * n);
    }
}
