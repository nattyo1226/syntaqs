//! Regression controls: time normal inputs and observe primitive rule calls.
use super::*;
use syntaqs_core::{Adjoint, HomogeneousParity, Identity, Parity, ParityOperator, Product, Trace};

fn layer(sites: &[HilbertSpace<usize>], placed_sum: bool) -> Op {
    let gate = Q::operator(O::Sx) + Q::operator(O::Sz);
    Op::product(sites.iter().map(|site| {
        if placed_sum {
            (Q::operator(O::Sx).at(site).unwrap() + Q::operator(O::Sz).at(site).unwrap()).unwrap()
        } else {
            gate.at(site).unwrap()
        }
    }))
    .unwrap()
}

pub(super) fn run() {
    bench("measurement empty", || black_box(1usize));
    for n in [8, 32, 128] {
        let sites: Vec<_> = (0..n).map(HilbertSpace::factor).collect();
        let space = HilbertSpace::tensor_all(sites.clone()).unwrap();
        for placed in [false, true] {
            bench(&format!("construction n={n} placed_sum={placed}"), || {
                layer(black_box(&sites), placed).bind(&space).unwrap()
            });
        }
    }
    let (h, psi, space, sites) = model(64);
    let ready = psi.evaluate().unwrap();
    bench("ready Neel n=64 evaluate", || {
        black_box(&psi).evaluate().unwrap()
    });
    bench("evaluated Neel n=64 evaluate_again", || {
        black_box(&ready).evaluate().unwrap()
    });
    let single = Q::operator(O::Sx).at(&sites[32]).unwrap();
    bench("Neel n=64 one_site_action", || {
        psi.act(&single).unwrap().evaluate().unwrap()
    });
    let duplicate = syntaqs_core::LocalState::sum((0..128).map(|_| Q::state(S::Up)));
    assert_eq!(duplicate.norm_squared(), 128_f64.powi(2));
    bench("ready local duplicate_sum n=128 evaluate", || {
        black_box(&duplicate).evaluate()
    });
    // The state is intentionally retained; this case measures one local action.
    black_box((h, space));

    for n in [16, 32] {
        let (h, _, _, _) = model(n);
        let expected = C::new(3. * (n - 1) as f64 / 16., 0.);
        assert!((h.hs_inner_product(&h).unwrap() - expected).norm() < 1e-12);
        bench(&format!("H n={n} hs_self"), || {
            h.hs_inner_product(&h).unwrap()
        });
    }
    contractions();
}

static TRACE_CALLS: AtomicUsize = AtomicUsize::new(0);
#[derive(PartialEq)]
struct CountOp(O);
impl Identity for CountOp {
    fn identity() -> Self {
        Self(O::I)
    }
}
impl HomogeneousParity for CountOp {
    fn parity(&self) -> Parity {
        self.0.parity()
    }
}
impl ParityOperator for CountOp {
    fn parity_operator() -> Self {
        Self(O::I)
    }
}
impl Product<C> for CountOp {
    type Term = Self;
    fn product(&self, rhs: &Self) -> LinearCombination<Self, C> {
        <O as Product<C>>::product(&self.0, &rhs.0).map(Self)
    }
}
impl Adjoint<C> for CountOp {
    type Term = Self;
    fn adjoint(&self) -> LinearCombination<Self, C> {
        <O as Adjoint<C>>::adjoint(&self.0).map(Self)
    }
}
impl Trace<C> for CountOp {
    fn trace(&self) -> C {
        TRACE_CALLS.fetch_add(1, Relaxed);
        <O as Trace<C>>::trace(&self.0)
    }
}

fn contractions() {
    type CQ = syntaqs_core::ExpressionFactory<CountState, O, C>;
    type OQ = syntaqs_core::ExpressionFactory<(), CountOp, C>;
    let n = 32;
    let sites: Vec<_> = (0..n).map(HilbertSpace::factor).collect();
    let space = HilbertSpace::tensor_all(sites.clone()).unwrap();
    let up = Q::product_state(&space, (0..n).map(|_| S::Up)).unwrap();
    let xs = Op::product(sites.iter().map(|s| Q::operator(O::Sx).at(s).unwrap()))
        .unwrap()
        .bind(&space)
        .unwrap();
    for zero_at in [None, Some(0), Some(n / 2), Some(n - 1)] {
        let expected = if zero_at.is_none() { 1. } else { 0. };
        let ket = |i| if zero_at == Some(i) { S::Down } else { S::Up };
        let other = Q::product_state(&space, (0..n).map(ket)).unwrap();
        assert_eq!(up.inner_product(&other).unwrap(), C::new(expected, 0.));
        bench(&format!("state n={n} inner zero_at={zero_at:?}"), || {
            up.inner_product(&other).unwrap()
        });
        let a = CQ::product_state(&space, (0..n).map(|_| CountState(S::Up))).unwrap();
        let b = CQ::product_state(&space, (0..n).map(|i| CountState(ket(i)))).unwrap();
        INNER_CALLS.store(0, Relaxed);
        assert_eq!(a.inner_product(&b).unwrap(), C::new(expected, 0.));
        println!(
            "state n={n} zero_at={zero_at:?} local_inner_product_calls={}",
            INNER_CALLS.load(Relaxed)
        );

        let op = |i| if zero_at == Some(i) { O::Sz } else { O::Sx };
        let ys = Op::product(
            sites
                .iter()
                .enumerate()
                .map(|(i, s)| Q::operator(op(i)).at(s).unwrap()),
        )
        .unwrap()
        .bind(&space)
        .unwrap();
        assert_eq!(xs.hs_inner_product(&ys).unwrap(), C::new(expected, 0.));
        bench(&format!("operator n={n} hs zero_at={zero_at:?}"), || {
            xs.hs_inner_product(&ys).unwrap()
        });
        let counted = |other: bool| {
            OperatorOf::<OQ, usize>::product(sites.iter().enumerate().map(|(i, s)| {
                OQ::operator(CountOp(if other { op(i) } else { O::Sx }))
                    .at(s)
                    .unwrap()
            }))
            .unwrap()
            .bind(&space)
            .unwrap()
        };
        TRACE_CALLS.store(0, Relaxed);
        assert_eq!(
            counted(false).hs_inner_product(&counted(true)).unwrap(),
            C::new(expected, 0.)
        );
        println!(
            "operator n={n} zero_at={zero_at:?} primitive_trace_calls={}",
            TRACE_CALLS.load(Relaxed)
        );
    }
}
