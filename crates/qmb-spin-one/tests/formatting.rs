use qmb_core::{HilbertSpace, LocalOperator, LocalState};
use qmb_spin_one::{SpinOneOperator as O, SpinOneState as S};

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
        (O::Dxy, "Dxy"),
        (O::Q0, "Q0"),
        (O::Qxy, "Qxy"),
        (O::Qyz, "Qyz"),
        (O::Qzx, "Qzx"),
        (O::Q1p, "Q1p"),
        (O::Q1m, "Q1m"),
        (O::Q2p, "Q2p"),
        (O::Q2m, "Q2m"),
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
    for (state, expected) in [(S::Plus, "+1"), (S::Zero, "0"), (S::Minus, "-1")] {
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
