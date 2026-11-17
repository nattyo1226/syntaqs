use syntaqs_core::prelude::*;

#[test]
fn prelude_supports_custom_local_algebra_and_expression_construction() {
    struct Ket;
    struct Op;
    impl Act<i32> for Ket {
        type Operator = Op;
        type Term = Self;

        fn act(&self, _: &Op) -> LinearCombination<Self, i32> {
            LinearCombination::term(2, Ket)
        }
    }

    type System = ExpressionFactory<Ket, Op, i32>;
    let space = HilbertSpace::factor(0);
    let state = System::state(Ket).at(&space).unwrap();
    let operator = System::operator(Op).at(&space).unwrap();
    let _: State<_, Ket, Op, i32> = state.act(&operator).unwrap();
    let terms = Ket.act(&Op);
    assert_eq!(terms.terms().next().unwrap().coefficient, 2);
}
