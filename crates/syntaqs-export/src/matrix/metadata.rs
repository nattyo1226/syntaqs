use std::marker::PhantomData;

/// Dense matrix memory layout, separate from tensor-factor basis order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    /// Element `(row, col)` is at `row * ncols + col`.
    RowMajor,
}

/// Shape, borrowed state basis and selected action space of a matrix.
///
/// Input and output use the same state basis and ordered factor space.
pub struct MatrixMetadata<'b, S, B, H = ()> {
    pub(crate) basis: &'b B,
    pub(crate) space: H,
    pub(crate) dimension: usize,
    pub(crate) factor_count: usize,
    pub(crate) dim_local: Option<usize>,
    pub(crate) state: PhantomData<fn() -> S>,
}

impl<'b, S, B, H> MatrixMetadata<'b, S, B, H> {
    /// Borrows the common local input and output state basis.
    pub fn basis(&self) -> &'b B {
        self.basis
    }
    /// Borrows the selected action space, or `()` for a local operator.
    pub fn space(&self) -> &H {
        &self.space
    }
    /// Borrows the selected input space.
    pub fn domain(&self) -> &H {
        &self.space
    }
    /// Borrows the selected output space.
    pub fn codomain(&self) -> &H {
        &self.space
    }
    /// Returns the number of rows and columns.
    pub fn shape(&self) -> [usize; 2] {
        [self.dimension; 2]
    }
    /// Returns the dense export's memory layout.
    pub fn layout(&self) -> Layout {
        Layout::RowMajor
    }
    /// Returns the number of basic factors in tensor order.
    pub fn factor_count(&self) -> usize {
        self.factor_count
    }
    /// Returns the local state-basis size, or None on the unit space.
    pub fn dim_local(&self) -> Option<usize> {
        self.dim_local
    }
}
