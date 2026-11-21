//! Basis expansion and sparse coordinates share index aggregation.

use alloc::{collections::BTreeMap, collections::btree_map::Entry, vec::Vec};
use num_traits::Zero;

use super::enumeration::basis_element;
use crate::{Basis, BasisError, expression::parts::Operands};

mod composite;
mod local;
mod operator;

type Expansion<C> = BTreeMap<Vec<usize>, C>;

// Remove exact zeros before intermediate maps feed later tensor products.
fn accumulate<K: Ord, C: Zero>(map: &mut BTreeMap<K, C>, key: K, coefficient: C) {
    match map.entry(key) {
        Entry::Vacant(entry) => {
            if !coefficient.is_zero() {
                entry.insert(coefficient);
            }
        }
        Entry::Occupied(mut entry) => {
            let existing = entry.get_mut();
            *existing = core::mem::replace(existing, C::zero()) + coefficient;
            if existing.is_zero() {
                entry.remove();
            }
        }
    }
}

fn reconstruct<P, C, B: Basis<P, C>, L, E: Clone>(
    terms: Expansion<C>,
    basis: &B,
    dimension: Option<usize>,
    make_local: impl Fn(P) -> L,
    place: impl Fn(usize, &L) -> E,
    make_term: impl Fn(Operands<E>, C) -> E,
) -> Result<Operands<E>, BasisError> {
    let mut locals = BTreeMap::new();
    let mut placed: Vec<BTreeMap<usize, E>> = Vec::new();
    terms
        .into_iter()
        .map(|(digits, coefficient)| {
            if placed.is_empty() {
                placed.resize_with(digits.len(), BTreeMap::new);
            }
            let factors = digits
                .into_iter()
                .enumerate()
                .map(|(position, digit)| {
                    let slot = match placed[position].entry(digit) {
                        Entry::Occupied(entry) => return Ok(entry.get().clone()),
                        Entry::Vacant(entry) => entry,
                    };
                    let local = match locals.entry(digit) {
                        Entry::Occupied(entry) => entry.into_mut(),
                        Entry::Vacant(entry) => entry.insert(make_local(basis_element(
                            basis,
                            digit,
                            dimension.expect("local factor"),
                        )?)),
                    };
                    Ok(slot.insert(place(position, local)).clone())
                })
                .collect::<Result<Operands<_>, BasisError>>()?;
            Ok(make_term(factors, coefficient))
        })
        .collect()
}
