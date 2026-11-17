/// Ordered binary multiplication; the right operand acts first.
#[derive(Clone, Debug, PartialEq)]
pub struct ProductExpr<E> {
    /// Left operand.
    pub lhs: E,
    /// Right operand.
    pub rhs: E,
}

/// The left action of an operator on a state.
#[derive(Clone, Debug, PartialEq)]
pub struct ActExpr<S, O> {
    /// State receiving the action.
    pub state: S,
    /// Operator acting on the state.
    pub operator: O,
}

/// An unevaluated adjoint.
#[derive(Clone, Debug, PartialEq)]
pub struct AdjointExpr<E> {
    /// Operand whose adjoint is taken.
    pub operand: E,
}
