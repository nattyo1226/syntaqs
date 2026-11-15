use super::{
    composite::{OpNode, Operator},
    local::OperatorNode,
};
use crate::{ExpressionError, HilbertSpace as H, Identity, LocalOperator};
use alloc::{rc::Rc, vec};
use core::{cell::Cell, ptr};

// Deliberately no Clone, PartialEq, Product, Adjoint, grading, or Dimension.
struct Primitive(u8);
impl Identity for Primitive {
    fn identity() -> Self {
        Self(0)
    }
}
#[derive(PartialEq, Eq)]
struct Index(u8);
struct OpaqueCoefficient;
type Bare = Operator<Index, Primitive, OpaqueCoefficient>;
fn bare(value: u8, factor: &H<Index>) -> Bare {
    LocalOperator::primitive(Primitive(value))
        .at(factor)
        .unwrap()
}

#[test]
fn immediate_construction_keeps_opaque_capabilities_and_order() {
    fn construct<I: Eq, O, C>(a: Operator<I, O, C>, b: Operator<I, O, C>) {
        let _ = (a.clone() + &b).unwrap();
        let _ = (a * b).unwrap();
    }
    struct NoIdentity;
    let a = H::factor(Index(1));
    let same = H::factor(Index(1));
    let b = H::factor(Index(2));
    construct(
        LocalOperator::<_, OpaqueCoefficient>::primitive(NoIdentity)
            .at(&a)
            .unwrap(),
        LocalOperator::primitive(NoIdentity).at(&same).unwrap(),
    );
    let sum = Bare::sum([bare(1, &a), bare(2, &same), bare(3, &a)]).unwrap();
    let OpNode::Local(_, local) = sum.expression.node() else {
        panic!("local sum")
    };
    let OperatorNode::Sum(parts) = local.node() else {
        panic!("sum retained")
    };
    for (child, expected) in parts.iter().zip([1, 2, 3]) {
        assert!(matches!(child.node(), OperatorNode::Primitive(Primitive(p)) if *p == expected));
    }
    let product = (bare(1, &a) * bare(2, &same)).unwrap();
    let OpNode::Local(_, local) = product.expression.node() else {
        panic!("local product")
    };
    let OperatorNode::Product(p) = local.node() else {
        panic!("product retained")
    };
    assert!(matches!(
        p.lhs.node(),
        OperatorNode::Primitive(Primitive(1))
    ));
    assert!(matches!(
        p.rhs.node(),
        OperatorNode::Primitive(Primitive(2))
    ));
    assert!(matches!(
        (bare(1, &a) + bare(2, &b)).unwrap().expression.node(),
        OpNode::Sum(_)
    ));
    let chain = Bare::product([bare(1, &a), bare(2, &b), bare(3, &a)]).unwrap();
    assert!(matches!(chain.expression.node(), OpNode::Product(_)));

    let ha = H::tensor_all([a.clone()]).unwrap();
    let hb = H::tensor_all([b.clone()]).unwrap();
    assert!(matches!(
        bare(1, &a).bind(&ha).unwrap() + bare(2, &b).bind(&hb).unwrap(),
        Err(ExpressionError::IncompatibleSpaces)
    ));
    assert!(matches!(Bare::sum([]).unwrap().expression.node(), OpNode::Sum(s) if s.is_empty()));
    assert!(matches!(
        Bare::product([]).unwrap().expression.node(),
        OpNode::Identity
    ));
}

#[derive(Clone)]
struct Coefficient;
type Op = Operator<Index, Primitive, Coefficient>;
fn op(value: u8, factor: &H<Index>) -> Op {
    LocalOperator::primitive(Primitive(value))
        .at(factor)
        .unwrap()
}

#[test]
fn explicit_grouping_preserves_scalars_binding_references_and_idempotence() {
    let a = H::factor(Index(8));
    let h = H::tensor_all([a.clone()]).unwrap();
    let source = ((Op::scalar(Coefficient) + op(1, &a).scaled(Coefficient)).unwrap()
        * op(2, &a).adjoint())
    .unwrap()
    .adjoint()
    .bind(&h)
    .unwrap();
    let grouped = source.group_local();
    assert!(
        source
            .space()
            .unwrap()
            .shares_node_with(grouped.space().unwrap())
    );
    assert!(matches!(source.expression.node(), OpNode::Adjoint(_)));
    assert!(matches!(grouped.expression.node(), OpNode::Local(_, _)));
    assert!(ptr::eq(
        grouped.expression.node(),
        grouped.group_local().expression.node()
    ));
    let unbound = (op(1, &a).scaled(Coefficient) + Op::zero())
        .unwrap()
        .group_local();
    assert!(unbound.space().is_none());
    assert!(matches!(unbound.expression.node(), OpNode::Local(_, _)));

    let scalar = (Op::scalar(Coefficient).adjoint() * Op::identity()).unwrap();
    assert!(ptr::eq(
        scalar.expression.node(),
        scalar.group_local().expression.node()
    ));
    // A structural zero never erases an invalid reference.
    let zero = (Op::zero() * op(1, &a)).unwrap().group_local();
    assert!(matches!(zero.expression.node(), OpNode::Local(_, _)));
    assert!(matches!(
        zero.bind(&H::unit()),
        Err(ExpressionError::MissingIndex)
    ));
    let b = H::factor(Index(9));
    let multi = Op::product([op(1, &a), op(2, &b), op(3, &a)]).unwrap();
    assert!(ptr::eq(
        multi.expression.node(),
        multi.group_local().expression.node()
    ));
}

struct CountClone(Rc<Cell<usize>>);
impl Clone for CountClone {
    fn clone(&self) -> Self {
        self.0.set(self.0.get() + 1);
        Self(self.0.clone())
    }
}

#[test]
fn shared_scalar_and_single_factor_subtrees_are_converted_once() {
    type Counted = Operator<Index, Primitive, CountClone>;
    let clones = Rc::new(Cell::new(0));
    let a = H::factor(Index(1));
    let b = H::factor(Index(2));
    let scalar = Counted::scalar(CountClone(clones.clone()));
    let shared = (scalar.clone() + LocalOperator::primitive(Primitive(1)).at(&a).unwrap()).unwrap();
    let root = Counted::sum(vec![
        shared.clone(),
        shared,
        (scalar + LocalOperator::primitive(Primitive(2)).at(&b).unwrap()).unwrap(),
    ])
    .unwrap();
    let grouped = root.group_local();
    assert_eq!(
        clones.get(),
        1,
        "shared scalar translation is memoized across factors"
    );
    let OpNode::Sum(parts) = grouped.expression.node() else {
        panic!("multi-factor sum")
    };
    assert!(ptr::eq(
        parts.as_slice()[0].node(),
        parts.as_slice()[1].node()
    ));
}

#[test]
fn deep_grouping_and_destruction_use_work_stacks() {
    std::thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(|| {
            let a = H::factor(Index(1));
            let mut local = op(1, &a);
            for _ in 0..20_000 {
                local = local.adjoint();
            }
            let grouped = local.group_local();
            assert!(matches!(grouped.expression.node(), OpNode::Local(_, _)));
            drop(local);
            drop(grouped);
            let mut scalar = Op::identity();
            for _ in 0..20_000 {
                scalar = scalar.adjoint();
            }
            let source = (scalar + op(1, &a)).unwrap();
            let grouped = source.group_local();
            assert!(matches!(grouped.expression.node(), OpNode::Local(_, _)));
            drop(source);
            drop(grouped);
            let b = H::factor(Index(2));
            let mut multi = (op(1, &a).scaled(Coefficient) + op(2, &b)).unwrap();
            for _ in 0..20_000 {
                multi = multi.adjoint();
            }
            let grouped = multi.group_local();
            drop(multi);
            drop(grouped);
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn unchanged_shared_products_do_not_expand_during_preparation() {
    let a = H::factor(Index(1));
    let b = H::factor(Index(2));
    let mut shared = (op(1, &a) * op(2, &b)).unwrap();
    // Only 67 composite nodes, despite exponentially many occurrence paths.
    for _ in 0..64 {
        shared = (&shared * &shared).unwrap();
    }
    assert!(ptr::eq(
        shared.expression.node(),
        shared.group_local().expression.node()
    ));
}
