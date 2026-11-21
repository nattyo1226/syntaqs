//! A choice of primitives and coefficient type, independent of space and geometry.

use crate::{ExpressionError, HilbertSpace, LocalOperator, LocalState, Operator, State};

/// Associates state/operator primitives and coefficients without construction methods.
///
/// Use [`ExpressionFactory`] for inherent constructors, or these associated types
/// and the `*Of` aliases in generic APIs. No algebraic capabilities are required.
pub trait QuantumSystem: Sized {
    /// State primitive, rather than a state expression.
    type State;
    /// Operator primitive, rather than an operator expression.
    type Operator;
    /// The whole coefficient type, including any complex components.
    type Coeff;
}

type FactoryMarker<S, O, C> = core::marker::PhantomData<fn() -> (S, O, C)>;

/// Constructs local and composite expressions for selected primitive and coefficient types.
///
/// No evaluation, numeric or cloning capabilities are required. Those bounds
/// belong to the operations on the returned expressions. Systems do not own
/// spaces or indices, and do not become part of expression identity: two systems
/// with the same associated types construct the same expression types.
///
/// ```
/// use syntaqs_core::{system::ExpressionFactory, system::LocalStateOf};
/// // A custom algebra can opt into construction before providing evaluation.
/// struct Ket;
/// struct Op;
/// struct Coeff;
/// type Custom = ExpressionFactory<Ket, Op, Coeff>;
/// let ket: LocalStateOf<Custom> = Custom::state(Ket);
/// let delayed = ket.act(&Custom::operator(Op));
/// let a = syntaqs_core::space::HilbertSpace::factor("a");
/// let b = syntaqs_core::space::HilbertSpace::factor("b");
/// let space = a.tensor(&b)?;
/// let state = Custom::state_tensor_from([Ket, Ket], &space)?;
/// let operator = Custom::try_operator_sum([Custom::operator_product_from([(Op, &a)])])?;
/// let delayed = state.act(&operator)?;
/// # Ok::<(), syntaqs_core::expression::error::ExpressionError>(())
/// ```
/// Its constructors are inherent methods and require no trait import. Operations on an existing
/// expression (scaling, placement, binding and evaluation) remain methods of the
/// expression. Type aliases such as [`StateOf`] are primarily for annotations.
///
/// All `try_` collections accept `ExpressionError`, stop at the first input
/// error, and validate composition only after all inputs succeed. An input
/// error therefore precedes any space errors in earlier successful operands.
/// Only iterator consumption stops: work already done to build an array or
/// vector is not rolled back.
pub struct ExpressionFactory<S, O, C>(FactoryMarker<S, O, C>);

impl<S, O, C> QuantumSystem for ExpressionFactory<S, O, C> {
    type State = S;
    type Operator = O;
    type Coeff = C;
}

impl<S, O, C> ExpressionFactory<S, O, C> {
    /// Holds a state primitive without evaluating it.
    pub fn state(primitive: S) -> LocalStateOf<Self> {
        LocalState::primitive(primitive)
    }

    /// Holds an operator primitive without evaluating it.
    pub fn operator(primitive: O) -> LocalOperatorOf<Self> {
        LocalOperator::primitive(primitive)
    }

    /// Tensors placed states on an existing space without creating a new space.
    pub fn state_tensor<I: Eq>(
        states: impl IntoIterator<Item = StateOf<Self, I>>,
        space: &HilbertSpace<I>,
    ) -> Result<StateOf<Self, I>, ExpressionError> {
        State::tensor_in(states, space)
    }

    /// Collects fallible states before validating their explicit target space.
    pub fn try_state_tensor<I: Eq>(
        states: impl IntoIterator<Item = Result<StateOf<Self, I>, ExpressionError>>,
        space: &HilbertSpace<I>,
    ) -> Result<StateOf<Self, I>, ExpressionError> {
        State::try_tensor_in(states, space)
    }

    /// Builds a tensor product from state primitives in the space's leaf order.
    /// A count other than one primitive per leaf returns `IncompatibleSpaces`.
    /// No coefficient arithmetic, evaluation or primitive cloning is required.
    pub fn state_tensor_from<I: Eq>(
        primitives: impl IntoIterator<Item = S>,
        space: &HilbertSpace<I>,
    ) -> Result<StateOf<Self, I>, ExpressionError> {
        let mut primitives = primitives.into_iter();
        let state = Self::try_state_tensor(
            space.leaves().map(|factor| {
                let primitive = primitives
                    .next()
                    .ok_or(ExpressionError::IncompatibleSpaces)?;
                Self::state(primitive).at(factor)
            }),
            space,
        )?;
        if primitives.next().is_some() {
            return Err(ExpressionError::IncompatibleSpaces);
        }
        Ok(state)
    }

    /// Places the primitives in `(primitive, &factor)` pairs and multiplies them in input order.
    ///
    /// Repeated indices are allowed. The rightmost operator acts first.
    /// The result is unbound, including identity for empty input; use `bind`
    /// to select an ambient space. The first placement error stops iteration.
    pub fn operator_product_from<'a, I: Eq + 'a>(
        assignments: impl IntoIterator<Item = (O, &'a HilbertSpace<I>)>,
    ) -> Result<OperatorOf<Self, I>, ExpressionError> {
        Self::try_operator_product(
            assignments
                .into_iter()
                .map(|(primitive, factor)| Self::operator(primitive).at(factor)),
        )
    }

    /// Sums placed states on `space`, including zero for empty input.
    pub fn state_sum<I: Eq>(
        states: impl IntoIterator<Item = StateOf<Self, I>>,
        space: &HilbertSpace<I>,
    ) -> Result<StateOf<Self, I>, ExpressionError> {
        State::sum(states, space)
    }

    /// Sums fallible states; the first input error precedes space validation.
    /// Later inputs are not pulled after an error. Empty input is zero on `space`.
    pub fn try_state_sum<I: Eq>(
        states: impl IntoIterator<Item = Result<StateOf<Self, I>, ExpressionError>>,
        space: &HilbertSpace<I>,
    ) -> Result<StateOf<Self, I>, ExpressionError> {
        State::try_sum(states, space)
    }

    /// Sums operators, validating bindings only if every operand is bound.
    /// Empty input gives unbound zero.
    pub fn operator_sum<I: Eq>(
        operators: impl IntoIterator<Item = OperatorOf<Self, I>>,
    ) -> Result<OperatorOf<Self, I>, ExpressionError> {
        Operator::sum(operators)
    }

    /// Sums fallible operators, returning the first input error before validation.
    /// Later inputs are not pulled after an error. Empty input is unbound zero.
    pub fn try_operator_sum<I: Eq>(
        operators: impl IntoIterator<Item = Result<OperatorOf<Self, I>, ExpressionError>>,
    ) -> Result<OperatorOf<Self, I>, ExpressionError> {
        Operator::try_sum(operators)
    }

    /// Multiplies placed operators in input order; the rightmost acts first.
    /// Bindings follow the same rules as sums; empty input is unbound identity.
    pub fn operator_product<I: Eq>(
        operators: impl IntoIterator<Item = OperatorOf<Self, I>>,
    ) -> Result<OperatorOf<Self, I>, ExpressionError> {
        Operator::product(operators)
    }

    /// Multiplies fallible operators, with input errors preceding binding errors.
    /// The first input error stops iteration. Empty input is unbound identity.
    pub fn try_operator_product<I: Eq>(
        operators: impl IntoIterator<Item = Result<OperatorOf<Self, I>, ExpressionError>>,
    ) -> Result<OperatorOf<Self, I>, ExpressionError> {
        Operator::try_product(operators)
    }
}

/// A local state type annotation for `Q`; construct it with [`ExpressionFactory::state`].
pub type LocalStateOf<Q> = LocalState<
    <Q as QuantumSystem>::State,
    <Q as QuantumSystem>::Operator,
    <Q as QuantumSystem>::Coeff,
>;

/// A local operator type annotation for `Q`; use [`ExpressionFactory::operator`] to construct it.
pub type LocalOperatorOf<Q> =
    LocalOperator<<Q as QuantumSystem>::Operator, <Q as QuantumSystem>::Coeff>;

/// A composite state type annotation for `Q` and independent index `I`.
/// Construct it with methods such as [`ExpressionFactory::state_tensor_from`].
pub type StateOf<Q, I> = State<
    I,
    <Q as QuantumSystem>::State,
    <Q as QuantumSystem>::Operator,
    <Q as QuantumSystem>::Coeff,
>;

/// A composite operator type annotation for `Q` and independent index `I`.
/// Construct it with methods such as [`ExpressionFactory::operator_product_from`].
pub type OperatorOf<Q, I> =
    Operator<I, <Q as QuantumSystem>::Operator, <Q as QuantumSystem>::Coeff>;
