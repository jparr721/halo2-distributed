//! Keygen Definitions

use ff::{Field, PrimeField};
use halo2curves::{bn256::Bn256, CurveExt};
use serde_derive::{Deserialize, Serialize};

use crate::{
    arithmetic::{parallelize, CurveAffine},
    distributed::{
        dispatcher::{Taskable, WorkerMethod},
        net::{from_bytes, to_bytes},
    },
    plonk::permutation::Argument,
    poly::{commitment::Params, kzg::commitment::ParamsKZG, EvaluationDomain},
};

/// Distributed request to perform keygen
#[derive(Debug, Clone)]
pub struct KeygenTaskKZG<'a, C: CurveAffine> {
    pub params: ParamsKZG<Bn256>,
    pub domain: &'a EvaluationDomain<C::Scalar>,
    pub p: &'a Argument,
    pub mapping: Vec<Vec<(usize, usize)>>,
}

impl<'a, C: CurveAffine> KeygenTaskKZG<'a, C> {
    pub fn new(
        params: ParamsKZG<Bn256>,
        domain: &'a EvaluationDomain<C::Scalar>,
        p: &'a Argument,
        mapping: Vec<Vec<(usize, usize)>>,
    ) -> Self {
        KeygenTaskKZG {
            params,
            domain,
            p,
            mapping,
        }
    }

    pub fn zero(&self) -> <<C as CurveAffine>::CurveExt as CurveExt>::ScalarExt {
        C::Scalar::ZERO
    }

    pub fn delta(&self) -> <<C as CurveAffine>::CurveExt as CurveExt>::ScalarExt {
        C::Scalar::DELTA
    }
}
