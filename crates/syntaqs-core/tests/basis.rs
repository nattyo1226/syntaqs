//! The unified basis trait does not prescribe numeric capabilities.

use syntaqs_core::{basis::Basis, expression::parts::linear_combination::LinearCombination};

enum Primitive {
    First,
    Second,
    Sum,
}
struct Scalar(u8);
struct CustomBasis;

impl Basis<Primitive, Scalar> for CustomBasis {
    fn expand_indices(&self, primitive: &Primitive) -> LinearCombination<usize, Scalar> {
        let term = |operand| LinearCombination::term(Scalar(1), operand);
        match primitive {
            Primitive::First => term(0),
            Primitive::Second => term(1),
            Primitive::Sum => term(0) + term(1),
        }
    }

    fn index(&self, primitive: &Primitive) -> Option<usize> {
        match primitive {
            Primitive::First => Some(0),
            Primitive::Second => Some(1),
            Primitive::Sum => None,
        }
    }

    fn element(&self, index: usize) -> Option<Primitive> {
        match index {
            0 => Some(Primitive::First),
            1 => Some(Primitive::Second),
            _ => None,
        }
    }
}

#[test]
fn one_basis_implementation_supports_all_methods_without_scalar_traits() {
    let basis = CustomBasis;
    let elements: Vec<_> = basis.elements().collect();
    assert_eq!(elements.len(), 2);
    assert_eq!(basis.index(&elements[0]), Some(0));
    assert_eq!(basis.index(&elements[1]), Some(1));
    assert_eq!(basis.index(&Primitive::Sum), None);
    let expansion = basis.expand(&Primitive::Sum).unwrap();
    assert_eq!(expansion.len(), 2);
    for (i, term) in expansion.into_iter().enumerate() {
        assert_eq!(basis.index(&term.operand), Some(i));
        assert_eq!(term.coefficient.0, 1);
    }
}
