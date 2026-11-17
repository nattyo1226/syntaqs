/// A coefficient and its operand, without numeric bounds on storage.
#[derive(Clone, Debug, PartialEq)]
pub struct ScaleExpr<E, C> {
    /// The coefficient, including explicitly stored zero and one.
    pub coefficient: C,
    /// The scaled operand.
    pub operand: E,
}
