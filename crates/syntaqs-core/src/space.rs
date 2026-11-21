//! Immutable ordinary and graded tensor spaces, generic over an equality-comparable index.

use alloc::{
    collections::{BinaryHeap, binary_heap::PeekMut},
    sync::Arc,
    vec,
    vec::Vec,
};
use core::{
    cmp::{Ordering, Reverse},
    fmt,
    iter::FusedIterator,
    num::NonZeroUsize,
    slice,
};

use crate::expression::parts::Operands;

/// Dimension of the linear space spanned by all values of a primitive type.
///
/// For a complete state space this is `d`; for its full operator space it is
/// `d²`. It is not the number of primitive variants or the span of one value.
/// Dimensions must agree with the algebra, action, and complete basis contracts.
pub trait Dimension {
    /// Returns the positive dimension, or an error if it does not fit in usize.
    fn dim() -> Result<NonZeroUsize, DimensionOverflow>;
}

/// A state-space or operator-space dimension does not fit in `usize`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("space dimension overflows usize")]
pub struct DimensionOverflow;

/// Composition rule for an ordered tensor product.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TensorKind {
    /// Ordinary tensor product, with commuting distinct factors.
    Ordinary,
    /// Z₂-graded tensor product, with Koszul signs for odd factors.
    Graded,
}

/// Errors constructing a tensor space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SpaceError {
    /// The same basic index occurs more than once, including inside children.
    #[error("duplicate factor index")]
    DuplicateIndex,
    /// Different effective composition rules remain after removing units.
    #[error("mixed tensor kinds")]
    MixedTensorKinds,
}

enum Node<I> {
    Factor(I),
    Tensor {
        kind: TensorKind,
        children: Operands<HilbertSpace<I>>,
        by_index: FactorIndex<I>,
    },
}

impl<I> Node<I> {
    fn index(&self) -> Option<&I> {
        match self {
            Self::Factor(index) => Some(index),
            Self::Tensor { .. } => None,
        }
    }
}

impl<I: fmt::Debug> fmt::Debug for Node<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Factor(index) => f.debug_tuple("Factor").field(index).finish(),
            Self::Tensor { kind, children, .. } => {
                f.debug_tuple("Tensor").field(kind).field(children).finish()
            }
        }
    }
}

struct FactorPosition<I> {
    factor: Arc<Node<I>>,
    position: usize,
}

// Entries refer only to factor nodes. They are ordered by ID and carry their
// positions in the tensor's independent, left-to-right leaf order.
enum FactorIndex<I> {
    Empty,
    Single(Arc<Node<I>>),
    Many(Arc<Vec<FactorPosition<I>>>),
}

impl<I> FactorIndex<I> {
    fn len(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Single(_) => 1,
            Self::Many(entries) => entries.len(),
        }
    }

    fn reuse(child: &HilbertSpace<I>) -> Self {
        match child.node() {
            Node::Factor(_) => Self::Single(child.root.as_ref().expect("live space").clone()),
            Node::Tensor { by_index, .. } => match by_index {
                Self::Empty => Self::Empty,
                Self::Single(factor) => Self::Single(factor.clone()),
                Self::Many(entries) => Self::Many(entries.clone()),
            },
        }
    }
}

impl<I: Ord> FactorIndex<I> {
    fn merge(children: &[HilbertSpace<I>]) -> Result<Self, SpaceError> {
        let mut factor_count = 0;
        let mut nonempty_count = 0;
        for child in children {
            let count = child.factor_count();
            factor_count += count;
            nonempty_count += usize::from(count != 0);
        }
        let mut entries: Vec<FactorPosition<I>> = Vec::with_capacity(factor_count);
        if factor_count == nonempty_count {
            // Singleton runs can be sorted in place without an auxiliary heap.
            for child in children {
                if let Some(run) = FactorRun::new(child, entries.len()) {
                    entries.push(run.entry());
                }
            }
            entries.sort_unstable_by(|a, b| a.index().cmp(b.index()));
            if entries
                .windows(2)
                .any(|pair| pair[0].index() == pair[1].index())
            {
                return Err(SpaceError::DuplicateIndex);
            }
        } else {
            // Merge sorted child runs without traversing descendants.
            let mut offset = 0;
            let mut runs = Vec::with_capacity(nonempty_count);
            for child in children {
                if let Some(run) = FactorRun::new(child, offset) {
                    runs.push(Reverse(run));
                    offset += child.factor_count();
                }
            }
            let mut runs = BinaryHeap::from(runs);
            while let Some(mut next) = runs.peek_mut() {
                let run = &mut next.0;
                if entries
                    .last()
                    .is_some_and(|last| last.index() == run.index())
                {
                    return Err(SpaceError::DuplicateIndex);
                }
                entries.push(run.entry());
                if let Some((head, tail)) = run.rest.split_first() {
                    run.factor = &head.factor;
                    run.position = head.position;
                    run.rest = tail;
                } else {
                    PeekMut::pop(next);
                }
            }
        }
        Ok(Self::Many(Arc::new(entries)))
    }

    fn position(&self, index: &I) -> Option<usize> {
        match self {
            Self::Empty => None,
            Self::Single(factor) => (factor.index() == Some(index)).then_some(0),
            Self::Many(entries) => entries
                .binary_search_by(|entry| entry.index().cmp(index))
                .ok()
                .map(|slot| entries[slot].position),
        }
    }
}

impl<I> FactorPosition<I> {
    fn index(&self) -> &I {
        self.factor.index().expect("indexed factor")
    }
}

// The remaining entries of one child, with their positions relative to that
// child. The offset makes those positions relative to the parent.
struct FactorRun<'a, I> {
    factor: &'a Arc<Node<I>>,
    position: usize,
    rest: &'a [FactorPosition<I>],
    offset: usize,
}

impl<'a, I> FactorRun<'a, I> {
    fn new(child: &'a HilbertSpace<I>, offset: usize) -> Option<Self> {
        let (factor, position, rest) = match child.node() {
            Node::Factor(_) => (child.root.as_ref().expect("live space"), 0, &[][..]),
            Node::Tensor { by_index, .. } => match by_index {
                FactorIndex::Empty => return None,
                FactorIndex::Single(factor) => (factor, 0, &[][..]),
                FactorIndex::Many(entries) => {
                    let (head, tail) = entries.split_first().expect("nonempty factor index");
                    (&head.factor, head.position, tail)
                }
            },
        };
        Some(Self {
            factor,
            position,
            rest,
            offset,
        })
    }

    fn index(&self) -> &I {
        self.factor.index().expect("indexed factor")
    }

    fn entry(&self) -> FactorPosition<I> {
        FactorPosition {
            factor: self.factor.clone(),
            position: self.offset + self.position,
        }
    }
}

impl<I: Ord> Ord for FactorRun<'_, I> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.index().cmp(rhs.index())
    }
}

impl<I: Ord> PartialOrd for FactorRun<'_, I> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

impl<I: Ord> PartialEq for FactorRun<'_, I> {
    fn eq(&self, rhs: &Self) -> bool {
        self.index() == rhs.index()
    }
}

impl<I: Ord> Eq for FactorRun<'_, I> {}

/// A shared factor or ordered, possibly nested tensor space.
///
/// Local primitive types and dimensions belong to expressions and their query
/// inputs. Index identity is supplied by I: Eq; cloning never clones an index.
///
/// ```
/// use core::num::NonZeroUsize;
/// use syntaqs_core::space::HilbertSpace;
/// let a = HilbertSpace::factor("a");
/// let b = HilbertSpace::factor("b");
/// let h = a.tensor(&b)?;
/// assert_eq!(h.dim_with(NonZeroUsize::new(2).unwrap())?, 4);
/// assert!(a.is_subsystem_of(&h));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug)]
pub struct HilbertSpace<I> {
    root: Option<Arc<Node<I>>>,
    effective_kind: Option<TensorKind>,
}

impl<I> Clone for HilbertSpace<I> {
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
            effective_kind: self.effective_kind,
        }
    }
}

impl<I> HilbertSpace<I> {
    fn new(node: Node<I>, effective_kind: Option<TensorKind>) -> Self {
        Self {
            root: Some(Arc::new(node)),
            effective_kind,
        }
    }

    fn new_tensor(
        kind: TensorKind,
        children: Operands<Self>,
        by_index: FactorIndex<I>,
        effective_kind: Option<TensorKind>,
    ) -> Self {
        Self::new(
            Node::Tensor {
                kind,
                children,
                by_index,
            },
            effective_kind,
        )
    }

    fn node(&self) -> &Node<I> {
        self.root.as_deref().expect("live space handle")
    }

    /// Creates a leaf from an already issued index, without generating an index.
    pub fn factor(index: I) -> Self {
        Self::new(Node::Factor(index), None)
    }

    /// Creates the empty tensor space, of dimension one.
    pub fn unit() -> Self {
        Self::new_tensor(
            TensorKind::Ordinary,
            Operands::Empty,
            FactorIndex::Empty,
            None,
        )
    }

    /// Returns the index of a leaf, or `None` for a tensor (even a singleton).
    pub fn factor_index(&self) -> Option<&I> {
        self.node().index()
    }

    /// Returns the stored tensor kind, including empty and singleton wrappers.
    pub fn kind(&self) -> Option<TensorKind> {
        match self.node() {
            Node::Factor(_) => None,
            Node::Tensor { kind, .. } => Some(*kind),
        }
    }

    /// Returns the composition rule after ignoring units and singleton wrappers.
    pub fn effective_kind(&self) -> Option<TensorKind> {
        self.effective_kind
    }

    /// Borrows the immediate children, preserving the stored grouping.
    pub fn children(&self) -> slice::Iter<'_, Self> {
        match self.node() {
            Node::Factor(_) => [].iter(),
            Node::Tensor { children, .. } => children.iter(),
        }
    }

    /// Promotes each immediate tensor child's children by one level.
    ///
    /// Factors stay in order, and an empty tensor child contributes no children.
    /// The effective tensor kind is preserved. If promotion makes the outer
    /// kind effective, it adopts the original effective kind.
    /// A factor or a tensor whose immediate children are all factors shares
    /// its original node.
    pub fn flatten(&self) -> Self {
        let Node::Tensor { kind, children, .. } = self.node() else {
            return self.clone();
        };
        if children
            .iter()
            .all(|child| matches!(child.node(), Node::Factor(_)))
        {
            return self.clone();
        }

        Self::new_flattened_tensor(
            *kind,
            children.as_slice(),
            FactorIndex::reuse(self),
            self.effective_kind,
        )
    }

    fn new_flattened_tensor(
        kind: TensorKind,
        children: &[Self],
        by_index: FactorIndex<I>,
        effective_kind: Option<TensorKind>,
    ) -> Self {
        let groups = children.iter().map(Self::promoted_children);
        let mut promoted = Vec::with_capacity(groups.clone().map(<[_]>::len).sum());
        for group in groups {
            promoted.extend_from_slice(group);
        }
        let kind = match effective_kind {
            Some(effective)
                if effective != kind
                    && promoted
                        .iter()
                        .filter(|child| child.factor_count() != 0)
                        .nth(1)
                        .is_some() =>
            {
                effective
            }
            _ => kind,
        };
        // Promotion leaves every factor at its original position.
        Self::new_tensor(kind, promoted.into(), by_index, effective_kind)
    }

    fn promoted_children(&self) -> &[Self] {
        match self.node() {
            Node::Factor(_) => slice::from_ref(self),
            Node::Tensor { children, .. } => children.as_slice(),
        }
    }

    pub(crate) fn factor_count(&self) -> usize {
        match self.node() {
            Node::Factor(_) => 1,
            Node::Tensor { by_index, .. } => by_index.len(),
        }
    }

    /// Lazily borrows all leaf indices in tensor order, using a work stack.
    pub fn indices(&self) -> Indices<'_, I> {
        Indices {
            leaves: self.leaves(),
        }
    }

    /// Lazily borrows the actual leaves in tensor order, including a leaf input.
    /// Empty tensors yield no leaves; traversal uses an explicit work stack.
    pub fn leaves(&self) -> Leaves<'_, I> {
        Leaves {
            pending: vec![self],
        }
    }

    /// Computes the tensor-space dimension using primitive P's span per factor.
    ///
    /// Use a state primitive for a state-space dimension, and an operator
    /// primitive for an operator-space dimension. The empty tensor has dimension
    /// one even when P's local dimension cannot be represented.
    pub fn dim<P: Dimension>(&self) -> Result<usize, DimensionOverflow> {
        let mut indices = self.indices();
        if indices.next().is_none() {
            return Ok(1);
        }
        let local = P::dim()?.get();
        indices.try_fold(local, |d, _| d.checked_mul(local).ok_or(DimensionOverflow))
    }

    /// Computes the dimension for an explicitly chosen uniform local space.
    ///
    /// This query does not associate a definition or primitive type with indices.
    pub fn dim_with(&self, local: core::num::NonZeroUsize) -> Result<usize, DimensionOverflow> {
        self.indices().try_fold(1usize, |d, _| {
            d.checked_mul(local.get()).ok_or(DimensionOverflow)
        })
    }

    /// Whether two handles share the same immutable space node.
    pub fn shares_node_with(&self, rhs: &Self) -> bool {
        Arc::ptr_eq(
            self.root.as_ref().expect("live space"),
            rhs.root.as_ref().expect("live space"),
        )
    }
}

impl<I: Ord> HilbertSpace<I> {
    /// Constructs a tensor, retaining children and grouping and rejecting duplicates.
    pub fn tensor_all(children: impl IntoIterator<Item = Self>) -> Result<Self, SpaceError> {
        Self::tensor_all_with(TensorKind::Ordinary, children)
    }

    /// Constructs a graded tensor, preserving grouping.
    pub fn tensor_all_g(children: impl IntoIterator<Item = Self>) -> Result<Self, SpaceError> {
        Self::tensor_all_with(TensorKind::Graded, children)
    }

    /// Constructs a tensor with an explicit composition rule.
    pub fn tensor_all_with(
        kind: TensorKind,
        children: impl IntoIterator<Item = Self>,
    ) -> Result<Self, SpaceError> {
        let children: Operands<Self> = children.into_iter().collect();
        let (by_index, effective_kind) = Self::tensor_metadata(kind, children.as_slice())?;
        Ok(Self::new_tensor(kind, children, by_index, effective_kind))
    }

    fn tensor_metadata(
        kind: TensorKind,
        children: &[Self],
    ) -> Result<(FactorIndex<I>, Option<TensorKind>), SpaceError> {
        let mut nonempty = children.iter().filter(|child| child.factor_count() != 0);
        match (nonempty.next(), nonempty.next()) {
            (None, _) => Ok((FactorIndex::Empty, None)),
            (Some(child), None) => Ok((FactorIndex::reuse(child), child.effective_kind)),
            (Some(_), Some(_)) => {
                if children
                    .iter()
                    .any(|c| c.effective_kind.is_some_and(|k| k != kind))
                {
                    return Err(SpaceError::MixedTensorKinds);
                }
                Ok((FactorIndex::merge(children)?, Some(kind)))
            }
        }
    }

    pub(crate) fn factor_position(&self, index: &I) -> Option<usize> {
        match self.node() {
            Node::Factor(leaf) => (leaf == index).then_some(0),
            Node::Tensor { by_index, .. } => by_index.position(index),
        }
    }

    /// Constructs a binary tensor with this space's factors on the left.
    pub fn tensor(&self, rhs: &Self) -> Result<Self, SpaceError> {
        self.tensor_with(TensorKind::Ordinary, rhs)
    }
    /// Constructs a binary graded tensor.
    pub fn tensor_g(&self, rhs: &Self) -> Result<Self, SpaceError> {
        self.tensor_with(TensorKind::Graded, rhs)
    }
    /// Constructs a binary tensor with an explicit rule.
    pub fn tensor_with(&self, kind: TensorKind, rhs: &Self) -> Result<Self, SpaceError> {
        Self::tensor_all_with(kind, [self.clone(), rhs.clone()])
    }

    /// Constructs an ordinary tensor and promotes the children's grouping by one level.
    ///
    /// The result and errors match `self.tensor(rhs)?.flatten()`.
    pub fn tensor_flatten(&self, rhs: &Self) -> Result<Self, SpaceError> {
        self.tensor_flatten_with(TensorKind::Ordinary, rhs)
    }

    /// Constructs a graded tensor and promotes the children's grouping by one level.
    pub fn tensor_flatten_g(&self, rhs: &Self) -> Result<Self, SpaceError> {
        self.tensor_flatten_with(TensorKind::Graded, rhs)
    }

    /// Constructs a tensor with an explicit rule, promoting each tensor operand's
    /// immediate children into the result.
    ///
    /// The result and errors match `self.tensor_with(kind, rhs)?.flatten()`.
    pub fn tensor_flatten_with(&self, kind: TensorKind, rhs: &Self) -> Result<Self, SpaceError> {
        let children = [self.clone(), rhs.clone()];
        let (by_index, effective_kind) = Self::tensor_metadata(kind, &children)?;
        Ok(Self::new_flattened_tensor(
            kind,
            &children,
            by_index,
            effective_kind,
        ))
    }
}

impl<I: Eq> HilbertSpace<I> {
    /// Tests equality of ordered factors, independently of grouping and sharing.
    pub fn is_compatible_with(&self, rhs: &Self) -> bool {
        self.shares_node_with(rhs)
            || (self.effective_kind == rhs.effective_kind && self.indices().eq(rhs.indices()))
    }

    /// Tests whether this space's factors form an ordered subsequence of `ambient`.
    pub fn is_subsystem_of(&self, ambient: &Self) -> bool {
        if self.effective_kind.is_some() && self.effective_kind != ambient.effective_kind {
            return false;
        }
        let mut ambient = ambient.indices();
        self.indices()
            .all(|index| ambient.any(|other| index == other))
    }

    /// Tests whether this space contains an index, without inferring its structure.
    pub fn contains_index(&self, index: &I) -> bool {
        self.indices().any(|i| i == index)
    }
}

/// A borrowing, depth-first iterator over a space's leaf indices.
pub struct Indices<'a, I> {
    leaves: Leaves<'a, I>,
}

impl<'a, I> Iterator for Indices<'a, I> {
    type Item = &'a I;
    fn next(&mut self) -> Option<Self::Item> {
        self.leaves.next().and_then(HilbertSpace::factor_index)
    }
}
impl<I> FusedIterator for Indices<'_, I> {}

/// A borrowing, depth-first iterator over the actual leaf space handles.
pub struct Leaves<'a, I> {
    pending: Vec<&'a HilbertSpace<I>>,
}

impl<'a, I> Iterator for Leaves<'a, I> {
    type Item = &'a HilbertSpace<I>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(space) = self.pending.pop() {
            match space.node() {
                Node::Factor(_) => return Some(space),
                Node::Tensor { children, .. } => self.pending.extend(children.iter().rev()),
            }
        }
        None
    }
}
impl<I> FusedIterator for Leaves<'_, I> {}

impl<I> Drop for HilbertSpace<I> {
    fn drop(&mut self) {
        let mut first = self.root.take();
        let mut pending = Vec::new();
        while let Some(root) = first.take().or_else(|| pending.pop()) {
            if let Some(Node::Tensor { children, .. }) = Arc::into_inner(root) {
                for mut child in children {
                    if let Some(root) = child.root.take() {
                        pending.push(root);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merged_indices_map_interleaved_ids_to_tensor_order() {
        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
        struct Id(u8);
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<HilbertSpace<Id>>();

        let a = HilbertSpace::factor(Id(30));
        let b = HilbertSpace::factor(Id(10));
        let left = a.tensor_g(&b).unwrap();
        let right = HilbertSpace::tensor_all_g([
            HilbertSpace::factor(Id(20)),
            HilbertSpace::factor(Id(40)),
        ])
        .unwrap();
        let unit = HilbertSpace::unit();
        let space = HilbertSpace::tensor_all_g([
            left,
            unit.clone(),
            right.clone(),
            HilbertSpace::factor(Id(25)),
        ])
        .unwrap();
        let alias = space.clone();
        let wrapped = HilbertSpace::tensor_all([unit, space.clone()]).unwrap();
        for candidate in [&space, &alias, &wrapped] {
            assert_eq!(candidate.effective_kind(), Some(TensorKind::Graded));
            assert_eq!(candidate.factor_count(), 5);
            for (position, id) in [30, 10, 20, 40, 25].into_iter().enumerate() {
                assert_eq!(candidate.factor_position(&Id(id)), Some(position));
            }
            assert_eq!(candidate.factor_position(&Id(99)), None);
            assert!(candidate.leaves().nth(1).unwrap().shares_node_with(&b));
        }
        let Node::Tensor {
            by_index: FactorIndex::Many(original),
            ..
        } = space.node()
        else {
            panic!("multi-factor index");
        };
        let Node::Tensor {
            by_index: FactorIndex::Many(reused),
            ..
        } = wrapped.node()
        else {
            panic!("inherited index");
        };
        assert!(Arc::ptr_eq(original, reused));
        let overlap = right.tensor_g(&HilbertSpace::factor(Id(20)));
        assert_eq!(overlap.unwrap_err(), SpaceError::DuplicateIndex);
        let wrong_kind = right.tensor(&HilbertSpace::factor(Id(20)));
        assert_eq!(wrong_kind.unwrap_err(), SpaceError::MixedTensorKinds);
    }

    #[test]
    fn units_and_singleton_wrappers_have_no_position_offset() {
        let empty = HilbertSpace::tensor_all_g([HilbertSpace::<u8>::unit()]).unwrap();
        assert_eq!(empty.factor_count(), 0);
        assert_eq!(empty.factor_position(&4), None);
        let factor = HilbertSpace::factor(4);
        let wrapped = empty.tensor(&factor).unwrap();
        assert_eq!(wrapped.factor_count(), 1);
        assert_eq!(wrapped.factor_position(&4), Some(0));
        assert_eq!(wrapped.factor_position(&5), None);
        let repeated = wrapped.tensor(&HilbertSpace::factor(4));
        assert_eq!(repeated.unwrap_err(), SpaceError::DuplicateIndex);
    }

    #[test]
    fn flatten_promotes_one_level_and_reuses_the_index() {
        let a = HilbertSpace::factor(30);
        let b = HilbertSpace::factor(10);
        let c = HilbertSpace::factor(20);
        let d = HilbertSpace::factor(40);
        let ab = a.tensor(&b).unwrap();
        let abc = ab.tensor(&c).unwrap();
        let unit = HilbertSpace::unit();
        let space = HilbertSpace::tensor_all([abc.clone(), unit.clone(), d.clone()]).unwrap();
        let flattened = space.flatten();
        let expected = [&ab, &c, &d];
        assert_eq!(flattened.children().len(), expected.len());
        for (child, original) in flattened.children().zip(expected) {
            assert!(child.shares_node_with(original));
        }
        assert!(space.children().next().unwrap().shares_node_with(&abc));
        assert!(flattened.is_compatible_with(&space));
        for (position, index) in [30, 10, 20, 40].into_iter().enumerate() {
            assert_eq!(flattened.factor_position(&index), Some(position));
        }
        let FactorIndex::Many(original) = FactorIndex::reuse(&space) else {
            panic!("multi-factor index");
        };
        let FactorIndex::Many(reused) = FactorIndex::reuse(&flattened) else {
            panic!("flattened index");
        };
        assert!(Arc::ptr_eq(&original, &reused));
        assert_eq!(flattened.flatten().children().len(), 4);
    }

    #[test]
    fn flatten_preserves_effective_kind_through_units_and_wrappers() {
        let unit = HilbertSpace::unit();
        let a = HilbertSpace::factor(3);
        let b = HilbertSpace::factor(1);
        for (inner, outer) in [
            (TensorKind::Graded, TensorKind::Ordinary),
            (TensorKind::Ordinary, TensorKind::Graded),
        ] {
            let pair = a.tensor_with(inner, &b).unwrap();
            let wrapper =
                HilbertSpace::tensor_all_with(outer, [unit.clone(), pair.clone()]).unwrap();
            let nested = HilbertSpace::tensor_all_with(outer, [wrapper]).unwrap();
            let flattened_once = nested.flatten();
            assert_eq!(flattened_once.kind(), Some(outer));
            assert_eq!(flattened_once.children().len(), 2);
            assert!(
                flattened_once
                    .children()
                    .next()
                    .unwrap()
                    .shares_node_with(&unit)
            );

            let flattened_pair = flattened_once.flatten();
            assert_eq!(flattened_pair.kind(), Some(inner));
            for space in [&flattened_once, &flattened_pair] {
                assert_eq!(space.effective_kind(), Some(inner));
                assert!(space.is_compatible_with(&pair));
                let parent =
                    HilbertSpace::tensor_all_with(inner, [space.clone(), HilbertSpace::factor(7)])
                        .unwrap();
                assert!(parent.indices().copied().eq([3, 1, 7]));
            }
            assert!(flattened_pair.flatten().shares_node_with(&flattened_pair));
        }

        let singleton = HilbertSpace::tensor_all_g([a.tensor(&unit).unwrap()])
            .unwrap()
            .flatten();
        assert_eq!(singleton.kind(), Some(TensorKind::Graded));
        assert_eq!(singleton.effective_kind(), None);
        assert_eq!(singleton.factor_position(&3), Some(0));
        assert!(singleton.children().next().unwrap().shares_node_with(&a));

        let empty = HilbertSpace::tensor_all_g([unit]).unwrap().flatten();
        assert_eq!(empty.kind(), Some(TensorKind::Graded));
        assert_eq!(empty.effective_kind(), None);
        assert_eq!(empty.children().len(), 0);

        struct Unordered;
        let leaf = HilbertSpace::factor(Unordered);
        assert!(leaf.flatten().shares_node_with(&leaf));
    }

    #[test]
    fn tensor_flatten_matches_tensor_then_flatten() {
        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
        struct Id(u8);

        let a = HilbertSpace::factor(Id(30));
        let b = HilbertSpace::factor(Id(10));
        let c = HilbertSpace::factor(Id(20));
        let ordinary = a.tensor(&b).unwrap();
        let graded = b.tensor_g(&c).unwrap();
        let unit = HilbertSpace::unit();
        let graded_unit = HilbertSpace::tensor_all_g([]).unwrap();
        let wrapped = unit.tensor(&graded).unwrap();
        let nested = HilbertSpace::tensor_all_g([wrapped.clone()]).unwrap();
        let spaces = [unit, graded_unit, a, c, ordinary, graded, wrapped, nested];
        for left in &spaces {
            for right in &spaces {
                for kind in [TensorKind::Ordinary, TensorKind::Graded] {
                    let expected = left.tensor_with(kind, right).map(|tensor| tensor.flatten());
                    let actual = match kind {
                        TensorKind::Ordinary => left.tensor_flatten(right),
                        TensorKind::Graded => left.tensor_flatten_g(right),
                    };
                    match (expected, actual) {
                        (Ok(expected), Ok(actual)) => {
                            assert_eq!(actual.kind(), expected.kind());
                            assert_eq!(actual.effective_kind(), expected.effective_kind());
                            assert_eq!(actual.children().len(), expected.children().len());
                            for (child, original) in actual.children().zip(expected.children()) {
                                assert!(child.shares_node_with(original));
                            }
                            assert!(actual.is_compatible_with(&expected));
                            for (position, index) in expected.indices().enumerate() {
                                assert_eq!(actual.factor_position(index), Some(position));
                            }
                        }
                        (Err(expected), Err(actual)) => assert_eq!(actual, expected),
                        (expected, actual) => {
                            panic!("composition mismatch: {expected:?}, {actual:?}")
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn tensor_flatten_checks_kind_before_duplicates_in_original_operands() {
        let left = HilbertSpace::factor(30)
            .tensor_g(&HilbertSpace::factor(10))
            .unwrap();
        let repeated = HilbertSpace::factor(10);
        assert_eq!(
            left.tensor_flatten(&repeated).unwrap_err(),
            SpaceError::MixedTensorKinds,
        );
        assert_eq!(
            left.tensor_flatten_g(&repeated).unwrap_err(),
            SpaceError::DuplicateIndex,
        );
    }
}
