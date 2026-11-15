//! Separate construction grouping from deferred query-time preparation.
use super::*;

pub(super) fn run() {
    for n in [4, 6, 8] {
        let sites: Vec<_> = (0..n).map(HilbertSpace::factor).collect();
        let space = HilbertSpace::tensor_all(sites.clone()).unwrap();
        for style in ["local", "direct", "deferred"] {
            let local_gate = Q::operator(O::Sx) + Q::operator(O::Sz);
            let layer = Op::product(sites.iter().map(|site| {
                match style {
                    "local" => local_gate.at(site).unwrap(),
                    "direct" => (Q::operator(O::Sx).at(site).unwrap()
                        + Q::operator(O::Sz).at(site).unwrap())
                    .unwrap(),
                    "deferred" => (Q::operator(O::Sx).at(site).unwrap().scaled(C::new(1., 0.))
                        + Q::operator(O::Sz).at(site).unwrap().scaled(C::new(1., 0.)))
                    .unwrap(),
                    _ => unreachable!(),
                }
            }))
            .unwrap();
            let identity = (layer.adjoint() * &layer)
                .unwrap()
                .scaled(C::new(0.5_f64.powi(n as i32), 0.))
                .bind(&space)
                .unwrap();
            let coordinates = identity.coordinates(&CartesianBasis).unwrap();
            assert_eq!(coordinates.len(), 1);
            let (digits, coefficient) = coordinates.entries().next().unwrap();
            assert!(digits.iter().all(|i| *i == 0));
            assert_eq!(*coefficient, C::new(1., 0.));
            bench(&format!("grouping n={n} {style} coordinates"), || {
                identity.coordinates(&CartesianBasis).unwrap()
            });
            bench(&format!("grouping n={n} {style} explicit_prepare"), || {
                identity.group_local()
            });
        }
    }
}
