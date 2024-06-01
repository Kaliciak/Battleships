use ark_bls12_381::Fr;
use ark_crypto_primitives::crh::sha256::constraints::{DigestVar, Sha256Gadget};
use ark_r1cs_std::boolean::Boolean;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::uint8::UInt8;
use ark_r1cs_std::ToBytesGadget;
use ark_relations::r1cs::Result;

pub type CircuitField = Fr;

pub struct ShipVars {
    pub x: FpVar<CircuitField>,
    pub y: FpVar<CircuitField>,
    pub size: FpVar<CircuitField>,
    pub size_numerical: Option<usize>,
    pub direction: FpVar<CircuitField>,
    pub is_vertical: Boolean<CircuitField>,
}

pub fn compute_hash(ships_vars: &[ShipVars; 15], salt_vars: &[UInt8<CircuitField>; 32]) -> Result<DigestVar<CircuitField>> {
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
