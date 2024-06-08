use std::{marker::PhantomData, ops, usize};

use crate::{
    circuit::commons::CircuitField,
    utils::{
        log::{Log, Logger},
        result::Res,
    },
};
use ark_bls12_381::FrConfig;
use ark_ff::{Fp, MontBackend};
use ark_groth16::{r1cs_to_qap::LibsnarkReduction, Groth16};
use ark_relations::r1cs::ConstraintSynthesizer;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_snark::SNARK;
use ark_std::{
    rand::{rngs::StdRng, SeedableRng},
    One, Zero,
};
use serde::{
    de,
    ser::{self},
    Deserialize, Serialize,
};

use super::keys::ArkKeys;

/// Proof that the sender has properly constructed game board
#[derive(Debug)]
pub struct CorrectnessProof<T>(
    pub ark_groth16::Proof<ark_ec::bls12::Bls12<ark_bls12_381::Config>>,
    PhantomData<T>,
);

impl<T> Serialize for CorrectnessProof<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut v = Vec::<u8>::new();
        if self.0.serialize_uncompressed(&mut v).is_err() {
            return Err(ser::Error::custom(
                "Error while serializing board correctness proof...",
            ));
        };
        v.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for CorrectnessProof<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v: Vec<u8> = Vec::<u8>::deserialize(deserializer)?;
        let x = ark_groth16::Proof::<ark_ec::bls12::Bls12::<ark_bls12_381::Config>>::deserialize_uncompressed_unchecked(&v[..]);
        match x {
            Ok(proof) => Ok(CorrectnessProof(proof, PhantomData)),
            Err(_) => Err(de::Error::custom(
                "Error while deserializing board correctness proof...",
            )),
        }
    }
}

impl<T: ConstraintSynthesizer<CircuitField>> CorrectnessProof<T> {
    pub fn create(real_circuit: T, logger: Logger, mut keys: ArkKeys) -> Res<Self> {
        let (_, pk) = &*(keys.acquire()?);

        let now = std::time::Instant::now();
        let mut rng: StdRng = StdRng::seed_from_u64(1);
        let proof: ark_groth16::Proof<ark_ec::bls12::Bls12<ark_bls12_381::Config>> =
            Groth16::<_, LibsnarkReduction>::prove(pk, real_circuit, &mut rng)?;
        let elapsed = now.elapsed();
        logger.log_message(&format!("Proof generated. Time: {:.2?}", elapsed))?;

        Ok(CorrectnessProof(proof, PhantomData))
    }
    pub fn is_correct(&mut self, input: PublicInput, mut keys: ArkKeys) -> Res<bool> {
        let (vk, _) = &*(keys.acquire()?);
        Ok(Groth16::<_, LibsnarkReduction>::verify(
            vk, &input.0, &self.0,
        )?)
    }
}

pub struct PublicInput(Vec<Fp<MontBackend<FrConfig, 4>, 4>>);

impl From<Vec<u8>> for PublicInput {
    fn from(value: Vec<u8>) -> Self {
        let size: usize = value.len();
        let mut input = vec![CircuitField::zero(); 8 * size];
        for i in 0..32 {
            for j in 0..8 {
                if value[i] >> j & 1 == 1 {
                    input[i * 8 + j] = CircuitField::one();
                }
            }
        }
        PublicInput(input)
    }
}

impl<T: Into<CircuitField>> ops::Add<T> for PublicInput {
    type Output = PublicInput;

    fn add(mut self, rhs: T) -> Self::Output {
        self.0.push(rhs.into());
        self
    }
}
