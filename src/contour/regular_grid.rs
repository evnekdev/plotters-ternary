//! Plotters-ternary compatibility facade over `ternary-contours` grid values.

pub(crate) use ternary_contours::GridTriangle;
pub use ternary_contours::{GridVertexId, LatticeCoordinate};

use crate::coord::TernaryPoint;
use ternary_contours::{FieldError, RegularTernaryScalarField as CoreField};

use super::ContourError;

/// A scalar field sampled on the regular ternary lattice `i+j+k=n`.
///
/// This compatibility wrapper preserves Plotters-ternary's semantic
/// [`TernaryPoint`] accessors while delegating all indexing, validation, and
/// lattice topology to [`ternary_contours::RegularTernaryScalarField`].
#[derive(Clone, Debug, PartialEq)]
pub struct RegularTernaryScalarField {
    core: CoreField,
}

impl RegularTernaryScalarField {
    pub fn new(subdivisions: usize, values: Vec<f64>) -> Result<Self, ContourError> {
        Ok(Self {
            core: CoreField::new(subdivisions, values)?,
        })
    }
    pub const fn subdivisions(&self) -> usize {
        self.core.subdivisions()
    }
    pub fn values(&self) -> &[f64] {
        self.core.values()
    }
    pub const fn vertex_count(&self) -> usize {
        self.core.vertex_count()
    }
    pub fn triangle_count(&self) -> Result<usize, ContourError> {
        Ok(self.core.triangle_count()?)
    }
    pub fn edge_count(&self) -> Result<usize, ContourError> {
        Ok(self.core.edge_count()?)
    }
    pub fn value(&self, id: GridVertexId) -> Result<f64, ContourError> {
        Ok(self.core.value(id)?)
    }
    pub fn vertex_id(&self, coordinate: LatticeCoordinate) -> Result<GridVertexId, ContourError> {
        Ok(self.core.vertex_id(coordinate)?)
    }
    pub fn lattice_coordinate(&self, id: GridVertexId) -> Result<LatticeCoordinate, ContourError> {
        Ok(self.core.lattice_coordinate(id)?)
    }
    pub fn composition(&self, id: GridVertexId) -> Result<TernaryPoint, ContourError> {
        let [a, b, c] = self.core.composition(id)?;
        Ok(TernaryPoint::new(a, b, c))
    }
    pub fn index_of(&self, i: usize, j: usize, k: usize) -> Result<usize, ContourError> {
        Ok(self.core.index_of(i, j, k)?)
    }
    pub fn coordinate_of(&self, index: usize) -> Result<LatticeCoordinate, ContourError> {
        Ok(self.core.coordinate_of(index)?)
    }
    pub fn composition_at(&self, index: usize) -> Result<TernaryPoint, ContourError> {
        self.composition(GridVertexId(index))
    }
    pub(crate) fn triangles(&self) -> Result<Vec<GridTriangle>, ContourError> {
        Ok(self.core.elementary_triangles()?)
    }
    #[cfg(feature = "cubic-alpha")]
    pub(crate) fn core(&self) -> &CoreField {
        &self.core
    }
}

impl From<FieldError> for ContourError {
    fn from(error: FieldError) -> Self {
        match error {
            FieldError::ZeroSubdivisions => Self::ZeroSubdivisions,
            FieldError::AllocationOverflow => Self::AllocationOverflow,
            FieldError::IncorrectValueCount { expected, actual } => {
                Self::IncorrectValueCount { expected, actual }
            }
            FieldError::NonFiniteValue { index, value } => Self::NonFiniteValue { index, value },
            FieldError::InvalidLatticeCoordinate {
                i,
                j,
                k,
                subdivisions,
            } => Self::InvalidLatticeCoordinate {
                i,
                j,
                k,
                subdivisions,
            },
            FieldError::InvalidVertexIndex {
                index,
                vertex_count,
            } => Self::InvalidVertexIndex {
                index,
                vertex_count,
            },
            FieldError::InsufficientStencil { samples } => Self::InsufficientStencil { samples },
            FieldError::CubicFeatureUnavailable => Self::CubicFeatureUnavailable,
            FieldError::Interpolation(error) => Self::Interpolation(error),
        }
    }
}
