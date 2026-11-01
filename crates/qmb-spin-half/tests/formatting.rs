use qmb_core::{HilbertSpace, LocalOperator, LocalState};
use qmb_spin_half::{SpinHalfOperator as O, SpinHalfState as S};

#[test]
fn every_primitive_has_a_label_and_states_gain_kets_only_in_expressions() {
    let factor = HilbertSpace::factor("a");
    for (op, expected) in [
        (O::I, "I"),
        (O::Sx, "Sx"),
        (O::Sy, "Sy"),
        (O::Sz, "Sz"),
        (O::Sp, "Sp"),
        (O::Sm, "Sm"),
    ] {
        assert_eq!(op.to_string(), expected);
        assert_eq!(
            LocalOperator::<_, i32>::primitive(op)
                .at(&factor)
                .unwrap()
                .to_string(),
            format!("{expected}[a]")
        );
    }
    for (state, expected) in [(S::Up, "up"), (S::Down, "down")] {
        assert_eq!(state.to_string(), expected);
        let local = LocalState::<_, O, i32>::primitive(state);
        assert_eq!(local.to_string(), format!("|{expected}>"));
        assert_eq!(
            local.at(&factor).unwrap().to_string(),
            format!("|{expected}[a]>")
        );
    }
    assert_eq!(LocalState::<S, O, i32>::zero().to_string(), "0");
}
