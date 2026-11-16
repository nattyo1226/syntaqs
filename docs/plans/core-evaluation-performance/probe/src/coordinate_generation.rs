//! Additional Stage 4 output controls; the original comparison cases stay intact.
use super::*;

pub(super) fn run() {
    for n in [16, 32, 64] {
        let (h, _, space, sites) = model(n);
        let expected: std::collections::BTreeMap<_, _> = h
            .coordinates(&CartesianBasis)
            .unwrap()
            .into_entries()
            .collect();
        let expanded = h.expand(&CartesianBasis).unwrap();
        assert_eq!(
            expanded
                .coordinates(&CartesianBasis)
                .unwrap()
                .into_entries()
                .collect::<std::collections::BTreeMap<_, _>>(),
            expected
        );
        bench(&format!("H n={n} expand"), || {
            h.expand(&CartesianBasis).unwrap()
        });
        BASIS_CALLS.store(0, Relaxed);
        ID_CALLS.store(0, Relaxed);
        let expanded = h.expand(&CountBasis).unwrap();
        println!(
            "H n={n} expand basis_calls={} identity_calls={}",
            BASIS_CALLS.load(Relaxed),
            ID_CALLS.load(Relaxed)
        );
        black_box(expanded);
        let string = Op::product(sites.iter().map(|s| Q::operator(O::Sx).at(s).unwrap()))
            .unwrap()
            .bind(&space)
            .unwrap();
        let coordinates = string.coordinates(&CartesianBasis).unwrap();
        assert_eq!(
            coordinates.into_entries().collect::<Vec<_>>(),
            [(vec![1; n], C::new(1.0, 0.0))]
        );
        bench(&format!("full Pauli string n={n} coordinates"), || {
            string.coordinates(&CartesianBasis).unwrap()
        });
        bench(&format!("full Pauli string n={n} expand"), || {
            string.expand(&CartesianBasis).unwrap()
        });
    }
    let (h, _, _, _) = model(16);
    let square = (h.clone() * h).unwrap();
    let expected: std::collections::BTreeMap<_, _> = square
        .coordinates(&CartesianBasis)
        .unwrap()
        .into_entries()
        .collect();
    assert_eq!(
        square
            .expand(&CartesianBasis)
            .unwrap()
            .coordinates(&CartesianBasis)
            .unwrap()
            .into_entries()
            .collect::<std::collections::BTreeMap<_, _>>(),
        expected
    );
    bench("H2 n=16 expand", || square.expand(&CartesianBasis).unwrap());
}
