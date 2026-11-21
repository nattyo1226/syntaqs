//! Spectral sampling, validation and reconstruction for decomposition.

use super::*;

pub(super) fn component<'a, E: FamilyExpression>(
    source: &'a OperatorFamily<E>,
    coordinates: Array2<E::Real>,
    options: &NumericalOptions<E::Real>,
) -> Result<LieComponent<'a, E>, LieError<E::Real>> {
    finite(coordinates.iter().copied())?;
    let operators = coordinates
        .columns()
        .into_iter()
        .map(|v| Ok(source.reconstruct(v)?.collected_in(source.context())?))
        .collect::<Result<Vec<_>, LieError<E::Real>>>()?;
    Ok(LieComponent {
        source,
        family: OperatorFamily::new_in(operators, source.context())?,
        coordinates,
        options: *options,
    })
}

fn check_reconstruction<E: FamilyExpression>(
    component: &LieComponent<'_, E>,
    index: usize,
) -> Result<(), LieError<E::Real>>
where
    NumericalOptions<E::Real>: Default,
{
    let report = component.family.orthonormality(&component.options)?;
    for (index, check) in report.validation().hermiticity().iter().enumerate() {
        if !check.passed() {
            return Err(LieError::NonHermitianElement {
                index,
                check: *check,
            });
        }
    }
    let independence = report.validation().independence();
    if !independence.is_independent() {
        return Err(NumericalError::SingularBasis {
            rank: independence.rank,
            dimension: independence.dimension,
        }
        .into());
    }
    if !report.check().passed() {
        return Err(LieError::ConstraintResidual {
            operation: "reconstructed component orthonormality",
            index,
            check: *report.check(),
        });
    }
    Ok(())
}

// SplitMix64 supplies a small deterministic sampling stream without introducing
// another production dependency. It is not a cryptographic generator.
pub(super) struct Sampler(pub(super) u64);
impl Sampler {
    pub(super) fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut value = self.0;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
        value ^= value >> 31;
        ((value >> 11) as f64) * (2.0 / 9007199254740992.0) - 1.0
    }
    pub(super) fn sample<R: Float + Scalar<Real = R> + Lapack>(
        &mut self,
        n: usize,
        basis: &[Array2<R>],
    ) -> Array2<R> {
        let mut result = Array2::zeros((n, n));
        for matrix in basis {
            // Conversion to either supported precision is finite.
            let coefficient = R::from_f64(self.next()).expect("finite f32/f64 sample");
            result = result + matrix.mapv(|x| x * coefficient);
        }
        result
    }
}

pub(super) fn groups<R: Float>(eigenvalues: &Array1<R>, threshold: R) -> Vec<Range<usize>> {
    if eigenvalues.is_empty() {
        return Vec::new();
    }
    let mut start = 0;
    let mut groups = Vec::new();
    for i in 1..eigenvalues.len() {
        if eigenvalues[i] - eigenvalues[i - 1] > threshold {
            groups.push(start..i);
            start = i;
        }
    }
    groups.push(start..eigenvalues.len());
    groups
}

fn restricted_adjoints<R>(
    ambient: &[Array2<R>],
    axes: &Array2<R>,
) -> Result<Vec<Array2<R>>, NumericalError>
where
    R: Float + Scalar<Real = R> + Lapack,
{
    let n = axes.nrows();
    let d = axes.ncols();
    let mut result = Vec::new();
    for j in 0..d {
        let combined = Array2::from_shape_fn((n, n), |(a, b)| {
            (0..n).fold(R::zero(), |sum, i| sum + axes[(i, j)] * ambient[i][(a, b)])
        });
        let matrix = axes.t().dot(&combined).dot(axes);
        finite(matrix.iter().copied())?;
        result.push(matrix);
    }
    Ok(result)
}

pub(super) fn validate_candidate<R>(
    ambient: &[Array2<R>],
    center: &Array2<R>,
    ideals: &[Array2<R>],
    options: &NumericalOptions<R>,
) -> Result<DecompositionValidation<R>, LieError<R>>
where
    R: Float + Scalar<Real = R> + Lapack,
    NumericalOptions<R>: Default,
{
    let n = center.nrows();
    let mut combined = Array2::zeros((n, n));
    let mut column = 0;
    for axes in std::iter::once(center).chain(ideals) {
        for vector in axes.columns() {
            combined.column_mut(column).assign(&vector);
            column += 1;
        }
    }
    let identity = Array2::eye(n);
    let identity_norm = vector_norm(identity.iter().copied());
    let orthonormality = options.residual.check(
        vector_norm((combined.t().dot(&combined) - &identity).iter().copied()),
        identity_norm,
    )?;
    let span = options.residual.check(
        vector_norm((combined.dot(&combined.t()) - &identity).iter().copied()),
        identity_norm,
    )?;
    let ambient_norm = vector_norm(ambient.iter().flat_map(|a| a.iter().copied()));
    let center_defect = vector_norm(ambient.iter().flat_map(|a| a.dot(center).into_iter()));
    let centrality = options.residual.check(
        center_defect,
        ambient_norm * vector_norm(center.iter().copied()),
    )?;
    let mut ideal_invariance = Vec::new();
    let mut cross_commutation = Vec::new();
    let mut abelianity = Vec::new();
    let mut ideal_centers = Vec::new();
    let mut simple_commutants = Vec::new();
    for axes in ideals {
        let projector = axes.dot(&axes.t());
        let defect = vector_norm(ambient.iter().flat_map(|a| {
            let action = a.dot(axes);
            (&action - projector.dot(&action)).into_iter()
        }));
        ideal_invariance.push(
            options
                .residual
                .check(defect, ambient_norm * vector_norm(axes.iter().copied()))?,
        );
        let restricted = restricted_adjoints(ambient, axes)?;
        let restricted_norm = vector_norm(restricted.iter().flat_map(|a| a.iter().copied()));
        abelianity.push(options.residual.check(restricted_norm, ambient_norm)?);
        let kernel = adjoint_kernel(&restricted, axes.ncols());
        let analysis = rank_nullspace(&kernel, options)?;
        ideal_centers.push(diagnostics(
            &kernel,
            &analysis,
            options,
            "restricted ideal center",
        )?);
        simple_commutants
            .push(symmetric_commutant(&restricted, axes.ncols(), options)?.diagnostics);
    }
    for (i, left) in ideals.iter().enumerate() {
        for right in &ideals[i + 1..] {
            let mut defect = R::zero();
            let mut scale = R::zero();
            for vector in left.columns() {
                let action = Array2::from_shape_fn((n, n), |(a, b)| {
                    (0..n).fold(R::zero(), |sum, k| sum + vector[k] * ambient[k][(a, b)])
                });
                defect = defect.hypot(vector_norm(action.dot(right).iter().copied()));
                scale = scale.hypot(
                    vector_norm(action.iter().copied()) * vector_norm(right.iter().copied()),
                );
            }
            cross_commutation.push(options.residual.check(defect, scale)?);
        }
    }
    Ok(DecompositionValidation {
        orthonormality,
        span,
        centrality,
        ideal_invariance,
        cross_commutation,
        abelianity,
        ideal_centers,
        simple_commutants,
    })
}

pub(super) fn decompose_with_sampler<'a, E, F>(
    adjoint: &AdjointRepresentation<'a, E>,
    settings: &DecompositionOptions,
    mut sample: F,
) -> Result<LieDecomposition<'a, E>, LieError<E::Real>>
where
    E: FamilyExpression,
    NumericalOptions<E::Real>: Default,
    F: FnMut(usize, &[Array2<E::Real>]) -> Array2<E::Real>,
{
    settings.validate()?;
    let options = adjoint.options();
    options.validate()?;
    let split = adjoint.center()?;
    let ambient = &split.orthonormal.adjoints;
    let semisimple = &split.complement_axes;
    let dimension = semisimple.ncols();
    let mut attempts = Vec::new();
    let mut accepted = None;
    let mut commutant_diagnostics = None;
    if dimension == 0 {
        let validation = validate_candidate(ambient, &split.center_axes, &[], options)?;
        for (operation, check) in [
            (
                "abelian decomposition orthonormality",
                validation.orthonormality,
            ),
            ("abelian decomposition span", validation.span),
            ("abelian decomposition centrality", validation.centrality),
        ] {
            if !check.passed() {
                return Err(LieError::ConstraintResidual {
                    operation,
                    index: 0,
                    check,
                });
            }
        }
        accepted = Some((Vec::new(), validation));
    } else {
        let restricted = restricted_adjoints(ambient, semisimple)?;
        let commutant = symmetric_commutant(&restricted, dimension, options)?;
        let expected = commutant.matrices.len();
        let relative_gap = options.eigenvalue_cutoff(dimension)?;
        commutant_diagnostics = Some(commutant.diagnostics);
        for _ in 0..settings.max_attempts {
            let spectrum = symmetric_eigen(&sample(dimension, &commutant.matrices), options)?;
            let largest = spectrum
                .values
                .iter()
                .copied()
                .fold(E::Real::zero(), |m, x| m.max(Float::abs(x)));
            let gap_threshold = relative_gap * largest;
            finite([gap_threshold])?;
            let groups = groups(&spectrum.values, gap_threshold);
            let mut attempt = DecompositionAttempt {
                eigenvalues: spectrum.values,
                relative_gap,
                gap_threshold,
                groups,
                validation: None,
                failure: None,
            };
            if attempt.groups.len() != expected {
                attempt.failure = Some(DecompositionFailure::ComponentCount {
                    expected,
                    found: attempt.groups.len(),
                });
                attempts.push(attempt);
                continue;
            }
            let axes: Vec<_> = attempt
                .groups
                .iter()
                .map(|group| {
                    semisimple.dot(
                        &spectrum
                            .vectors
                            .slice_axis(Axis(1), Slice::from(group.clone())),
                    )
                })
                .collect();
            let validation = match validate_candidate(ambient, &split.center_axes, &axes, options) {
                Ok(validation) => validation,
                Err(LieError::ConstraintResidual {
                    operation,
                    index,
                    check,
                }) => {
                    attempt.failure = Some(DecompositionFailure::ConstraintResidual {
                        operation,
                        index,
                        check,
                    });
                    attempts.push(attempt);
                    continue;
                }
                Err(error) => return Err(error),
            };
            let passed = validation.passed();
            attempt.validation = Some(validation.clone());
            if !passed {
                attempt.failure = Some(DecompositionFailure::Validation);
            }
            attempts.push(attempt);
            if passed {
                accepted = Some((axes, validation));
                break;
            }
        }
    }
    let Some((axes, validation)) = accepted else {
        return Err(LieError::DecompositionExhausted {
            seed: settings.seed,
            attempts,
        });
    };
    let center = if split.center_axes.ncols() == 0 {
        None
    } else {
        Some(component(
            adjoint.family(),
            split.center_coordinates().clone(),
            options,
        )?)
    };
    let simple_ideals = axes
        .iter()
        .map(|q| component(adjoint.family(), split.orthonormal.forward.dot(q), options))
        .collect::<Result<Vec<_>, _>>()?;
    for (index, component) in center.iter().chain(&simple_ideals).enumerate() {
        check_reconstruction(component, index)?;
    }
    let diagnostics = DecompositionDiagnostics {
        center: split.diagnostics().clone(),
        semisimple_commutant: commutant_diagnostics,
        attempts,
        validation,
    };
    Ok(LieDecomposition {
        family: adjoint.family(),
        center,
        simple_ideals,
        diagnostics,
        options: *options,
        decomposition_options: *settings,
    })
}
