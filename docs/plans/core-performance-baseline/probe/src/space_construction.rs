use std::hint::black_box;

use syntaqs_core::space::HilbertSpace;

use crate::{
    Space,
    measure::{Case, CountedIndex, Mode, Runner},
    spin_half::hamiltonian,
};

#[derive(Clone, Copy)]
enum Construction {
    OneShot,
    Successive(Composition, Retention),
}

#[derive(Clone, Copy)]
enum Composition {
    Tensor,
    ThenFlatten,
    TensorFlatten,
}

#[derive(Clone, Copy)]
enum Retention {
    FinalOnly,
    All,
}

const CONSTRUCTIONS: [Construction; 7] = [
    Construction::OneShot,
    Construction::Successive(Composition::Tensor, Retention::FinalOnly),
    Construction::Successive(Composition::Tensor, Retention::All),
    Construction::Successive(Composition::ThenFlatten, Retention::FinalOnly),
    Construction::Successive(Composition::ThenFlatten, Retention::All),
    Construction::Successive(Composition::TensorFlatten, Retention::FinalOnly),
    Construction::Successive(Composition::TensorFlatten, Retention::All),
];

impl Construction {
    fn api(self) -> &'static str {
        match self {
            Self::OneShot => "tensor_all",
            Self::Successive(Composition::Tensor, Retention::FinalOnly) => "tensor_successive",
            Self::Successive(Composition::Tensor, Retention::All) => "tensor_successive_retained",
            Self::Successive(Composition::ThenFlatten, Retention::FinalOnly) => {
                "tensor_successive_flatten"
            }
            Self::Successive(Composition::ThenFlatten, Retention::All) => {
                "tensor_successive_flatten_retained"
            }
            Self::Successive(Composition::TensorFlatten, Retention::FinalOnly) => {
                "tensor_flatten_successive"
            }
            Self::Successive(Composition::TensorFlatten, Retention::All) => {
                "tensor_flatten_successive_retained"
            }
        }
    }

    fn build<I: Ord>(self, parts: &[HilbertSpace<I>]) -> (HilbertSpace<I>, Vec<HilbertSpace<I>>) {
        let Self::Successive(composition, retention) = self else {
            return (
                HilbertSpace::tensor_all(parts.iter().cloned()).unwrap(),
                Vec::new(),
            );
        };

        let (first, rest) = parts.split_first().expect("nonempty benchmark input");
        let retain = matches!(retention, Retention::All);
        let mut retained = Vec::new();
        if retain {
            retained.reserve_exact(rest.len().saturating_sub(1));
        }
        let mut space = first.clone();
        for (i, next) in rest.iter().enumerate() {
            match composition {
                Composition::Tensor => space = space.tensor(next).unwrap(),
                Composition::ThenFlatten => {
                    space = space.tensor(next).unwrap();
                    space = space.flatten();
                }
                Composition::TensorFlatten => space = space.tensor_flatten(next).unwrap(),
            }
            if retain && i + 1 < rest.len() {
                retained.push(space.clone());
            }
        }
        (space, retained)
    }
}

fn prepare<I: Ord>(indices: impl Iterator<Item = I>, grouped: bool) -> Vec<HilbertSpace<I>> {
    let leaves = indices.map(HilbertSpace::factor).collect::<Vec<_>>();
    if grouped {
        leaves
            .chunks_exact(4)
            .map(|group| HilbertSpace::tensor_all(group.iter().cloned()).unwrap())
            .collect()
    } else {
        leaves
    }
}

fn measure_binding(r: &Runner, input: &Case, parts: &[Space]) {
    let leaves: Vec<_> = parts
        .iter()
        .flat_map(|part| part.leaves().cloned())
        .collect();
    let operator = hamiltonian(&leaves);
    for construction in CONSTRUCTIONS {
        if matches!(construction, Construction::Successive(_, Retention::All)) {
            continue;
        }
        let api = construction.api();
        let reused = Case {
            system: "space_bind",
            ..*input
        };
        if reused.id(api).contains(&r.filter) {
            let (space, _) = construction.build(parts);
            r.measure(&reused, api, || operator.bind(black_box(&space)).unwrap());
        }
        for order in [1, 8] {
            let combined = Case {
                system: "space_build_bind",
                order,
                ..*input
            };
            r.measure(&combined, api, || {
                let (space, _) = construction.build(black_box(parts));
                for _ in 0..order {
                    drop(black_box(operator.bind(&space).unwrap()));
                }
                space
            });
        }
    }
}

pub fn run(r: &Runner) {
    for n in [8, 32, 128, 512, 1_024] {
        let ordered = (0..n).collect::<Vec<_>>();
        let interleaved = (0..n / 2).flat_map(|i| [n - 1 - i, i]).collect::<Vec<_>>();
        for (workload, ids, grouped) in [
            ("space_flat_ordered", &ordered, false),
            ("space_flat_unsorted", &interleaved, false),
            ("space_nested_unsorted", &interleaved, true),
        ] {
            let case = Case {
                system: "space",
                workload,
                sites: n,
                factors: n,
                order: 1,
            };
            if matches!(r.mode, Mode::Diagnostics) {
                let parts = prepare(ids.iter().copied().map(CountedIndex), grouped);
                let prebuilt = HilbertSpace::tensor_all(parts.iter().cloned()).unwrap();
                r.diagnose_index_comparisons(&case, "flatten", || prebuilt.flatten());
                r.diagnose(&case, "flatten", "result_children", || {
                    let flattened = prebuilt.flatten();
                    assert!(flattened.is_compatible_with(&prebuilt));
                    assert_eq!(flattened.children().len(), n);
                    flattened.children().len()
                });
                drop(prebuilt);
                for construction in CONSTRUCTIONS {
                    let api = construction.api();
                    r.diagnose_index_comparisons(&case, api, || construction.build(&parts));
                    r.diagnose(&case, api, "retained_intermediates", || {
                        let (space, retained) = construction.build(&parts);
                        assert!(space.indices().map(|id| id.0).eq(ids.iter().copied()));
                        let expected = match construction {
                            Construction::Successive(_, Retention::All) => parts.len() - 2,
                            _ => 0,
                        };
                        if matches!(
                            construction,
                            Construction::Successive(
                                Composition::ThenFlatten | Composition::TensorFlatten,
                                _
                            )
                        ) {
                            assert_eq!(space.children().len(), n);
                        }
                        assert_eq!(retained.len(), expected);
                        let width = if grouped { 4 } else { 1 };
                        for (i, intermediate) in retained.iter().enumerate() {
                            assert!(
                                intermediate
                                    .indices()
                                    .map(|id| id.0)
                                    .eq(ids[..(i + 2) * width].iter().copied())
                            );
                        }
                        retained.len()
                    });
                }
            } else {
                let parts = prepare(ids.iter().copied(), grouped);
                let prebuilt = HilbertSpace::tensor_all(parts.iter().cloned()).unwrap();
                r.measure_construction(&case, "flatten", || black_box(&prebuilt).flatten());
                drop(prebuilt);
                for construction in CONSTRUCTIONS {
                    r.measure_construction(&case, construction.api(), || {
                        construction.build(black_box(&parts))
                    });
                }
                if matches!(r.mode, Mode::Time | Mode::Allocations) && matches!(n, 32 | 128 | 1_024)
                {
                    measure_binding(r, &case, &parts);
                }
            }
        }
    }
}
