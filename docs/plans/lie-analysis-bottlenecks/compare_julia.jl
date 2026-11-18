using LieAlgebra, LinearAlgebra, Random, Statistics

BLAS.set_num_threads(1)
println("Julia ", VERSION, ", BLAS ", BLAS.get_config(), ", threads ", BLAS.get_num_threads())

const UNIT = Matrix{ComplexF64}(I, 2, 2)
const X = ComplexF64[0 1; 1 0]
const Y = ComplexF64[0 -im; im 0]
const Z = ComplexF64[1 0; 0 -1]
const LOCAL = vcat([kron(a, UNIT) for a in (X, Y, Z)], [kron(UNIT, a) for a in (X, Y, Z)])
const PAULI = MatrixBasis([kron(a, b) for a in (UNIT, X, Y, Z) for b in (UNIT, X, Y, Z)][2:end])

function majorana_basis(modes)
    gammas = [reduce(kron, [k < site ? Z : k == site ? axis : UNIT for k in 1:modes])
              for site in 1:modes for axis in (X, Y)]
    return MatrixBasis([-im * gammas[a] * gammas[b] for a in 1:2modes for b in a+1:2modes])
end

function collect_timings(basis, expected)
    t_ad = @elapsed ad = adjoint_matrices(basis)
    t_commutant = @elapsed comm = commutant(ad)
    rng = MersenneTwister(816)
    t_split = @elapsed begin
        factors = eigen(generic_commutant_element(rng, comm))
        groups = LieAlgebra.eigenspace_groups(factors.values)
        transformed = transform_basis(basis, factors.vectors)
        parts = [component_basis(transformed, group) for group in groups]
    end
    @assert length(comm) == length(expected)
    @assert sort(length.(parts)) == expected
    return (t_ad, t_commutant, t_split)
end

function measure_case(label, basis, expected)
    collect_timings(basis, expected)
    @assert sort(length.(decompose(MersenneTwister(816), basis))) == expected
    measurements = map(1:5) do _
        GC.gc()
        stages = collect_timings(basis, expected)
        GC.gc()
        rng = MersenneTwister(816)
        t_total = @elapsed parts = decompose(rng, basis)
        @assert sort(length.(parts)) == expected
        return (stages..., t_total)
    end
    medians = [median(getindex.(measurements, k))*1_000 for k in 1:4]
    println(label, " medians (ms) for adjoint, commutant, split, decompose ", medians)
end

function thin_nullspace(A; atol=1e-10)
    @assert size(A, 1) >= size(A, 2)
    factors = svd(A; full=false)
    start = count(>(atol), factors.S) + 1
    return copy((@view factors.Vt[start:end, :])')
end

function measure_nullspace()
    constraints = LieAlgebra.commutant_matrix(adjoint_matrices(PAULI))
    full = nullspace(constraints; atol=1e-10)
    thin = thin_nullspace(constraints)
    @assert size(full) == size(thin) == (225, 1)
    @assert norm(full * full' - thin * thin') < 1e-12
    @assert norm(constraints * thin) < 1e-10
    for (label, solve) in (("full", A -> nullspace(A; atol=1e-10)), ("thin", thin_nullspace))
        solve(constraints)
        measurements = map(1:5) do _
            GC.gc()
            @timed solve(constraints)
        end
        println(label, " nullspace (ms, allocated MiB) ",
                (median(getproperty.(measurements, :time))*1_000,
                 median(getproperty.(measurements, :bytes))/2^20))
    end
end

function expect_reality_error(label, basis)
    try
        decompose(MersenneTwister(816), basis)
        error("expected the real-matrix check to fail")
    catch e
        @assert e isa ArgumentError && occursin("Matrix is not real", sprint(showerror, e))
        println(label, " rejected by the real-matrix check")
    end
end

struct ZeroRNG <: AbstractRNG end
Random.randn(::ZeroRNG) = 0.0

function check_stability()
    mixed = copy(LOCAL)
    mixed[4] += mixed[1]
    mixed_basis = MatrixBasis(mixed)
    @assert is_closed(mixed_basis)
    expect_reality_error("nonorthonormal", mixed_basis)
    expect_reality_error("abelian center of dimension 2", MatrixBasis(LOCAL[[3, 6]]))

    basis = MatrixBasis([LOCAL[1], LOCAL[2], (LOCAL[3] + LOCAL[4]) / sqrt(2)])
    @assert !is_closed(basis)
    @assert length.(decompose(MersenneTwister(816), basis)) == [3]
    println("nonclosed input returned a single 3-dimensional component")

    @assert length.(decompose(ZeroRNG(), MatrixBasis(LOCAL))) == [6]
    println("forced spectral collision returned a single 6-dimensional component")
end

measure_case("spin-half-1", MatrixBasis([X, Y, Z]), [3])
measure_case("spin-half-2", MatrixBasis(LOCAL), [3, 3])
measure_case("pauli-2", PAULI, [15])
measure_case("fermion-3", majorana_basis(3), [15])
measure_nullspace()
check_stability()
