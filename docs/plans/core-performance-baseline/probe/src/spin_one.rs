use std::hint::black_box;

use syntaqs_geometry::geometry::{Boundary, chain::Chain};
use syntaqs_spin_one::{
    SpinOne,
    basis::{CartesianBasis, LadderBasis, SzBasis},
    models::SpinOneModels,
    state::SpinOneState::{Minus, Plus, Zero},
};

use crate::{
    C, close, factors,
    measure::{Case, Runner},
};

type Q = SpinOne<C>;

pub fn run(r: &Runner) {
    for n in [4, 8, 16, 32, 64] {
        let c = Case {
            system: "spin_one",
            workload: "blbq_neel",
            sites: n,
            factors: n,
            order: 1,
        };
        let sites = factors(n);
        let chain = Chain::new(n, Boundary::Open);
        let build = |sites: &[_]| SpinOneModels::<Q>::blbq(&chain, sites, 1.0, 0.25).unwrap();
        let h = build(&sites);
        let psi = Q::product_state(
            h.space().unwrap(),
            [Plus, Minus].into_iter().cycle().take(n),
        )
        .unwrap();
        let delayed = psi.act(&h).unwrap();
        r.measure(&c, "build_model", || build(black_box(&sites)));
        r.measure(&c, "evaluate", || black_box(&delayed).evaluate().unwrap());
        r.measure(&c, "expectation", || {
            black_box(&psi).expectation(&h).unwrap()
        });
        r.measure(&c, "state_coordinates_sz", || {
            black_box(&delayed).coordinates(&SzBasis).unwrap()
        });
        r.measure(&c, "operator_coordinates_cartesian", || {
            black_box(&h).coordinates(&CartesianBasis).unwrap()
        });
        r.measure(&c, "operator_coordinates_ladder", || {
            black_box(&h).coordinates(&LadderBasis).unwrap()
        });
        r.measure(&c, "operator_expand_cartesian", || {
            black_box(&h).expand(&CartesianBasis).unwrap()
        });
        r.diagnose(&c, "state_coordinates_sz", "nnz", || {
            let out = delayed.coordinates(&SzBasis).unwrap();
            let index = |value| {
                <SzBasis as syntaqs_core::basis::Basis<_, C>>::index(&SzBasis, &value).unwrap()
            };
            let original: Vec<_> = [Plus, Minus]
                .into_iter()
                .cycle()
                .take(n)
                .map(index)
                .collect();
            let value = |key: &[usize]| *out.entries().find(|(d, _)| *d == key).unwrap().1;
            let diagonal = (n - 1) as f64 * 0.5;
            close(value(&original), diagonal);
            close(psi.expectation(&h).unwrap(), diagonal);
            for i in 0..n - 1 {
                let mut digits = original.clone();
                digits.swap(i, i + 1);
                close(value(&digits), -0.25);
                digits[i..=i + 1].fill(index(Zero));
                close(value(&digits), -0.75);
            }
            assert_eq!(out.len(), 1 + 2 * (n - 1));
            out.len()
        });
        r.diagnose(&c, "operator_coordinates_cartesian", "nnz", || {
            h.coordinates(&CartesianBasis).unwrap().len()
        });
        r.diagnose(&c, "operator_coordinates_ladder", "nnz", || {
            h.coordinates(&LadderBasis).unwrap().len()
        });
    }
}
