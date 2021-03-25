use criterion::{criterion_group, criterion_main, Criterion};
use ff::Field;
use threshold_crypto::poly::{BivarCommitment, BivarPoly, Commitment, Poly};
use threshold_crypto::Fr;

const TEST_DEGREES: [usize; 4] = [5, 10, 20, 40];
const TEST_THRESHOLDS: [usize; 4] = [5, 10, 20, 40];
const RNG_SEED: [u8; 16] = *b"0123456789abcdef";

mod poly_benches {
    use super::*;
    use rand::SeedableRng;
    use rand_xorshift::XorShiftRng;

    /// Benchmarks multiplication of two polynomials.
    fn multiplication(c: &mut Criterion) {
        let mut rng = XorShiftRng::from_seed(RNG_SEED);
        c.bench_function_over_inputs(
            "Polynomial multiplication",
            move |b, &&deg| {
                let rand_factors = || {
                    let lhs = Poly::random(deg, &mut rng);
                    let rhs = Poly::random(deg, &mut rng);
                    (lhs, rhs)
                };
                b.iter_with_setup(rand_factors, |(lhs, rhs)| &lhs * &rhs)
            },
            &TEST_DEGREES,
        );
    }

    /// Benchmarks subtraction of two polynomials
    fn subtraction(c: &mut Criterion) {
        let mut rng = XorShiftRng::from_seed(RNG_SEED);
        c.bench_function_over_inputs(
            "Polynomial subtraction",
            move |b, &&deg| {
                let rand_factors = || {
                    let lhs = Poly::random(deg, &mut rng);
                    let rhs = Poly::random(deg, &mut rng);
                    (lhs, rhs)
                };
                b.iter_with_setup(rand_factors, |(lhs, rhs)| &lhs - &rhs)
            },
            &TEST_DEGREES,
        );
    }

    /// Benchmarks addition of two polynomials
    fn addition(c: &mut Criterion) {
        let mut rng = XorShiftRng::from_seed(RNG_SEED);
        c.bench_function_over_inputs(
            "Polynomial addition",
            move |b, &&deg| {
                let rand_factors = || {
                    let lhs = Poly::random(deg, &mut rng);
                    let rhs = Poly::random(deg, &mut rng);
                    (lhs, rhs)
                };
                b.iter_with_setup(rand_factors, |(lhs, rhs)| &lhs + &rhs)
            },
            &TEST_DEGREES,
        );
    }

    /// Benchmarks Lagrange interpolation for a polynomial.
    fn interpolate(c: &mut Criterion) {
        let mut rng = XorShiftRng::from_seed(RNG_SEED);
        c.bench_function_over_inputs(
            "Polynomial interpolation",
            move |b, &&deg| {
                b.iter_with_setup(
                    || {
                        (0..=deg)
                            .map(|i| (i, Fr::random(&mut rng)))
                            .collect::<Vec<_>>()
                    },
                    Poly::interpolate,
                )
            },
            &TEST_DEGREES,
        );
    }

    criterion_group! {
        name = poly_benches;
        config = Criterion::default();
        targets = multiplication, interpolate, addition, subtraction,
    }
}

mod commitment_benches {
    use super::*;
    use rand::SeedableRng;
    use rand_xorshift::XorShiftRng;

    /// Benchmarks serialization of a univariate commitment
    fn serialization(c: &mut Criterion) {
        let mut rng = XorShiftRng::from_seed(RNG_SEED);
        c.bench_function_over_inputs(
            "Commitment Serialization",
            move |b, &&deg| {
                let rand_factors = || {
                    let poly = Poly::random(deg, &mut rng).commitment();
                    poly
                };
                b.iter_with_setup(rand_factors, |p| {
                    bincode::serialize(&p).expect("lhs boom ser")
                })
            },
            &TEST_DEGREES,
        );
    }

    /// Benchmarks deserialization of a univariate commitment
    fn deserialization(c: &mut Criterion) {
        let mut rng = XorShiftRng::from_seed(RNG_SEED);
        c.bench_function_over_inputs(
            "Commitment Deserialization",
            move |b, &&deg| {
                let rand_factors = || {
                    let poly = Poly::random(deg, &mut rng).commitment();
                    let poly_ser = bincode::serialize(&poly).expect("lhs boom ser");
                    poly_ser
                };
                b.iter_with_setup(rand_factors, |p| {
                    bincode::deserialize::<Commitment>(&p).expect("lhs boom deser")
                })
            },
            &TEST_DEGREES,
        );
    }

    criterion_group! {
        name = commitment_benches;
        config = Criterion::default();
        targets = serialization, deserialization,
    }
}

mod bicommitment_benches {
    use super::*;
    use rand::SeedableRng;
    use rand_xorshift::XorShiftRng;

    /// Benchmarks serialization of a bivariate commitment
    fn serialization(c: &mut Criterion) {
        let mut rng = XorShiftRng::from_seed(RNG_SEED);
        c.bench_function_over_inputs(
            "BiCommitment Serialization",
            move |b, &&deg| {
                let rand_factors = || {
                    let bipoly = BivarPoly::random(deg, &mut rng).commitment();
                    bipoly
                };
                b.iter_with_setup(rand_factors, |b| {
                    bincode::serialize(&b).expect("rhs boom ser")
                })
            },
            &TEST_DEGREES,
        );
    }

    /// Benchmarks deserialization of a bivariate commitment
    fn deserialization(c: &mut Criterion) {
        let mut rng = XorShiftRng::from_seed(RNG_SEED);
        c.bench_function_over_inputs(
            "BiCommitment Deserialization",
            move |b, &&deg| {
                let rand_factors = || {
                    let bipoly = BivarPoly::random(deg, &mut rng).commitment();
                    let bipoly_ser = bincode::serialize(&bipoly).expect("rhs boom ser");
                    bipoly_ser
                };
                b.iter_with_setup(rand_factors, |b| {
                    bincode::deserialize::<BivarCommitment>(&b).expect("lhs boom deser")
                })
            },
            &TEST_DEGREES,
        );
    }

    criterion_group! {
        name = bicommitment_benches;
        config = Criterion::default();
        targets = serialization, deserialization,
    }
}

mod public_key_set_benches {
    use super::*;
    use rand::SeedableRng;
    use rand_xorshift::XorShiftRng;
    use std::collections::BTreeMap;
    use threshold_crypto::SecretKeySet;

    /// Benchmarks combining signatures
    fn combine_signatures(c: &mut Criterion) {
        let mut rng = XorShiftRng::from_seed(RNG_SEED);
        let msg = "Test message";
        c.bench_function_over_inputs(
            "Combine Signatures",
            move |b, &&threshold| {
                let sk_set = SecretKeySet::random(threshold, &mut rng);
                let pk_set = sk_set.public_keys();
                let sigs: BTreeMap<_, _> = (0..=threshold)
                    .map(|i| {
                        let sig = sk_set.secret_key_share(i).sign(msg);
                        (i, sig)
                    })
                    .collect();
                b.iter(|| {
                    pk_set
                        .combine_signatures(&sigs)
                        .expect("could not combine signatures");
                })
            },
            &TEST_THRESHOLDS,
        );
    }

    criterion_group! {
        name = public_key_set_benches;
        config = Criterion::default();
        targets = combine_signatures,
    }
}

criterion_main!(
    poly_benches::poly_benches,
    public_key_set_benches::public_key_set_benches,
    commitment_benches::commitment_benches,
    bicommitment_benches::bicommitment_benches,
);
