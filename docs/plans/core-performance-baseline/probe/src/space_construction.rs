use std::hint::black_box;

use syntaqs_core::space::HilbertSpace;

use crate::measure::{Case, CountedIndex, Mode, Runner};

#[derive(Clone, Copy)]
enum Construction {
    OneShot,
    Successive { flatten: bool, retain: bool },
}

const CONSTRUCTIONS: [Construction; 5] = [
    Construction::OneShot,
    Construction::Successive {
        flatten: false,
        retain: false,
    },
    Construction::Successive {
        flatten: false,
        retain: true,
    },
    Construction::Successive {
        flatten: true,
        retain: false,
    },
    Construction::Successive {
        flatten: true,
        retain: true,
    },
];

impl Construction {
    fn api(self) -> &'static str {
        match self {
            Self::OneShot => "tensor_all",
            Self::Successive {
                flatten: false,
                retain: false,
            } => "tensor_successive",
            Self::Successive {
                flatten: false,
                retain: true,
            } => "tensor_successive_retained",
            Self::Successive {
                flatten: true,
                retain: false,
            } => "tensor_successive_flatten",
            Self::Successive {
                flatten: true,
                retain: true,
            } => "tensor_successive_flatten_retained",
        }
    }

    fn build<I: Ord>(self, parts: &[HilbertSpace<I>]) -> (HilbertSpace<I>, Vec<HilbertSpace<I>>) {
        let Self::Successive { flatten, retain } = self else {
            return (
                HilbertSpace::tensor_all(parts.iter().cloned()).unwrap(),
                Vec::new(),
            );
        };

        let (first, rest) = parts.split_first().expect("nonempty benchmark input");
        let mut retained = Vec::new();
        if retain {
            retained.reserve_exact(rest.len().saturating_sub(1));
        }
        let mut space = first.clone();
        for (i, next) in rest.iter().enumerate() {
            space = space.tensor(next).unwrap();
            if flatten {
                space = space.flatten();
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
                            Construction::Successive { retain: true, .. } => parts.len() - 2,
                            _ => 0,
                        };
                        if matches!(construction, Construction::Successive { flatten: true, .. }) {
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
            }
        }
    }
}
