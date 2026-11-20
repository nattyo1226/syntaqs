use std::hint::black_box;

use syntaqs_core::space::HilbertSpace;
use syntaqs_fermion::{
    Fermion,
    basis::{LadderBasis, MajoranaBasis, OccupationBasis},
    models::FermionModels,
    operator::FermionOperator as O,
    state::FermionState::{Empty, Occupied},
};
use syntaqs_geometry::geometry::{Boundary, chain::Chain};

use crate::{
    C, Space, close, factors,
    measure::{Case, CountedIndex, Mode, Runner},
};

type Q = Fermion<C>;

fn case(workload: &'static str, sites: usize, factors: usize) -> Case {
    Case {
        system: "fermion",
        workload,
        sites,
        factors,
        order: 1,
    }
}

pub fn run(r: &Runner) {
    hubbard(r);
    hopping(r);
    product_zero_before_sum(r);
}

fn product_zero_before_sum(r: &Runner) {
    let rest = Q::operator(O::Create) + Q::operator(O::Number);
    let annihilate = Q::operator(O::Annihilate);
    let product = &rest * &annihilate;
    let local = Q::state(Empty).act(&(product.clone() + Q::operator(O::I)));
    let c = case("local_zero_before_sum", 1, 1);
    r.measure(&c, "evaluate", || black_box(&local).evaluate());
    r.diagnose(&c, "state_coordinates_occupation", "nnz", || {
        let out = local.coordinates(&OccupationBasis).unwrap();
        assert!(out.entries().eq([(&[0][..], &C::new(1.0, 0.0))]));
        out.len()
    });

    let modes = factors(2);
    let space = HilbertSpace::tensor_all_g(modes.iter().cloned()).unwrap();
    let vacuum = Q::product_state(&space, [Empty; 2]).unwrap();
    let spanning_sum = (Q::operator(O::Create).at(&modes[0]).unwrap()
        + Q::operator(O::Number).at(&modes[1]).unwrap())
    .unwrap();
    let spanning_product = (&spanning_sum * annihilate.at(&modes[1]).unwrap()).unwrap();
    for (workload, product) in [
        ("placed_zero_before_sum", product.at(&modes[1]).unwrap()),
        ("composite_zero_before_sum", spanning_product),
    ] {
        let c = case(workload, 2, 2);
        let identity = Q::operator(O::I).at(&modes[0]).unwrap();
        let operator = (product + identity).unwrap();
        let delayed = vacuum.act(&operator).unwrap();
        r.measure(&c, "evaluate", || black_box(&delayed).evaluate().unwrap());
        r.diagnose(&c, "state_coordinates_occupation", "nnz", || {
            let out = delayed.coordinates(&OccupationBasis).unwrap();
            assert!(out.entries().eq([(&[0, 0][..], &C::new(1.0, 0.0))]));
            out.len()
        });
    }
}

fn hubbard(r: &Runner) {
    for n in [2, 4, 8, 16, 32, 64] {
        let modes = factors(2 * n);
        let sites: Vec<_> = modes
            .chunks_exact(2)
            .map(|p| HilbertSpace::tensor_all_g(p.iter().cloned()).unwrap())
            .collect();
        let chain = Chain::new(n, Boundary::Open);
        let build = |sites: &[Space]| FermionModels::<Q>::hubbard(&chain, sites, 1.0, 4.0).unwrap();
        let h = build(&sites);
        let space = h.space().unwrap();
        let c = case("hubbard", n, 2 * n);
        r.measure(&c, "build_model", || build(black_box(&sites)));
        r.measure(&c, "operator_coordinates_ladder", || {
            black_box(&h).coordinates(&LadderBasis).unwrap()
        });
        r.measure(&c, "operator_coordinates_majorana", || {
            black_box(&h).coordinates(&MajoranaBasis).unwrap()
        });
        r.measure(&c, "operator_expand_majorana", || {
            black_box(&h).expand(&MajoranaBasis).unwrap()
        });
        r.measure(&c, "trace", || black_box(&h).trace().unwrap());
        r.diagnose(&c, "operator_coordinates_ladder", "nnz", || {
            let len = h.coordinates(&LadderBasis).unwrap().len();
            assert_eq!(len, n + 4 * (n - 1));
            len
        });
        r.diagnose(&c, "operator_coordinates_majorana", "nnz", || {
            let len = h.coordinates(&MajoranaBasis).unwrap().len();
            assert_eq!(len, 1 + 3 * n + 4 * (n - 1));
            let actual = h.trace().unwrap() / 2.0_f64.powi((2 * n) as i32);
            close(actual, n as f64);
            len
        });
        let original: Vec<_> = [1, 0, 0, 1].into_iter().cycle().take(2 * n).collect();
        let neel = Q::product_state(
            space,
            original
                .iter()
                .map(|&i| if i == 0 { Empty } else { Occupied }),
        )
        .unwrap();
        let vacuum = Q::product_state(space, std::iter::repeat_n(Empty, 2 * n)).unwrap();
        for (workload, state, is_vacuum) in [
            ("hubbard_neel", neel, false),
            ("hubbard_vacuum", vacuum, true),
        ] {
            let c = case(workload, n, 2 * n);
            let delayed = state.act(&h).unwrap();
            r.measure(&c, "evaluate", || black_box(&delayed).evaluate().unwrap());
            r.measure(&c, "expectation", || {
                black_box(&state).expectation(&h).unwrap()
            });
            r.measure(&c, "state_coordinates_occupation", || {
                black_box(&delayed).coordinates(&OccupationBasis).unwrap()
            });
            r.diagnose(&c, "state_coordinates_occupation", "nnz", || {
                let out = delayed.coordinates(&OccupationBasis).unwrap();
                close(state.expectation(&h).unwrap(), 0.0);
                assert_eq!(out.len(), if is_vacuum { 0 } else { 2 * (n - 1) });
                if !is_vacuum {
                    for i in 0..(2 * n - 2) {
                        let mut moved = original.clone();
                        moved.swap(i, i + 2);
                        let value = *out.entries().find(|(d, _)| *d == moved).unwrap().1;
                        close(value, if original[i + 1] == 0 { -1.0 } else { 1.0 });
                    }
                }
                out.len()
            });
        }
    }
}

fn hopping(r: &Runner) {
    for n in [16, 64, 128] {
        let modes = factors(n);
        let space = HilbertSpace::tensor_all_g(modes.iter().cloned()).unwrap();
        let state = Q::product_state(
            &space,
            (0..n).map(|i| if i == 0 || i == 2 { Occupied } else { Empty }),
        )
        .unwrap();
        for (workload, to) in [("hop_near", 1), ("hop_mid", n / 2), ("hop_far", n - 1)] {
            let c = case(workload, n, n);
            let hop = Q::operator_product_at([(&modes[to], O::Create), (&modes[0], O::Annihilate)])
                .unwrap()
                .bind(&space)
                .unwrap();
            let delayed = state.act(&hop).unwrap();
            r.measure(&c, "evaluate", || black_box(&delayed).evaluate().unwrap());
            r.measure(&c, "operator_coordinates_ladder", || {
                black_box(&hop).coordinates(&LadderBasis).unwrap()
            });
            r.diagnose(&c, "state_coordinates_occupation", "nnz", || {
                let out = delayed.coordinates(&OccupationBasis).unwrap();
                assert_eq!(out.len(), 1);
                let (digits, &value) = out.entries().next().unwrap();
                assert_eq!(digits.iter().sum::<usize>(), 2);
                assert_eq!((digits[2], digits[to]), (1, 1));
                close(value, if to > 2 { -1.0 } else { 1.0 });
                out.len()
            });
            r.diagnose(&c, "operator_coordinates_ladder", "nnz", || {
                let len = hop.coordinates(&LadderBasis).unwrap().len();
                assert_eq!(len, 1);
                len
            });
            if matches!(r.mode, Mode::Diagnostics) {
                let modes: Vec<_> = (0..n)
                    .map(|i| HilbertSpace::factor(CountedIndex(i)))
                    .collect();
                let space = HilbertSpace::tensor_all_g(modes.iter().cloned()).unwrap();
                let state = Q::product_state(
                    &space,
                    (0..n).map(|i| if i == 0 || i == 2 { Occupied } else { Empty }),
                )
                .unwrap();
                let hop =
                    Q::operator_product_at([(&modes[to], O::Create), (&modes[0], O::Annihilate)])
                        .unwrap()
                        .bind(&space)
                        .unwrap();
                let delayed = state.act(&hop).unwrap();
                r.diagnose_index_comparisons(&c, "evaluate", || delayed.evaluate().unwrap());
            }
        }
    }
}
