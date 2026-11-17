use super::args::Ambient;
use crate::{
    Export, ExportError, Exported, MatrixArgs, MatrixMetadata,
    backend::{MatrixBuilder, MatrixStorage},
    coordinates::into_flat_entries,
};
use num_traits::{One, Zero};
use std::{marker::PhantomData, ops::Mul};
use syntaqs_core::{
    algebra::Act, algebra::Adjoint, basis::Basis, basis::BasisError, coefficient::Conjugate,
    expression::composite::Operator, expression::error::ExpressionError,
    expression::local::LocalOperator, expression::local::LocalState, space::Dimension,
    space::HilbertSpace,
};

impl<'b, S, O, C, B, D> Export<Exported<D, MatrixMetadata<'b, S, B>>, MatrixArgs<'b, S, B>>
    for &LocalOperator<O, C>
where
    C: One + Clone + Zero + Mul<Output = C> + Conjugate<Output = C>,
    S: PartialEq + Dimension + Act<C, Operator = O, Term = S>,
    O: Adjoint<C, Term = O>,
    B: Basis<S, C>,
    D: MatrixStorage<C>,
{
    type Error = ExportError;
    fn export(
        self,
        args: MatrixArgs<'b, S, B>,
    ) -> Result<Exported<D, MatrixMetadata<'b, S, B>>, ExportError> {
        let dimension = S::dim()?.get();
        let mut builder = D::builder(dimension)?;
        let mut count = 0;
        for primitive in args.basis.elements() {
            if count >= dimension || args.basis.index(&primitive) != Some(count) {
                return Err(BasisError::InvalidEnumeration.into());
            }
            let column = LocalState::primitive(primitive)
                .act(self)
                .coordinates(args.basis)?;
            builder.column(count, into_flat_entries(column))?;
            count += 1;
        }
        if count != dimension {
            return Err(BasisError::InvalidEnumeration.into());
        }
        Ok(Exported {
            data: builder.finish()?,
            metadata: MatrixMetadata {
                basis: args.basis,
                space: (),
                dimension,
                factor_count: 1,
                dim_local: Some(dimension),
                state: PhantomData,
            },
        })
    }
}

impl<'b, I: Eq, S, O, C, B, A, D>
    Export<Exported<D, MatrixMetadata<'b, S, B, HilbertSpace<I>>>, MatrixArgs<'b, S, B, A>>
    for &Operator<I, O, C>
where
    C: PartialEq + Clone + Zero + One + Conjugate<Output = C>,
    S: PartialEq + Dimension + Act<C, Operator = O, Term = S>,
    O: Adjoint<C, Term = O>
        + syntaqs_core::algebra::HomogeneousParity
        + syntaqs_core::algebra::ParityOperator,
    B: Basis<S, C>,
    A: Ambient<I>,
    D: MatrixStorage<C>,
{
    type Error = ExportError;
    fn export(
        self,
        args: MatrixArgs<'b, S, B, A>,
    ) -> Result<Exported<D, MatrixMetadata<'b, S, B, HilbertSpace<I>>>, ExportError> {
        let space = args
            .ambient
            .explicit()
            .or_else(|| self.space())
            .ok_or(ExpressionError::UnboundOperator)?;
        // Validate every reference even for a zero operator, before allocating output.
        let bound = self.bind(space)?;
        let dimension = space.dim::<S>()?;
        let factor_count = space.indices().count();
        let dim_local = if factor_count == 0 {
            None
        } else {
            Some(S::dim()?.get())
        };
        let mut builder = D::builder(dimension)?;
        for (col, input) in space.basis_states::<S, O, C, _>(args.basis)?.enumerate() {
            let column = input?.act(&bound)?.coordinates(args.basis)?;
            builder.column(col, into_flat_entries(column))?;
        }
        Ok(Exported {
            data: builder.finish()?,
            metadata: MatrixMetadata {
                basis: args.basis,
                space: space.clone(),
                dimension,
                factor_count,
                dim_local,
                state: PhantomData,
            },
        })
    }
}
