//! Borrowed, iterative mathematical formatting; never runs algebra rules.

use alloc::{collections::BTreeMap, vec, vec::Vec};
use core::{fmt, fmt::Write, slice};

use super::{
    LocalOperator, LocalState, Operator, State,
    composite::{OpExpr, OpNode, StateExpr, StateNode as CompositeState},
    local::{OperatorNode, StateNode},
};

impl<S: fmt::Display, O: fmt::Display, C: fmt::Display> fmt::Display for LocalState<S, O, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display::<&str, S, O, C>(Expr::LocalState(self, None), f)
    }
}

impl<O: fmt::Display, C: fmt::Display> fmt::Display for LocalOperator<O, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display::<&str, &str, O, C>(Expr::LocalOperator(self), f)
    }
}

impl<I: fmt::Display, S: fmt::Display, O: fmt::Display, C: fmt::Display> fmt::Display
    for State<I, S, O, C>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display(Expr::State(&self.expression), f)
    }
}

impl<I: fmt::Display, O: fmt::Display, C: fmt::Display> fmt::Display for Operator<I, O, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display::<I, &str, O, C>(Expr::Operator(&self.expression), f)
    }
}

// Dummy &str parameters in the local/operator entry points have Display but
// never occur as values. They avoid imposing unused public index/state bounds.
enum Expr<'a, I, S, O, C> {
    LocalState(&'a LocalState<S, O, C>, Option<&'a I>),
    LocalOperator(&'a LocalOperator<O, C>),
    PlacedOperator(&'a LocalOperator<O, C>, &'a I),
    State(&'a StateExpr<I, S, O, C>),
    Operator(&'a OpExpr<I, O, C>),
}

impl<I, S, O, C> Copy for Expr<'_, I, S, O, C> {}
impl<I, S, O, C> Clone for Expr<'_, I, S, O, C> {
    fn clone(&self) -> Self {
        *self
    }
}

enum Children<'a, I, S, O, C> {
    LocalStates(slice::Iter<'a, LocalState<S, O, C>>, Option<&'a I>),
    LocalOperators(slice::Iter<'a, LocalOperator<O, C>>),
    States(slice::Iter<'a, StateExpr<I, S, O, C>>),
    Operators(slice::Iter<'a, OpExpr<I, O, C>>),
}

impl<'a, I, S, O, C> Iterator for Children<'a, I, S, O, C> {
    type Item = Expr<'a, I, S, O, C>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::LocalStates(iter, index) => iter.next().map(|s| Expr::LocalState(s, *index)),
            Self::LocalOperators(iter) => iter.next().map(Expr::LocalOperator),
            Self::States(iter) => iter.next().map(Expr::State),
            Self::Operators(iter) => iter.next().map(Expr::Operator),
        }
    }
}

enum View<'a, I, S, O, C> {
    Ket(&'a S, Option<&'a I>),
    Primitive(&'a O),
    Zero,
    Identity,
    Unit,
    Scale(&'a C, Expr<'a, I, S, O, C>),
    Sum(Children<'a, I, S, O, C>),
    Product(Expr<'a, I, S, O, C>, Expr<'a, I, S, O, C>),
    Action(Expr<'a, I, S, O, C>, Expr<'a, I, S, O, C>),
    Adjoint(Expr<'a, I, S, O, C>),
    PlacedOperator(&'a LocalOperator<O, C>, &'a I),
    Tensor(&'a [StateExpr<I, S, O, C>]),
    TensorKet(&'a StateExpr<I, S, O, C>),
}

#[derive(Clone, Copy)]
enum Kind {
    Atom,
    Sum,
    Scale,
    Product,
    Action,
    Adjoint,
    Tensor,
}

impl<I, S, O, C> View<'_, I, S, O, C> {
    fn kind(&self) -> Kind {
        match self {
            Self::Scale(..) => Kind::Scale,
            Self::Sum(_) => Kind::Sum,
            Self::Product(..) => Kind::Product,
            Self::Action(..) => Kind::Action,
            Self::Adjoint(_) => Kind::Adjoint,
            Self::Tensor(_) => Kind::Tensor,
            _ => Kind::Atom,
        }
    }
}

#[derive(Clone, Copy)]
enum Context {
    Root,
    Sum,
    ProductLeft,
    ProductRight,
    Scale,
    Tensor,
    ActionOperator,
    ActionState,
    Placement,
    Adjoint,
}

impl Context {
    fn brackets(self, kind: Kind) -> bool {
        use Kind::*;
        match self {
            Self::Root => false,
            Self::Sum => matches!(kind, Sum),
            Self::ProductLeft => matches!(kind, Sum | Scale | Action | Tensor),
            Self::ProductRight => matches!(kind, Sum | Scale | Product | Action | Tensor),
            Self::Scale => matches!(kind, Sum | Scale | Product | Action | Tensor),
            Self::Tensor => matches!(kind, Sum | Scale | Action | Tensor),
            Self::ActionOperator => matches!(kind, Sum | Scale | Product),
            Self::ActionState => matches!(kind, Sum | Scale | Action | Tensor),
            Self::Placement => !matches!(kind, Atom),
            Self::Adjoint => true,
        }
    }
}

// Classification belongs to this one formatting call. Keys identify shared
// syntax nodes, not Index values, and are never dereferenced as raw pointers.
// Remembering both true and false avoids quadratic rescans of deep mixed tensors.
struct PureTensors<I, S, O, C> {
    known: BTreeMap<*const CompositeState<I, S, O, C>, bool>,
}

enum Scan<'a, I, S, O, C> {
    Visit(&'a StateExpr<I, S, O, C>),
    Children(slice::Iter<'a, StateExpr<I, S, O, C>>),
    Finish(&'a StateExpr<I, S, O, C>),
}

impl<I, S, O, C> PureTensors<I, S, O, C> {
    fn new() -> Self {
        Self {
            known: BTreeMap::new(),
        }
    }

    fn is_pure(&mut self, root: &StateExpr<I, S, O, C>) -> bool {
        let key = root.node() as *const _;
        if let Some(pure) = self.known.get(&key) {
            return *pure;
        }
        let mut pending = vec![Scan::Visit(root)];
        while let Some(task) = pending.pop() {
            match task {
                Scan::Visit(state) => {
                    let key = state.node() as *const _;
                    if self.known.contains_key(&key) {
                        continue;
                    }
                    match state.node() {
                        CompositeState::Local(_, local) => {
                            self.known
                                .insert(key, matches!(local.node(), StateNode::Primitive(_)));
                        }
                        CompositeState::Tensor(children) => {
                            pending.push(Scan::Finish(state));
                            pending.push(Scan::Children(children.iter()));
                        }
                        _ => {
                            self.known.insert(key, false);
                        }
                    }
                }
                Scan::Children(mut children) => {
                    if let Some(child) = children.next() {
                        pending.push(Scan::Children(children));
                        pending.push(Scan::Visit(child));
                    }
                }
                Scan::Finish(state) => {
                    let CompositeState::Tensor(children) = state.node() else {
                        unreachable!("tensor classification frame");
                    };
                    let pure = !children.is_empty()
                        && children.iter().all(|s| self.known[&(s.node() as *const _)]);
                    self.known.insert(state.node() as *const _, pure);
                }
            }
        }
        self.known[&key]
    }
}

fn view<'a, I, S, O, C>(
    mut expr: Expr<'a, I, S, O, C>,
    pure: &mut PureTensors<I, S, O, C>,
) -> View<'a, I, S, O, C> {
    loop {
        return match expr {
            Expr::LocalState(state, index) => match state.node() {
                StateNode::Primitive(s) => View::Ket(s, index),
                StateNode::Scale(s) => {
                    View::Scale(&s.coefficient, Expr::LocalState(&s.operand, index))
                }
                StateNode::Sum(s) if s.is_empty() => View::Zero,
                StateNode::Sum(s) => View::Sum(Children::LocalStates(s.iter(), index)),
                StateNode::Act(a) => View::Action(
                    match index {
                        Some(i) => Expr::PlacedOperator(&a.operator, i),
                        None => Expr::LocalOperator(&a.operator),
                    },
                    Expr::LocalState(&a.state, index),
                ),
            },
            Expr::LocalOperator(op) => match op.node() {
                OperatorNode::Primitive(p) => View::Primitive(p),
                OperatorNode::Scale(s) => {
                    View::Scale(&s.coefficient, Expr::LocalOperator(&s.operand))
                }
                OperatorNode::Sum(s) if s.is_empty() => View::Zero,
                OperatorNode::Sum(s) => View::Sum(Children::LocalOperators(s.iter())),
                OperatorNode::Product(p) => {
                    View::Product(Expr::LocalOperator(&p.lhs), Expr::LocalOperator(&p.rhs))
                }
                OperatorNode::Adjoint(a) => View::Adjoint(Expr::LocalOperator(&a.operand)),
            },
            Expr::PlacedOperator(op, index) => View::PlacedOperator(op, index),
            Expr::State(state) => match state.node() {
                CompositeState::Local(factor, local) => {
                    expr = Expr::LocalState(local, factor.factor_index());
                    continue;
                }
                CompositeState::Unit => View::Unit,
                CompositeState::Scale(s) => View::Scale(&s.coefficient, Expr::State(&s.operand)),
                CompositeState::Sum(s) if s.is_empty() => View::Zero,
                CompositeState::Sum(s) => View::Sum(Children::States(s.iter())),
                CompositeState::Tensor(_) if pure.is_pure(state) => View::TensorKet(state),
                CompositeState::Tensor(s) => View::Tensor(s.as_slice()),
                CompositeState::Act(a) => {
                    View::Action(Expr::Operator(&a.operator), Expr::State(&a.state))
                }
            },
            Expr::Operator(op) => match op.node() {
                OpNode::Local(factor, local) => {
                    View::PlacedOperator(local, factor.factor_index().expect("placed factor"))
                }
                OpNode::Identity => View::Identity,
                OpNode::Scale(s) => View::Scale(&s.coefficient, Expr::Operator(&s.operand)),
                OpNode::Sum(s) if s.is_empty() => View::Zero,
                OpNode::Sum(s) => View::Sum(Children::Operators(s.iter())),
                OpNode::Product(p) => View::Product(Expr::Operator(&p.lhs), Expr::Operator(&p.rhs)),
                OpNode::Adjoint(a) => View::Adjoint(Expr::Operator(&a.operand)),
            },
        };
    }
}

struct KetLeaves<'a, I, S, O, C> {
    pending: Vec<Scan<'a, I, S, O, C>>,
}

impl<'a, I, S, O, C> KetLeaves<'a, I, S, O, C> {
    fn new(root: &'a StateExpr<I, S, O, C>) -> Self {
        Self {
            pending: vec![Scan::Visit(root)],
        }
    }
}

impl<'a, I, S, O, C> Iterator for KetLeaves<'a, I, S, O, C> {
    type Item = (&'a S, &'a I);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(task) = self.pending.pop() {
            match task {
                Scan::Visit(s) => match s.node() {
                    CompositeState::Tensor(children) => {
                        self.pending.push(Scan::Children(children.iter()));
                    }
                    CompositeState::Local(factor, local) => {
                        let StateNode::Primitive(p) = local.node() else {
                            unreachable!("classified primitive ket");
                        };
                        return Some((p, factor.factor_index().expect("placed factor")));
                    }
                    _ => unreachable!("classified tensor ket"),
                },
                Scan::Children(mut children) => {
                    if let Some(child) = children.next() {
                        self.pending.push(Scan::Children(children));
                        self.pending.push(Scan::Visit(child));
                    }
                }
                Scan::Finish(_) => unreachable!("ket traversal has no postorder frames"),
            }
        }
        None
    }
}

enum Task<'a, I, S, O, C> {
    Expr(Expr<'a, I, S, O, C>, Context, usize),
    Text(&'static str),
    Value(&'a dyn fmt::Display),
    Index(&'a I),
    Line(usize),
    Sequence(Children<'a, I, S, O, C>, bool, bool, usize),
    Ket(KetLeaves<'a, I, S, O, C>, bool, usize),
}

fn line(f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
    f.write_char('\n')?;
    for _ in 0..indent {
        f.write_str("    ")?;
    }
    Ok(())
}

fn display<I: fmt::Display, S: fmt::Display, O: fmt::Display, C: fmt::Display>(
    root: Expr<'_, I, S, O, C>,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    let pretty = f.alternate();
    let mut pure = PureTensors::new();
    let mut pending = vec![Task::Expr(root, Context::Root, 0)];
    while let Some(task) = pending.pop() {
        match task {
            Task::Text(text) => f.write_str(text)?,
            // A fresh formatter keeps outer precision/width/alternate flags
            // local to the expression rather than forwarding them to leaves.
            Task::Value(value) => write!(f, "{value}")?,
            Task::Index(index) => write!(
                &mut IndexPrefix {
                    out: f,
                    remaining: 8,
                },
                "{index}"
            )?,
            Task::Line(indent) => line(f, indent)?,
            Task::Sequence(mut children, first, tensor, indent) => {
                if let Some(child) = children.next() {
                    pending.push(Task::Sequence(children, false, tensor, indent));
                    pending.push(Task::Expr(
                        child,
                        if tensor {
                            Context::Tensor
                        } else {
                            Context::Sum
                        },
                        indent,
                    ));
                    if !first {
                        if pretty {
                            pending.push(Task::Text(if tensor { "⊗ " } else { "+ " }));
                            pending.push(Task::Line(indent));
                        } else {
                            pending.push(Task::Text(if tensor { " ⊗ " } else { " + " }));
                        }
                    }
                }
            }
            Task::Ket(mut leaves, first, indent) => {
                if let Some((state, index)) = leaves.next() {
                    pending.push(Task::Ket(leaves, false, indent));
                    pending.push(Task::Text("]"));
                    pending.push(Task::Index(index));
                    pending.push(Task::Text("["));
                    pending.push(Task::Value(state));
                    if !first {
                        if pretty {
                            pending.push(Task::Line(indent + 1));
                        }
                        pending.push(Task::Text(","));
                    }
                }
            }
            Task::Expr(expr, context, indent) => {
                let node = view(expr, &mut pure);
                if context.brackets(node.kind()) {
                    f.write_str("(")?;
                    pending.push(Task::Text(")"));
                    if pretty && matches!(node.kind(), Kind::Sum | Kind::Tensor) {
                        pending.push(Task::Line(indent));
                        pending.push(Task::Expr(expr, Context::Root, indent + 1));
                        line(f, indent + 1)?;
                    } else {
                        pending.push(Task::Expr(expr, Context::Root, indent));
                    }
                    continue;
                }
                match node {
                    View::Zero => f.write_str("0")?,
                    View::Identity => f.write_str("I")?,
                    View::Unit => f.write_str("unit")?,
                    View::Primitive(p) => pending.push(Task::Value(p)),
                    View::Ket(state, index) => {
                        f.write_str("|")?;
                        pending.push(Task::Text(">"));
                        if let Some(index) = index {
                            pending.push(Task::Text("]"));
                            pending.push(Task::Index(index));
                            pending.push(Task::Text("["));
                        }
                        pending.push(Task::Value(state));
                    }
                    View::Scale(coefficient, operand) => {
                        f.write_str("(")?;
                        pending.push(Task::Expr(operand, Context::Scale, indent));
                        pending.push(Task::Text(") * "));
                        pending.push(Task::Value(coefficient));
                    }
                    View::Sum(children) => {
                        pending.push(Task::Sequence(children, true, false, indent));
                    }
                    View::Product(lhs, rhs) => {
                        pending.push(Task::Expr(rhs, Context::ProductRight, indent));
                        pending.push(Task::Text(" * "));
                        pending.push(Task::Expr(lhs, Context::ProductLeft, indent));
                    }
                    View::Action(operator, state) => {
                        pending.push(Task::Expr(state, Context::ActionState, indent));
                        pending.push(Task::Text(" "));
                        pending.push(Task::Expr(operator, Context::ActionOperator, indent));
                    }
                    View::Adjoint(operand) => {
                        pending.push(Task::Text("†"));
                        pending.push(Task::Expr(operand, Context::Adjoint, indent));
                    }
                    View::PlacedOperator(op, index) => {
                        pending.push(Task::Text("]"));
                        pending.push(Task::Index(index));
                        pending.push(Task::Text("["));
                        pending.push(Task::Expr(
                            Expr::LocalOperator(op),
                            Context::Placement,
                            indent,
                        ));
                    }
                    View::Tensor(children) => {
                        pending.push(Task::Sequence(
                            Children::States(children.iter()),
                            true,
                            true,
                            indent,
                        ));
                    }
                    View::TensorKet(root) => {
                        f.write_str("|")?;
                        pending.push(Task::Text(">"));
                        pending.push(Task::Ket(KetLeaves::new(root), true, indent));
                    }
                }
            }
        }
    }
    Ok(())
}

struct IndexPrefix<'a, W: ?Sized> {
    out: &'a mut W,
    remaining: usize,
}

impl<W: fmt::Write + ?Sized> fmt::Write for IndexPrefix<'_, W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.remaining == 0 {
            return Ok(());
        }
        let end = s
            .char_indices()
            .nth(self.remaining)
            .map_or(s.len(), |(end, _)| end);
        let prefix = &s[..end];
        self.out.write_str(prefix)?;
        self.remaining -= prefix.chars().count();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HilbertSpace, expr::parts::SumExpr};
    use alloc::string::String;

    #[test]
    fn pure_tensor_grouping_and_shared_classification_are_iterative() {
        std::thread::Builder::new()
            .stack_size(128 * 1024)
            .spawn(|| {
                type E = StateExpr<&'static str, &'static str, &'static str, i32>;
                let factor = HilbertSpace::factor("a");
                let local = LocalState::primitive("up");
                let leaf = E::new(CompositeState::Local(factor.clone(), local));
                let mut grouped = leaf;
                let mut shared = E::new(CompositeState::Unit);
                for _ in 0..20_000 {
                    // Public constructors elide unary tensors. Exercise the
                    // formatter on retained grouping without building 20k spaces.
                    grouped = E::new(CompositeState::Tensor(SumExpr::One(grouped)));
                    // Classification must visit shared nodes once even when
                    // displaying every occurrence would have exponential size.
                    shared = E::new(CompositeState::Tensor(
                        [shared.clone(), shared].into_iter().collect(),
                    ));
                }
                let mut pure = PureTensors::new();
                assert!(!pure.is_pure(&shared));
                assert_eq!(pure.known.len(), 20_001);
                assert!(!pure.is_pure(&shared));
                assert_eq!(pure.known.len(), 20_001);
                assert!(pure.is_pure(&grouped));
                assert_eq!(pure.known.len(), 40_002);
                let state = State {
                    expression: grouped,
                    space: factor,
                };
                let mut out = String::new();
                write!(out, "{state}").unwrap();
                assert_eq!(out, "|up[a]>");
            })
            .unwrap()
            .join()
            .unwrap();
    }
}
