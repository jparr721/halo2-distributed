//! Keygen Definitions

use ff::{Field, PrimeField};
use halo2curves::{bn256::Bn256, CurveExt};
use serde_derive::{Deserialize, Serialize};

use crate::{
    arithmetic::{parallelize, CurveAffine},
    distributed_util::{
        dispatcher::{Taskable, WorkerMethod},
        net::{from_bytes, to_bytes},
    },
    plonk::permutation::Argument,
    poly::{commitment::Params, kzg::commitment::ParamsKZG, EvaluationDomain},
};

/// Distributed request to perform keygen
#[derive(Debug, Clone)]
pub struct KeygenTaskKZG<'a, C: CurveAffine, P: Params<'a, C>> {
    pub params: &'a P,
    pub domain: &'a EvaluationDomain<C::Scalar>,
    pub p: &'a Argument,
    pub mapping: Vec<Vec<(usize, usize)>>,
}

impl<'a, C: CurveAffine, P: Params<'a, C>> KeygenTaskKZG<'a, C, P> {
    pub fn new(
        params: &'a P,
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
