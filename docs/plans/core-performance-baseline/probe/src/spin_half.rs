use std::{cell::Cell, hint::black_box};

use syntaqs_core::{
    basis::Basis,
    expression::parts::linear_combination::LinearCombination,
    space::HilbertSpace,
    system::{OperatorOf, StateOf},
};
use syntaqs_geometry::geometry::{Boundary, chain::Chain};
use syntaqs_spin_half::{
    SpinHalf,
    basis::{CartesianBasis, SzBasis},
    models::SpinHalfModels,
    operator::SpinHalfOperator as O,
    state::SpinHalfState::{Down, Up},
};

use crate::{
    C, Space, close, factors,
    measure::{Case, CountedIndex, Mode, Runner},
};

type Q = SpinHalf<C>;
type Models = SpinHalfModels<Q>;
type Op = OperatorOf<Q, usize>;
type St = StateOf<Q, usize>;

fn case(workload: &'static str, n: usize, order: usize) -> Case {
    Case {
        system: "spin_half",
        workload,
        sites: n,
        factors: n,
        order,
    }
}

pub fn run(r: &Runner) {
    construction(r);
    product_construction(r);
    heisenberg(r);
    states(r);
    local_action(r);
    gate_layers(r);
}

fn neel(space: &Space, n: usize) -> St {
    Q::product_state(space, [Up, Down].into_iter().cycle().take(n)).unwrap()
}

fn hamiltonian(sites: &[Space]) -> Op {
    Models::heisenberg(&Chain::new(sites.len(), Boundary::Open), sites, 1.0).unwrap()
}

fn construction(r: &Runner) {
    for n in [8, 32, 128, 512, 1024] {
        let c = case("heisenberg_construction", n, 1);
        let sites = factors(n);
        let space = HilbertSpace::tensor_all(sites.iter().cloned()).unwrap();
        let h = hamiltonian(&sites);
        let psi = neel(h.space().unwrap(), n);
        r.measure(&c, "tensor_all", || {
            HilbertSpace::tensor_all(black_box(&sites).iter().cloned()).unwrap()
        });
        r.measure(&c, "product_state", || neel(black_box(&space), n));
        r.measure(&c, "build_model", || hamiltonian(black_box(&sites)));
        r.measure(&c, "bind", || black_box(&h).bind(&space).unwrap());
        r.measure(&c, "act", || black_box(&psi).act(&h).unwrap());
        if n >= 128 {
            r.measure_prepared(
                &c,
                "bind_fresh",
                || HilbertSpace::tensor_all(sites.iter().cloned()).unwrap(),
                |fresh| black_box(&h).bind(fresh).unwrap(),
            );
            r.measure(&c, "act_in_equivalent", || {
                black_box(&psi).act_in(&h, &space).unwrap()
            });
        }

        if !matches!(r.mode, Mode::Diagnostics) {
            continue;
        }
        let sites: Vec<_> = (0..n)
            .map(|i| HilbertSpace::factor(CountedIndex(i)))
            .collect();
        let chain = Chain::new(n, Boundary::Open);
        let space = HilbertSpace::tensor_all(sites.iter().cloned()).unwrap();
        let h = Models::heisenberg(&chain, &sites, 1.0).unwrap();
        let psi = Q::product_state(h.space().unwrap(), std::iter::repeat_n(Up, n)).unwrap();
        r.diagnose_index_comparisons(&c, "tensor_all", || {
            HilbertSpace::tensor_all(sites.iter().cloned()).unwrap()
        });
        r.diagnose_index_comparisons(&c, "product_state", || {
            Q::product_state(&space, std::iter::repeat_n(Up, n)).unwrap()
        });
        r.diagnose_index_comparisons(&c, "build_model", || {
            Models::heisenberg(&chain, &sites, 1.0).unwrap()
        });
        h.bind(&space).unwrap();
        r.diagnose_index_comparisons(&c, "bind", || h.bind(&space).unwrap());
        r.diagnose_index_comparisons(&c, "act", || psi.act(&h).unwrap());
        if n >= 128 {
            let fresh = HilbertSpace::tensor_all(sites.iter().cloned()).unwrap();
            r.diagnose_index_comparisons(&c, "bind_fresh", || h.bind(&fresh).unwrap());
            r.diagnose_index_comparisons(&c, "act_in_equivalent", || {
                psi.act_in(&h, &space).unwrap()
            });
        }
    }
}

fn product_construction(r: &Runner) {
    for n in [8, 64, 1024] {
        let sites = factors(n);
        let space = HilbertSpace::tensor_all(sites.iter().cloned()).unwrap();
        let operands: Vec<Op> = sites
            .iter()
            .map(|site| Q::operator(O::Sx).at(site).unwrap())
            .collect();
        let product = Op::product(operands.iter().cloned())
            .unwrap()
            .bind(&space)
            .unwrap();
        let sequence = case("operator_product_construction", n, n);
        let binary = case("operator_product_construction", n, 2);
        r.measure(&sequence, "product_sequence", || {
            Op::product(black_box(&operands).iter().cloned()).unwrap()
        });
        r.measure(&binary, "binary_mul", || {
            (black_box(&product) * black_box(&product)).unwrap()
        });
        r.diagnose(&sequence, "product_sequence", "nnz", || {
            let out = product.coordinates(&CartesianBasis).unwrap();
            assert_eq!(out.len(), 1);
            let (digits, &value) = out.entries().next().unwrap();
            assert_eq!(digits, vec![1; n]);
            close(value, 1.0);
            out.len()
        });
        r.diagnose(&binary, "binary_mul", "nnz", || {
            let squared = (&product * &product).unwrap();
            let out = squared.coordinates(&CartesianBasis).unwrap();
            assert_eq!(out.len(), 1);
            let (digits, &value) = out.entries().next().unwrap();
            assert_eq!(digits, vec![0; n]);
            close(value, 1.0);
            out.len()
        });
    }
}

#[derive(Default)]
struct CountedCartesian {
    calls: Cell<usize>,
}

impl Basis<O, C> for CountedCartesian {
    fn expand(&self, primitive: &O) -> LinearCombination<O, C> {
        self.calls.set(self.calls.get() + 1);
        CartesianBasis.expand(primitive)
    }

    fn index(&self, primitive: &O) -> Option<usize> {
        <CartesianBasis as Basis<O, C>>::index(&CartesianBasis, primitive)
    }

    fn elements(&self) -> impl Iterator<Item = O> + '_ {
        <CartesianBasis as Basis<O, C>>::elements(&CartesianBasis)
    }
}

fn heisenberg(r: &Runner) {
    for n in [4, 8, 16, 32, 64, 128] {
        let sites = factors(n);
        let h = hamiltonian(&sites);
        let space = h.space().unwrap();
        let c = case("heisenberg", n, 1);
        r.measure(&c, "operator_coordinates_cartesian", || {
            black_box(&h).coordinates(&CartesianBasis).unwrap()
        });
        r.measure(&c, "operator_expand_cartesian", || {
            black_box(&h).expand(&CartesianBasis).unwrap()
        });
        if n <= 64 {
            r.measure(&c, "hs_self", || {
                black_box(&h).hs_inner_product(&h).unwrap()
            });
        }
        r.diagnose(&c, "operator_coordinates_cartesian", "nnz", || {
            let out = h.coordinates(&CartesianBasis).unwrap();
            assert_eq!(out.len(), 3 * (n - 1));
            for (_, &value) in out.entries() {
                close(value, -0.25);
            }
            close(h.hs_inner_product(&h).unwrap(), 3.0 * (n - 1) as f64 / 16.0);
            out.len()
        });
        r.diagnose(
            &c,
            "operator_coordinates_cartesian",
            "basis_expand_calls",
            || {
                let basis = CountedCartesian::default();
                h.coordinates(&basis).unwrap();
                basis.calls.get()
            },
        );
        for (workload, psi, aligned) in [
            ("heisenberg_neel", neel(space, n), false),
            (
                "heisenberg_aligned",
                Q::product_state(space, std::iter::repeat_n(Up, n)).unwrap(),
                true,
            ),
        ] {
            let c = case(workload, n, 1);
            let delayed = psi.act(&h).unwrap();
            r.measure(&c, "evaluate", || black_box(&delayed).evaluate().unwrap());
            r.measure(&c, "expectation", || {
                black_box(&psi).expectation(&h).unwrap()
            });
            r.measure(&c, "state_coordinates_sz", || {
                black_box(&delayed).coordinates(&SzBasis).unwrap()
            });
            r.diagnose(&c, "state_coordinates_sz", "nnz", || {
                let out = delayed.coordinates(&SzBasis).unwrap();
                let mut digits = vec![0; n];
                if !aligned {
                    for (i, digit) in digits.iter_mut().enumerate() {
                        *digit = i % 2;
                    }
                }
                let diagonal = (n - 1) as f64 * if aligned { -0.25 } else { 0.25 };
                close(psi.expectation(&h).unwrap(), diagonal);
                close(
                    *out.entries().find(|(d, _)| *d == digits).unwrap().1,
                    diagonal,
                );
                assert_eq!(out.len(), if aligned { 1 } else { n });
                if !aligned {
                    for i in 0..n - 1 {
                        digits.swap(i, i + 1);
                        close(*out.entries().find(|(d, _)| *d == digits).unwrap().1, -0.5);
                        digits.swap(i, i + 1);
                    }
                }
                out.len()
            });
        }
        if n <= 64 {
            let c = case("heisenberg", n, 2);
            let squared = (&h * &h).unwrap();
            r.measure(&c, "operator_coordinates_cartesian", || {
                black_box(&squared).coordinates(&CartesianBasis).unwrap()
            });
            r.measure(&c, "operator_expand_cartesian", || {
                black_box(&squared).expand(&CartesianBasis).unwrap()
            });
            r.diagnose(&c, "operator_coordinates_cartesian", "nnz", || {
                let out = squared.coordinates(&CartesianBasis).unwrap();
                close(
                    *out.entries().next().unwrap().1,
                    3.0 * (n - 1) as f64 / 16.0,
                );
                out.len()
            });
            r.diagnose(
                &c,
                "operator_coordinates_cartesian",
                "basis_expand_calls",
                || {
                    let basis = CountedCartesian::default();
                    squared.coordinates(&basis).unwrap();
                    basis.calls.get()
                },
            );
            r.diagnose(
                &c,
                "operator_coordinates_cartesian",
                "syntactic_term_products",
                || 9 * (n - 1).pow(2),
            );
        }
    }

    for n in [6, 10] {
        let h = hamiltonian(&factors(n));
        let psi = neel(h.space().unwrap(), n);
        for order in [1, 2, 4, 8] {
            let c = case("heisenberg_repeated_neel", n, order);
            let delayed = (0..order).fold(psi.clone(), |state, _| state.act(&h).unwrap());
            r.measure(&c, "chain_act", || {
                (0..order).fold(black_box(&psi).clone(), |state, _| state.act(&h).unwrap())
            });
            r.measure(&c, "evaluate", || black_box(&delayed).evaluate().unwrap());
            r.measure(&c, "state_coordinates_sz", || {
                black_box(&delayed).coordinates(&SzBasis).unwrap()
            });
            r.diagnose(&c, "state_coordinates_sz", "nnz", || {
                let out = delayed.coordinates(&SzBasis).unwrap();
                let reference = reference_heisenberg(n, order);
                assert_eq!(out.len(), reference.iter().filter(|c| **c != 0.0).count());
                for (digits, value) in out.entries() {
                    let mask = digits
                        .iter()
                        .enumerate()
                        .fold(0, |mask, (i, d)| mask | (d << i));
                    close(*value, reference[mask]);
                }
                out.len()
            });
        }
    }
}

fn reference_heisenberg(n: usize, order: usize) -> Vec<f64> {
    let mut state = vec![0.0; 1 << n];
    state[(1..n).step_by(2).fold(0, |mask, i| mask | (1 << i))] = 1.0;
    for _ in 0..order {
        let mut next = vec![0.0; 1 << n];
        for (mask, amplitude) in state.into_iter().enumerate() {
            for i in 0..n - 1 {
                if ((mask >> i) ^ (mask >> (i + 1))) & 1 == 0 {
                    next[mask] -= 0.25 * amplitude;
                } else {
                    next[mask] += 0.25 * amplitude;
                    next[mask ^ (3 << i)] -= 0.5 * amplitude;
                }
            }
        }
        state = next;
    }
    state
}

fn plus(space: &Space) -> St {
    let local = (Q::state(Up) + Q::state(Down)).scaled(std::f64::consts::FRAC_1_SQRT_2);
    Q::try_tensor_in(space, space.leaves().map(|f| local.at(f))).unwrap()
}

fn states(r: &Runner) {
    for n in [6, 10, 14, 18] {
        let space = HilbertSpace::tensor_all(factors(n)).unwrap();
        let ghz = Q::state_sum(
            &space,
            [
                Q::product_state(&space, std::iter::repeat_n(Up, n)).unwrap(),
                Q::product_state(&space, std::iter::repeat_n(Down, n)).unwrap(),
            ],
        )
        .unwrap()
        .scaled(std::f64::consts::FRAC_1_SQRT_2);
        for (workload, state, nnz) in [("plus", plus(&space), 1 << n), ("ghz", ghz, 2)] {
            let c = case(workload, n, 0);
            r.measure(&c, "evaluate", || black_box(&state).evaluate().unwrap());
            r.measure(&c, "norm_squared", || {
                black_box(&state).norm_squared().unwrap()
            });
            r.measure(&c, "state_coordinates_sz", || {
                black_box(&state).coordinates(&SzBasis).unwrap()
            });
            if n <= 14 || nnz == 2 {
                r.measure(&c, "state_expand_sz", || {
                    black_box(&state).expand(&SzBasis).unwrap()
                });
            }
            r.diagnose(&c, "state_coordinates_sz", "nnz", || {
                let out = state.coordinates(&SzBasis).unwrap();
                assert_eq!(out.len(), nnz);
                for (_, &value) in out.entries() {
                    close(value, 1.0 / (nnz as f64).sqrt());
                }
                close(C::from(state.norm_squared().unwrap()), 1.0);
                out.len()
            });
        }
    }
    for n in [8, 16, 32, 64] {
        let c = case("w", n, 0);
        let space = HilbertSpace::tensor_all(factors(n)).unwrap();
        let build = |space: &Space| {
            Q::try_state_sum(
                space,
                (0..n).map(|down| {
                    Q::product_state(space, (0..n).map(|i| if i == down { Down } else { Up }))
                }),
            )
            .unwrap()
            .scaled(1.0 / (n as f64).sqrt())
        };
        let state = build(&space);
        r.measure(&c, "build_state", || build(black_box(&space)));
        r.measure(&c, "evaluate", || black_box(&state).evaluate().unwrap());
        r.measure(&c, "norm_squared", || {
            black_box(&state).norm_squared().unwrap()
        });
        r.measure(&c, "state_coordinates_sz", || {
            black_box(&state).coordinates(&SzBasis).unwrap()
        });
        r.diagnose(&c, "state_coordinates_sz", "nnz", || {
            let out = state.coordinates(&SzBasis).unwrap();
            assert_eq!(out.len(), n);
            for (digits, &value) in out.entries() {
                assert_eq!(digits.iter().sum::<usize>(), 1);
                close(value, 1.0 / (n as f64).sqrt());
            }
            close(C::from(state.norm_squared().unwrap()), 1.0);
            out.len()
        });
    }
}

fn local_action(r: &Runner) {
    for n in [16, 64, 256, 1024] {
        let c = case("single_x_on_neel", n, 1);
        let sites = factors(n);
        let space = HilbertSpace::tensor_all(sites.iter().cloned()).unwrap();
        let psi = neel(&space, n);
        let x = Q::operator(O::Sx).at(&sites[n - 1]).unwrap();
        let delayed = psi.act(&x).unwrap();
        r.measure(&c, "evaluate_input", || black_box(&psi).evaluate().unwrap());
        r.measure(&c, "act", || black_box(&psi).act(&x).unwrap());
        r.measure(&c, "evaluate", || black_box(&delayed).evaluate().unwrap());
        r.diagnose(&c, "state_coordinates_sz", "nnz", || {
            let out = delayed.coordinates(&SzBasis).unwrap();
            assert_eq!(out.len(), 1);
            let (digits, &value) = out.entries().next().unwrap();
            assert!(digits[..n - 1].iter().enumerate().all(|(i, &d)| d == i % 2));
            assert_eq!(digits[n - 1], 0);
            close(value, 1.0);
            out.len()
        });
        if matches!(r.mode, Mode::Diagnostics) {
            let sites: Vec<_> = (0..n)
                .map(|i| HilbertSpace::factor(CountedIndex(i)))
                .collect();
            let space = HilbertSpace::tensor_all(sites.iter().cloned()).unwrap();
            let state = Q::product_state(&space, [Up, Down].into_iter().cycle().take(n)).unwrap();
            let op = Q::operator(O::Sx).at(&sites[n - 1]).unwrap();
            let delayed = state.act(&op).unwrap();
            r.diagnose_index_comparisons(&c, "act", || state.act(&op).unwrap());
            r.diagnose_index_comparisons(&c, "evaluate", || delayed.evaluate().unwrap());
        }
    }
}

fn gate_layers(r: &Runner) {
    for bonds in [2, 4, 6, 8] {
        let n = 2 * bonds;
        let sites = factors(n);
        let space = HilbertSpace::tensor_all(sites.iter().cloned()).unwrap();
        let build = |sites: &[Space]| {
            Q::operator_product(sites.chunks_exact(2).map(|pair| {
                let xx = Q::operator_product_at([(&pair[0], O::Sx), (&pair[1], O::Sx)]).unwrap();
                (Op::identity() + xx.scaled(C::i())).unwrap()
            }))
            .unwrap()
            .scaled(2.0_f64.powi(-(bonds as i32)).sqrt())
            .bind(&space)
            .unwrap()
        };
        let layer = build(&sites);
        let echo = (&layer.adjoint() * &layer).unwrap();
        let psi = Q::product_state(&space, std::iter::repeat_n(Up, n)).unwrap();
        r.measure(&case("xx_gate_layer", n, 1), "build_layer", || {
            build(black_box(&sites))
        });
        for (workload, order, op, nnz) in [
            ("xx_gate_layer", 1, &layer, 1 << bonds),
            ("xx_gate_echo", 2, &echo, 1),
        ] {
            let c = case(workload, n, order);
            let delayed = psi.act(op).unwrap();
            r.measure(&c, "evaluate", || black_box(&delayed).evaluate().unwrap());
            r.measure(&c, "operator_coordinates_cartesian", || {
                black_box(op).coordinates(&CartesianBasis).unwrap()
            });
            r.diagnose(&c, "operator_coordinates_cartesian", "nnz", || {
                let out = op.coordinates(&CartesianBasis).unwrap();
                assert_eq!(out.len(), nnz);
                let (digits, &value) = out.entries().next().unwrap();
                assert_eq!(digits, vec![0; n]);
                close(value, 1.0 / (nnz as f64).sqrt());
                out.len()
            });
            r.diagnose(
                &c,
                "operator_coordinates_cartesian",
                "syntactic_term_products",
                || (1usize << bonds).pow(order as u32),
            );
            r.diagnose(&c, "state_coordinates_sz", "nnz", || {
                let out = delayed.coordinates(&SzBasis).unwrap();
                assert_eq!(out.len(), nnz);
                close(
                    C::from(out.entries().map(|(_, z)| z.norm_sqr()).sum::<f64>()),
                    1.0,
                );
                out.len()
            });
        }
    }
}
