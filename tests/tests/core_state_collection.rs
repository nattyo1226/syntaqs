//! Repeated actions against analytic rotations and an independent bit-state H.
use num_complex::Complex;
use syntaqs_core::{HilbertSpace as H, LocalOperator, LocalState, Operator, State};
use syntaqs_spin_half::{SpinHalfOperator as O, SpinHalfState as S, SzBasis};
use syntaqs_tests::graded::{Real, c};

fn repeated_actions<T: Real>() {
    type C<T> = Complex<T>;
    let up = LocalState::<S, O, C<T>>::primitive(S::Up);
    let rotation = LocalOperator::primitive(O::I).scaled(c::<T>(0.1f64.cos(), 0.0))
        + LocalOperator::primitive(O::Sy).scaled(c::<T>(0.0, -0.1f64.sin()));
    for steps in [1, 8, 32, 128] {
        let mut state = up.clone();
        for _ in 0..steps {
            state = state.act(&rotation);
        }
        let evaluated = state.evaluate();
        let actual = evaluated.coordinates(&SzBasis).unwrap();
        for i in 0..2 {
            let expected = c::<T>(
                if i == 0 {
                    (0.1 * steps as f64).cos()
                } else {
                    (0.1 * steps as f64).sin()
                },
                0.0,
            );
            let value = actual
                .entries()
                .find(|(key, _)| key[0] == i)
                .map(|(_, v)| *v)
                .unwrap_or_else(|| c(0.0, 0.0));
            let delta = value - expected;
            assert!(
                delta.re.to_f64().unwrap().abs() < steps as f64 * T::TOL
                    && delta.im.to_f64().unwrap().abs() < T::TOL
            );
        }
        let composed = up
            .act(&LocalOperator::product(
                (0..steps).map(|_| rotation.clone()),
            ))
            .coordinates(&SzBasis)
            .unwrap();
        assert_eq!(
            actual.into_entries().collect::<Vec<_>>(),
            composed.into_entries().collect::<Vec<_>>()
        );
    }
    let n = 6;
    let sites: Vec<_> = (0..n).map(H::factor).collect();
    let space = H::tensor_all(sites.iter().cloned()).unwrap();
    let h = Operator::try_sum(
        (0..n - 1)
            .flat_map(|i| [O::Sx, O::Sy, O::Sz].map(move |op| (i, op)))
            .map(|(i, op)| {
                Ok((LocalOperator::<O, C<T>>::primitive(op).at(&sites[i])?
                    * LocalOperator::primitive(op).at(&sites[i + 1])?)?
                .scaled(c(-0.25, 0.0)))
            }),
    )
    .unwrap()
    .bind(&space)
    .unwrap();
    let mut psi = State::tensor_in(
        &space,
        sites.iter().enumerate().map(|(i, site)| {
            LocalState::<S, O, C<T>>::primitive(if i % 2 == 0 { S::Up } else { S::Down })
                .at(site)
                .unwrap()
        }),
    )
    .unwrap();
    let mut reference = vec![0.0; 1 << n];
    reference[0b010101] = 1.0;
    for _ in 0..8 {
        let mut next = vec![0.0; 1 << n];
        for (bits, a) in reference.iter().copied().enumerate() {
            for i in 0..n - 1 {
                let left = 1 << (n - 1 - i);
                let right = left >> 1;
                let z = |mask| if bits & mask == 0 { 1.0 } else { -1.0 };
                next[bits] -= 0.25 * z(left) * z(right) * a;
                if z(left) != z(right) {
                    next[bits ^ left ^ right] -= 0.5 * a;
                }
            }
        }
        psi = psi.act(&h).unwrap();
        let evaluated = psi.evaluate().unwrap();
        let actual: std::collections::BTreeMap<_, _> = evaluated
            .coordinates(&SzBasis)
            .unwrap()
            .flat_entries()
            .map(|r| {
                let (i, c) = r.unwrap();
                (i, *c)
            })
            .collect();
        for (i, expected) in next.iter().copied().enumerate() {
            let value = actual.get(&i).copied().unwrap_or_else(|| c(0.0, 0.0));
            assert!(
                (value.re.to_f64().unwrap() - expected).abs() <= T::TOL * (1.0 + expected.abs())
            );
            assert!(value.im.to_f64().unwrap().abs() < T::TOL);
        }
        assert!(actual.len() <= 20); // The half-filled six-site sector has 20 states.
        reference = next;
    }
    // Act scope and original-reference validation still run before collection.
    let a = H::factor(0);
    let outside = H::factor(9);
    let state = State::place(&up, &a).unwrap();
    let invalid = (Operator::<i32, O, C<T>>::zero()
        * LocalOperator::<O, C<T>>::primitive(O::I)
            .at(&outside)
            .unwrap())
    .unwrap();
    assert!(state.act(&invalid).is_err());
}
#[test]
fn rotations_and_heisenberg_powers_f32() {
    repeated_actions::<f32>();
}
#[test]
fn rotations_and_heisenberg_powers_f64() {
    repeated_actions::<f64>();
}
