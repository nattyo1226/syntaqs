//! Collect an ordinary commutator of two single sparse blocks.

use super::{Block, EvaluatedOperator, Factored, OperatorTerm, Sum, overlaps};
use crate::{
    Adjoint, HilbertSpace, HomogeneousParity, Identity, Product, expression::composite::OpExpr,
};
use alloc::{collections::BTreeMap, vec, vec::Vec};
use core::ops::Neg;
use num_complex::Complex;
use num_traits::{Num, NumCast, Zero};

impl<O, R> EvaluatedOperator<O, Complex<R>>
where
    O: PartialEq
        + Identity
        + HomogeneousParity
        + Product<Complex<R>, Term = O>
        + Adjoint<Complex<R>, Term = O>,
    R: Copy + Num + NumCast + Neg<Output = R>,
{
    pub(crate) fn commutator<I: Ord>(
        left: &OpExpr<I, O, Complex<R>>,
        right: &OpExpr<I, O, Complex<R>>,
        space: &HilbertSpace<I>,
    ) -> Option<Self> {
        let mut result = Self::empty(space);
        let left = result.evaluate(left, space);
        let [a] = left.blocks.as_slice() else {
            return None;
        };
        if a.terms.len() < 2 {
            return None;
        }
        let right = result.evaluate(right, space);
        let [b] = right.blocks.as_slice() else {
            return None;
        };
        let scale = left.coefficient * right.coefficient;
        if b.terms.len() < 2
            || (a.support.len() == 1 && b.support.len() == 1)
            || !overlaps(&a.support, &b.support)
            || scale.is_zero()
            || result.primitives.iter().any(|p| !p.reflexive)
            || !disjoint_products_are_finite(&left, a, &right, b)
        {
            return None;
        }
        let mut ab = Factored::from_block(result.multiply_commutator_pairs(a, b));
        let mut ba = Factored::from_block(result.multiply_commutator_pairs(b, a));
        ab.coefficient = scale * ab.coefficient;
        ba.coefficient = -(scale * ba.coefficient);
        result.product = result.add(vec![ab, ba]);
        Some(result)
    }

    fn multiply_commutator_pairs(
        &mut self,
        left: &Block<Complex<R>>,
        right: &Block<Complex<R>>,
    ) -> Block<Complex<R>> {
        // The index keeps term positions so retained products can accumulate
        // in the existing row order in each direction.
        let mut terms_at = BTreeMap::<usize, Vec<usize>>::new();
        let mut odd_terms = Vec::new();
        for (index, term) in right.terms.iter().enumerate() {
            for &(position, _) in &term.factors {
                terms_at.entry(position).or_default().push(index);
            }
            if right.odd.unwrap_or_else(|| self.term_is_odd(term)) {
                odd_terms.push(index);
            }
        }

        let mut sum = Sum::new();
        let mut candidates = Vec::new();
        for a in &left.terms {
            for &(position, _) in &a.factors {
                if let Some(indices) = terms_at.get(&position) {
                    candidates.extend_from_slice(indices);
                }
            }
            // The bracket is ordinary even in a graded space. Disjoint odd
            // terms anticommute and contribute through both products.
            if !odd_terms.is_empty() && left.odd.unwrap_or_else(|| self.term_is_odd(a)) {
                candidates.extend_from_slice(&odd_terms);
            }
            candidates.sort_unstable();
            candidates.dedup();
            for index in candidates.drain(..) {
                for term in self.multiply_terms(a, &right.terms[index]) {
                    sum.add(term, &self.primitives);
                }
            }
        }
        sum.finish(&self.primitives)
    }

    fn term_is_odd(&self, term: &OperatorTerm<Complex<R>>) -> bool {
        term.factors
            .iter()
            .fold(false, |odd, &(_, id)| odd ^ self.primitives[id].odd)
    }
}

// Bound the sum of absolute components of the disjoint products in both
// directions. Keep the symbolic path if that sum may overflow in R.
fn disjoint_products_are_finite<R>(
    lhs: &Factored<Complex<R>>,
    a: &Block<Complex<R>>,
    rhs: &Factored<Complex<R>>,
    b: &Block<Complex<R>>,
) -> bool
where
    R: NumCast,
{
    let weight = |c: &Complex<R>| Some(c.re.to_f64()?.abs() + c.im.to_f64()?.abs());
    let bound = |product: &Factored<Complex<R>>, block: &Block<Complex<R>>| {
        let terms: f64 = block
            .terms
            .iter()
            .map(|term| weight(&term.coefficient))
            .sum::<Option<_>>()?;
        Some((1.0 + weight(&product.coefficient)?) * (1.0 + terms))
    };
    bound(lhs, a)
        .zip(bound(rhs, b))
        .and_then(|(a, b)| R::from(2.0 * a * b))
        .and_then(|value| value.to_f64())
        .is_some_and(f64::is_finite)
}
