use core::hash;
use std::result;

use ark_bls12_381::Fr;
use ark_r1cs_std::alloc::AllocVar;
use ark_r1cs_std::eq::EqGadget;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::fields::FieldVar;
use ark_r1cs_std::uint8::UInt8;
use ark_r1cs_std::{R1CSVar, ToBitsGadget, ToBytesGadget};
use ark_relations::ns;
use ark_relations::r1cs::{ConstraintSystemRef, Result, SynthesisError};
use ark_crypto_primitives::crh::sha256::constraints::{DigestVar, Sha256Gadget};

pub type Curve = ark_bls12_381::Bls12_381;
pub type CircuitField = Fr;

use crate::model::{Board, Ship};

#[derive(Copy, Clone, Debug)]
pub struct BoardCircuit {
    pub board: Board,
    pub salt: [u8; 32], // TODO rozważyć typy
    pub hash: [u8; 32],
}

struct ShipVars {
    pub x: FpVar<CircuitField>,
    pub y: FpVar<CircuitField>,
    pub size: FpVar<CircuitField>,
    pub direction: FpVar<CircuitField>,
}

// TODO: wczytywać UInt8 i później konwersja na FpVar
// TODO: sprawdzic czy statki sa dobrze ulozone -czy dobre dlugosci, w srodku planszy, nie stykaja sie
impl ark_relations::r1cs::ConstraintSynthesizer<CircuitField> for BoardCircuit {
  fn generate_constraints(self, cs: ConstraintSystemRef<CircuitField>) -> Result<()> {
    // Create private variables for each ship
    let ships_vars: [ShipVars; 15] = self.board.ships.map(|ship| create_ship_vars(&ship, &cs).unwrap());
    // Create private variable for hash salt
    let salt_vars: [UInt8<CircuitField>; 32] = self.salt.map(|bit| UInt8::new_witness(ns!(cs, "salt"), || Ok(bit)).unwrap());
    // Create input for hash of ships
    let hash_vars: [UInt8<CircuitField>; 32] = self.hash.map(|bit| UInt8::new_input(ns!(cs, "hash"), || Ok(bit)).unwrap());

    let mut hash_gadget: Sha256Gadget<CircuitField> = Sha256Gadget::default();
    // hash every ship
    ships_vars.iter().for_each(|ship_vars| hash_gadget.update(&[
      cast_fp_var_to_uint8(&ship_vars.x).unwrap(),
      cast_fp_var_to_uint8(&ship_vars.y).unwrap(),
      cast_fp_var_to_uint8(&ship_vars.size).unwrap(),
      cast_fp_var_to_uint8(&ship_vars.direction).unwrap(),
    ]).unwrap());
    // add salt
    hash_gadget.update(&salt_vars)?;
    // compute hash
    let digest_var: DigestVar<CircuitField> = hash_gadget.finalize().unwrap();
    // compare the hashes
    hash_vars.iter().zip(digest_var.0).for_each(|(h1, h2)| h1.enforce_equal(&h2).unwrap());

    Ok(())
  }
}

fn create_ship_vars(ship: &Ship, cs: &ConstraintSystemRef<CircuitField>) -> Result<ShipVars> {
    Ok(ShipVars {
        x: FpVar::new_witness(ns!(cs, "shipX"), || Ok(CircuitField::from(ship.x)))?,
        y: FpVar::new_witness(ns!(cs, "shipY"), || Ok(CircuitField::from(ship.y)))?,
        size: FpVar::new_witness(ns!(cs, "shipSize"), || Ok(CircuitField::from(ship.size)))?,
        direction: FpVar::new_witness(ns!(cs, "shipDirection"), || Ok(CircuitField::from(ship.direction as u8)))?,
    })
}

// Need to check earlier if the var is less than 8 bytes long
fn cast_fp_var_to_uint8(var: &FpVar<CircuitField>) -> Result<UInt8<CircuitField>> {
  let bytes = FpVar::to_bytes(&var)?;
  // to_bytes function returns [var, 0, 0, 0, ...] -- a vector of UInt8 of len 32 (in case of not too large var)
  Ok(bytes[0].clone())
}
