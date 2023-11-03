use super::fields::{fp::Fp, fp2::Fp2, fq::Fq};
use crate::ff::WithSmallOrderMulGroup;
use crate::ff::{Field, PrimeField};
use crate::group::{prime::PrimeCurveAffine, Curve, Group as _, GroupEncoding};
use crate::hash_to_curve::svdw_hash_to_curve;
use crate::{Coordinates, CurveAffine, CurveExt};
use core::cmp;
use core::fmt::Debug;
use core::iter::Sum;
use core::ops::{Add, Mul, Neg, Sub};
use group::cofactor::CofactorGroup;
use rand::RngCore;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

#[cfg(feature = "derive_serde")]
use serde::{Deserialize, Serialize};

use crate::{
    impl_add_binop_specify_output, impl_binops_additive, impl_binops_additive_specify_output,
    impl_binops_multiplicative, impl_binops_multiplicative_mixed, impl_sub_binop_specify_output,
    new_curve_impl,
};

const G1_GENERATOR_X: Fp = Fp::from_raw([
    0x9ffffcd2ffffffff,
    0xa2a7e8c30006b945,
    0xe4a7a5fe8fadffd6,
    0x443f9a5cda8a6c7b,
    0xa803ca76f439266f,
    0x0130e0000d7f70e4,
    0x2400000000002400,
]);
const G1_GENERATOR_Y: Fp = Fp::from_raw([0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

const PLUTO_A: Fp = Fp::ZERO;
const PLUTO_B: Fp = Fp::from_raw([0x39, 0, 0, 0, 0, 0, 0]);

const ERIS_GENERATOR_X: Fq = Fq::from_raw([
    0x1ffffcd2ffffffff,
    0x9ca7e85d60050af4,
    0xe4a775fe8e177fd6,
    0x443f9a5c7a8a6c7b,
    0xa803ca76f439266f,
    0x0130e0000d7f70e4,
    0x2400000000002400,
]);
const ERIS_GENERATOR_Y: Fq = Fq::from_raw([0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

const ERIS_A: Fq = Fq::ZERO;
const ERIS_B: Fq = Fq::from_raw([0x39, 0, 0, 0, 0, 0, 0]);

const G2_GENERATOR_X: Fp2 = Fp2 {
    // 0x13576c81faf3a13fd815d0e9bd54b845ee935948b84498b27ca972bfb93722e223c9e276a4ebe7559cfc86dd865f07d64f2b5fe6556f9066
    c0: Fp::from_raw([
        0x4f2b5fe6556f9066,
        0x9cfc86dd865f07d6,
        0x23c9e276a4ebe755,
        0x7ca972bfb93722e2,
        0xee935948b84498b2,
        0xd815d0e9bd54b845,
        0x13576c81faf3a13f,
    ]),

    //0x142164cb875db0465e5092f9380f44f555243d011699b7393029f2d201554727aeb383298fdf5847b9b3dff01bbe8d63fe7c781a8fd7bf21
    c1: Fp::from_raw([
        0xfe7c781a8fd7bf21,
        0xb9b3dff01bbe8d63,
        0xaeb383298fdf5847,
        0x3029f2d201554727,
        0x55243d011699b739,
        0x5e5092f9380f44f5,
        0x142164cb875db046,
    ]),
};
const G2_GENERATOR_Y: Fp2 = Fp2 {
    //0x2239f7408ead478c58e88d4df1e7418c42fdbb92e64ba85aa4dc17d7dace3f32eb471c004db774bfe78574aca67b3898cd1b78ad106ab9fe
    c0: Fp::from_raw([
        0xcd1b78ad106ab9fe,
        0xe78574aca67b3898,
        0xeb471c004db774bf,
        0xa4dc17d7dace3f32,
        0x42fdbb92e64ba85a,
        0x58e88d4df1e7418c,
        0x2239f7408ead478c,
    ]),

    // 0x1260b04d51136590dbb53dfd7caf450aeca714555bbe4f079ca65d97eb28fc9fc697b4e10bbcd9e0539ef82a731fb88ed49e3c080e6d945d
    c1: Fp::from_raw([
        0xd49e3c080e6d945d,
        0x539ef82a731fb88e,
        0xc697b4e10bbcd9e0,
        0x9ca65d97eb28fc9f,
        0xeca714555bbe4f07,
        0xdbb53dfd7caf450a,
        0x1260b04d51136590,
    ]),
};

const TRITON_A: Fp2 = Fp2::ZERO;

// u + 3
const TRITON_B: Fp2 = Fp2 {
    c0: Fp::from_raw([0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
    c1: Fp::ONE,
};

impl group::cofactor::CofactorGroup for G1 {
    type Subgroup = G1;

    fn clear_cofactor(&self) -> Self {
        *self
    }

    fn into_subgroup(self) -> CtOption<Self::Subgroup> {
        CtOption::new(self, 1.into())
    }

    fn is_torsion_free(&self) -> Choice {
        1.into()
    }
}

new_curve_impl!(
    (pub),
    G1,
    G1Affine,
    false,
    Fp,
    Fq,
    (G1_GENERATOR_X,G1_GENERATOR_Y),
    PLUTO_A,
    PLUTO_B,
    "pluto",
    |curve_id, domain_prefix| svdw_hash_to_curve(curve_id, domain_prefix, G1::SVDW_Z),
);

impl group::cofactor::CofactorGroup for Eris {
    type Subgroup = Eris;

    fn clear_cofactor(&self) -> Self {
        *self
    }

    fn into_subgroup(self) -> CtOption<Self::Subgroup> {
        CtOption::new(self, 1.into())
    }

    fn is_torsion_free(&self) -> Choice {
        1.into()
    }
}

impl G1 {
    const SVDW_Z: Fp = Fp::ONE;
}

new_curve_impl!(
    (pub),
    Eris,
    ErisAffine,
    false,
    Fq,
    Fp,
    (ERIS_GENERATOR_X,ERIS_GENERATOR_Y),
    ERIS_A,
    ERIS_B,
    "eris",
    |curve_id, domain_prefix| svdw_hash_to_curve(curve_id, domain_prefix, Eris::SVDW_Z),
);

impl CofactorGroup for G2 {
    type Subgroup = G2;

    fn clear_cofactor(&self) -> Self {
        // cofactor = 2*p - q
        //0x24000000000024000130e0000d7f70e4a803ca76f439266f443f9a5d3a8a6c7be4a7d5fe91447fd6a8a7e928a00867971ffffcd300000001
        let e: [u8; 56] = [
            0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x24, 0x00, 0x01, 0x30, 0xe0, 0x00, 0x0d, 0x7f,
            0x70, 0xe4, 0xa8, 0x03, 0xca, 0x76, 0xf4, 0x39, 0x26, 0x6f, 0x44, 0x3f, 0x9a, 0x5d,
            0x3a, 0x8a, 0x6c, 0x7b, 0xe4, 0xa7, 0xd5, 0xfe, 0x91, 0x44, 0x7f, 0xd6, 0xa8, 0xa7,
            0xe9, 0x28, 0xa0, 0x08, 0x67, 0x97, 0x1f, 0xff, 0xfc, 0xd3, 0x00, 0x00, 0x00, 0x01,
        ];

        // self * TRITON_COFACTOR
        let mut acc = G2::identity();
        for bit in e
            .iter()
            .flat_map(|byte| (0..8).rev().map(move |i| Choice::from((byte >> i) & 1u8)))
            .skip(1)
        {
            acc = acc.double();
            acc = G2::conditional_select(&acc, &(acc + self), bit);
        }
        acc
    }

    fn into_subgroup(self) -> CtOption<Self::Subgroup> {
        unimplemented!()
    }

    fn is_torsion_free(&self) -> Choice {
        // group order = p
        let e: [u8; 56] = [
            0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x24, 0x00, 0x01, 0x30, 0xe0, 0x00, 0x0d, 0x7f,
            0x70, 0xe4, 0xa8, 0x03, 0xca, 0x76, 0xf4, 0x39, 0x26, 0x6f, 0x44, 0x3f, 0x9a, 0x5c,
            0xda, 0x8a, 0x6c, 0x7b, 0xe4, 0xa7, 0xa5, 0xfe, 0x8f, 0xad, 0xff, 0xd6, 0xa2, 0xa7,
            0xe8, 0xc3, 0x00, 0x06, 0xb9, 0x45, 0x9f, 0xff, 0xfc, 0xd3, 0x00, 0x00, 0x00, 0x01,
        ];
        // self * GROUP_ORDER;
        let mut acc = G2::identity();
        for bit in e
            .iter()
            .flat_map(|byte| (0..8).rev().map(move |i| Choice::from((byte >> i) & 1u8)))
            .skip(1)
        {
            acc = acc.double();
            acc = G2::conditional_select(&acc, &(acc + self), bit);
        }
        acc.is_identity()
    }
}

impl Eris {
    const SVDW_Z: Fq = Fq::ONE;
}

new_curve_impl!(
    (pub),
    G2,
    G2Affine,
    false,
    Fp2,
    Fq,
    (G2_GENERATOR_X,G2_GENERATOR_Y),
    TRITON_A,
    TRITON_B,
    "triton",
    |_, _| unimplemented!(),
);

#[test]
fn test_curve_pluto() {
    crate::tests::curve::curve_tests::<G1>();
}
#[test]
fn test_curve_eris() {
    crate::tests::curve::curve_tests::<Eris>();
}
#[test]
fn test_curve_triton() {
    crate::tests::curve::curve_tests::<G2>();
}

#[test]
fn test_serialization() {
    crate::tests::curve::random_serialization_test::<G1>();
    crate::tests::curve::random_serialization_test::<Eris>();
    crate::tests::curve::random_serialization_test::<G2>();
    #[cfg(feature = "derive_serde")]
    crate::tests::curve::random_serde_test::<G1>();
    #[cfg(feature = "derive_serde")]
    crate::tests::curve::random_serde_test::<Eris>();
    #[cfg(feature = "derive_serde")]
    crate::tests::curve::random_serde_test::<G2>();
}

#[test]
fn test_hash_to_curve() {
    crate::tests::curve::hash_to_curve_test::<G1>();
    crate::tests::curve::hash_to_curve_test::<Eris>();
}

#[test]
fn test_endo_consistency() {
    let g = Eris::generator();
    assert_eq!(g * Fp::ZETA, g.endo());

    let g = G1::generator();
    assert_eq!(g * Fq::ZETA, g.endo());

    let g = G2::generator();
    assert_eq!(g * Fq::ZETA, g.endo());
}
