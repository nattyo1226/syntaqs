//! Combined grouping, repeated collection, contraction, and coordinate queries.
use syntaqs_core::{HilbertSpace as H, Operator, TensorKind};
use syntaqs_tests::graded::*;

fn pipeline<T: Real>() {
    for kind in [TensorKind::Ordinary, TensorKind::Graded] {
        let sites = [H::factor(4), H::factor(1), H::factor(8)];
        let space = H::tensor_all_with(kind, sites.iter().cloned()).unwrap();
        // A three-site hopping model with a local potential and a spectator.
        let hop = (placed::<T>(CREATE, &sites[1]) * placed(A, &sites[0])).unwrap();
        let potential = (placed::<T>(Op::I, &sites[2]) - placed(Op::P, &sites[2]))
            .unwrap()
            .scaled(c(0.25, 0.0));
        let h = Operator::sum([hop.clone(), hop.adjoint(), potential])
            .unwrap()
            .bind(&space)
            .unwrap();
        let grouped = h.group_local();
        let hopping = product(
            &embedding::<T>(CREATE, 1, 3, kind),
            &embedding(A, 0, 3, kind),
        );
        let reverse = adjoint(&hopping);
        let identity = embedding::<T>(Op::I, 2, 3, kind);
        let parity = embedding::<T>(Op::P, 2, 3, kind);
        let matrix: Vec<_> = (0..64)
            .map(|i| hopping[i] + reverse[i] + (identity[i] - parity[i]) * c(0.25, 0.0))
            .collect();
        let squared = (grouped.clone() * &grouped).unwrap();
        let square_matrix = product(&matrix, &matrix);
        close(
            squared.hs_inner_product(&squared).unwrap(),
            trace(&product(&adjoint(&square_matrix), &square_matrix)) / c(8.0, 0.0),
        );
        // Matrix-unit identity has two components; Parities has a singleton.
        for (expanded, coordinates, elements) in [
            (
                squared.expand(&Units).unwrap(),
                squared
                    .coordinates(&Units)
                    .unwrap()
                    .into_entries()
                    .collect::<Vec<_>>(),
                [Op::U(0, 0), A, CREATE, Op::U(1, 1)],
            ),
            (
                squared.expand(&Parities).unwrap(),
                squared
                    .coordinates(&Parities)
                    .unwrap()
                    .into_entries()
                    .collect::<Vec<_>>(),
                [Op::I, Op::P, A, CREATE],
            ),
        ] {
            let mut reconstructed = vec![c(0.0, 0.0); 64];
            for (digits, coefficient) in &coordinates {
                let mut term = identity.clone();
                for (position, digit) in digits.iter().copied().enumerate() {
                    term = product(&term, &embedding(elements[digit], position, 3, kind));
                }
                for (value, entry) in reconstructed.iter_mut().zip(term) {
                    *value = *value + entry * *coefficient;
                }
            }
            values(&reconstructed, &square_matrix);
            for bits in 0..8 {
                let input = ket::<T>(bits, &sites, kind);
                let mut basis_vector = vec![c(0.0, 0.0); 8];
                basis_vector[bits] = c(1.0, 0.0);
                values(
                    &vector(&input.act(&expanded).unwrap()),
                    &act(&square_matrix, &basis_vector),
                );
            }
        }
        let mut state = ket::<T>(0b101, &sites, kind);
        let mut expected = vec![c(0.0, 0.0); 8];
        expected[0b101] = c(1.0, 0.0);
        for _ in 0..4 {
            state = state.act(&grouped).unwrap().evaluate().unwrap();
            expected = act(&matrix, &expected);
            values(&vector(&state), &expected);
            close(
                state.inner_product(&state).unwrap(),
                expected
                    .iter()
                    .fold(c(0.0, 0.0), |sum, a| sum + a.conj() * *a),
            );
        }
    }
}

#[test]
fn integrated_pipeline_f32() {
    pipeline::<f32>();
}

#[test]
fn integrated_pipeline_f64() {
    pipeline::<f64>();
}
