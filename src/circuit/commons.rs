use ark_bls12_381::Fr;
use ark_crypto_primitives::crh::sha256::constraints::{DigestVar, Sha256Gadget};
use ark_r1cs_std::alloc::AllocVar;
use ark_r1cs_std::boolean::Boolean;
use ark_r1cs_std::eq::EqGadget;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::fields::FieldVar;
use ark_r1cs_std::uint8::UInt8;
use ark_r1cs_std::ToBytesGadget;
use ark_relations::ns;
use ark_relations::r1cs::{ConstraintSystemRef, Result};
use std::cmp::Ordering;

pub type CircuitField = Fr;

use crate::model::Ship;

pub const SHIPS_SIZES: [usize; 15] = [1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 4, 4, 5];

pub struct ShipVars {
    pub x: FpVar<CircuitField>,
    pub y: FpVar<CircuitField>,
    pub size: FpVar<CircuitField>,
    pub size_numerical: Option<usize>,
    pub direction: FpVar<CircuitField>,
    pub is_vertical: Boolean<CircuitField>,
}

pub fn create_ship_vars(ship: &Ship, cs: &ConstraintSystemRef<CircuitField>) -> Result<ShipVars> {
    let direction = FpVar::new_witness(ns!(cs, "shipDirection"), || {
        Ok(CircuitField::from(ship.direction as u8))
    })?;
    // direction <= 1
    FpVar::enforce_cmp(&direction, &FpVar::one(), Ordering::Less, true)?;

    let is_vertical = FpVar::is_eq(&direction, &FpVar::zero()).unwrap();

    Ok(ShipVars {
        x: FpVar::new_witness(ns!(cs, "shipX"), || Ok(CircuitField::from(ship.x)))?,
        y: FpVar::new_witness(ns!(cs, "shipY"), || Ok(CircuitField::from(ship.y)))?,
        size: FpVar::new_witness(ns!(cs, "shipSize"), || Ok(CircuitField::from(ship.size)))?,
        // To be set later
        size_numerical: None,
        direction,
        is_vertical,
    })
}

pub fn compute_hash(
    ships_vars: &[ShipVars; 15],
    salt_vars: &[UInt8<CircuitField>; 32],
) -> Result<DigestVar<CircuitField>> {
    let mut hash_gadget: Sha256Gadget<CircuitField> = Sha256Gadget::default();
    // Hash every ship
    ships_vars.iter().for_each(|ship_vars| {
        hash_gadget
            .update(&[
                cast_fp_var_to_uint8(&ship_vars.x).unwrap(),
                cast_fp_var_to_uint8(&ship_vars.y).unwrap(),
                cast_fp_var_to_uint8(&ship_vars.size).unwrap(),
                cast_fp_var_to_uint8(&ship_vars.direction).unwrap(),
            ])
            .unwrap()
    });
    // Add salt
    hash_gadget.update(salt_vars)?;
    // Compute hash
    hash_gadget.finalize()
}

// Need to also check if the var is less than 8 bytes long
pub fn cast_fp_var_to_uint8(var: &FpVar<CircuitField>) -> Result<UInt8<CircuitField>> {
    let bytes = FpVar::to_bytes(var)?;
    // to_bytes function returns [var, 0, 0, 0, ...] -- a vector of UInt8 of len 32 (in case of not too large var)
    Ok(bytes[0].clone())
}
