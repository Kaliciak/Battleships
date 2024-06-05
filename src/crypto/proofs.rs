use crate::{
    board_circuit::CircuitField,
    utils::{
        log::{Log, Logger},
        result::Res,
    },
    Board, BoardCircuit,
};
use ark_groth16::{r1cs_to_qap::LibsnarkReduction, Groth16};
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
use sha2::{Digest, Sha256};

use super::keys::ArkKeys;

/// Proof that the sender has properly constructed game board
#[derive(Debug)]
pub struct BoardCorrectnessProof(
    pub ark_groth16::Proof<ark_ec::bls12::Bls12<ark_bls12_381::Config>>,
);

impl Serialize for BoardCorrectnessProof {
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

impl<'de> Deserialize<'de> for BoardCorrectnessProof {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v: Vec<u8> = Vec::<u8>::deserialize(deserializer)?;
        let x = ark_groth16::Proof::<ark_ec::bls12::Bls12::<ark_bls12_381::Config>>::deserialize_uncompressed_unchecked(&v[..]);
        match x {
            Ok(proof) => Ok(BoardCorrectnessProof(proof)),
            Err(_) => Err(de::Error::custom(
                "Error while deserializing board correctness proof...",
            )),
        }
    }
}

impl BoardCorrectnessProof {
    pub fn create(board: Board, logger: Logger, mut keys: ArkKeys) -> Res<(Self, BoardCircuit)> {
        let salt = [1; 32];
        let ships = board.ships;

        // create a Sha256 object
        let mut hasher = Sha256::new();

        ships
            .iter()
            .for_each(|ship| hasher.update([ship.x, ship.y, ship.size, ship.direction as u8]));
        hasher.update(salt);

        // read hash digest and consume hasher
        let hash_result = hasher.finalize();

        let real_circuit = BoardCircuit {
            board: Board { ships },
            salt,
            hash: hash_result.into(),
        };

        let (_, pk) = &*(keys.acquire()?);

        let now = std::time::Instant::now();
        let mut rng: StdRng = StdRng::seed_from_u64(1);
        let proof: ark_groth16::Proof<ark_ec::bls12::Bls12<ark_bls12_381::Config>> =
            Groth16::<_, LibsnarkReduction>::prove(pk, real_circuit, &mut rng)?;
        let elapsed = now.elapsed();
        logger.log_message(&format!("Proof generated. Time: {:.2?}", elapsed))?;

        Ok((BoardCorrectnessProof(proof), real_circuit))
    }
    pub fn is_correct(&mut self, hash: [u8; 32], mut keys: ArkKeys) -> Res<bool> {
        let (vk, _) = &*(keys.acquire()?);
        let mut input = [CircuitField::zero(); 8 * 32];
        for i in 0..32 {
            for j in 0..8 {
                if hash[i] >> j & 1 == 1 {
                    input[i * 8 + j] = CircuitField::one();
                }
            }
        }
        Ok(Groth16::<_, LibsnarkReduction>::verify(
            vk, &input, &self.0,
        )?)
    }
}
