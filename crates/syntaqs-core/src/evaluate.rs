//! Evaluation and scalar contractions using primitive rules.

mod collect;
mod composite;
pub(crate) mod inner_product;
mod local;
mod trace;

mod state_terms;

#[cfg(test)]
mod tests;
