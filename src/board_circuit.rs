use ark_bls12_381::Fr;
use ark_r1cs_std::alloc::AllocVar;
use ark_r1cs_std::boolean::Boolean;
use ark_r1cs_std::eq::EqGadget;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::select::CondSelectGadget;
use ark_r1cs_std::uint8::UInt8;
use ark_r1cs_std::ToBytesGadget;
use ark_relations::ns;
use ark_relations::r1cs::{ConstraintSystemRef, Result};
use ark_crypto_primitives::crh::sha256::constraints::{DigestVar, Sha256Gadget};

pub type Curve = ark_bls12_381::Bls12_381;
pub type CircuitField = Fr;

use crate::model::{Board, Ship};

#[derive(Copy, Clone, Debug)]
pub struct BoardCircuit {
    pub board: Board,
    pub salt: [u8; 32],
    pub hash: [u8; 32],
}

struct ShipVars {
    pub x: FpVar<CircuitField>,
    pub y: FpVar<CircuitField>,
    pub size: FpVar<CircuitField>,
    pub direction: FpVar<CircuitField>,
}

// TODO: sprawdzic czy statki sa dobrze ulozone -czy dobre dlugosci, w srodku planszy, nie stykaja sie
impl ark_relations::r1cs::ConstraintSynthesizer<CircuitField> for BoardCircuit {
  fn generate_constraints(self, cs: ConstraintSystemRef<CircuitField>) -> Result<()> {
    // Create private variables for each ship
    let ships_vars: [ShipVars; 15] = self.board.ships.map(|ship| create_ship_vars(&ship, &cs).unwrap());
    // Create private variable for hash salt
    let salt_vars: [UInt8<CircuitField>; 32] = self.salt.map(|bit| UInt8::new_witness(ns!(cs, "salt"), || Ok(bit)).unwrap());
    // Create input for hash of ships
    let hash_vars: [UInt8<CircuitField>; 32] = self.hash.map(|bit| UInt8::new_input(ns!(cs, "hash"), || Ok(bit)).unwrap());

    //--------------------------
    // Check if hash is correct
    let mut hash_gadget: Sha256Gadget<CircuitField> = Sha256Gadget::default();
    // Hash every ship
    ships_vars.iter().for_each(|ship_vars| hash_gadget.update(&[
      cast_fp_var_to_uint8(&ship_vars.x).unwrap(),
      cast_fp_var_to_uint8(&ship_vars.y).unwrap(),
      cast_fp_var_to_uint8(&ship_vars.size).unwrap(),
      cast_fp_var_to_uint8(&ship_vars.direction).unwrap(),
    ]).unwrap());
    // Add salt
    hash_gadget.update(&salt_vars)?;
    // Compute hash
    let digest_var: DigestVar<CircuitField> = hash_gadget.finalize().unwrap();
    // Compare the hashes
    hash_vars.iter().zip(digest_var.0).for_each(|(h1, h2)| h1.enforce_equal(&h2).unwrap());

    //-------------------------------
    // Check if the board is correct

    // Generate needed constants
    // constans[i] -- constant representing i
    // TODO check if all are needed
    let constants: Vec<FpVar<CircuitField>> = (0..=11).map(|number| FpVar::new_constant(ns!(cs, "constant"), CircuitField::from(number)).unwrap()).collect();

    // Check if all values are from the correct range
    for ship_index in 0..15 {
      // 1 <= x, y <= 10
      FpVar::enforce_cmp(&ships_vars[ship_index].x, &constants[1], std::cmp::Ordering::Greater, true)?;
      FpVar::enforce_cmp(&ships_vars[ship_index].x, &constants[10], std::cmp::Ordering::Less, true)?;
      FpVar::enforce_cmp(&ships_vars[ship_index].y, &constants[1], std::cmp::Ordering::Greater, true)?;
      FpVar::enforce_cmp(&ships_vars[ship_index].y, &constants[10], std::cmp::Ordering::Less, true)?;

      // direction <= 1
      FpVar::enforce_cmp(&ships_vars[ship_index].direction, &constants[1], std::cmp::Ordering::Less, true)?;
    }

    // Check lengths of the ships
    // Require ships sorted by the length
    // 5 of len 1, 4 of 2, 3 of 3, 2 of 4, 1 of 5
    let mut ship_index = 0;
    for ship_size in 1..=5 {
      for _size_count in 0..(6-ship_size) {
        FpVar::enforce_equal(&ships_vars[ship_index].size, &constants[ship_size])?;
        ship_index += 1;
      }
    }

    // Check if every ship is placed within a 10x10 board
    ships_vars.iter().for_each(|ship_vars| {
      let is_vertical = FpVar::is_eq(&ship_vars.direction, &constants[0]).unwrap();

      let new_x = &ship_vars.x + &ship_vars.size;
      let new_y = &ship_vars.y + &ship_vars.size;

      // new_x, new_y < 10
      let is_within_x = FpVar::is_cmp(&new_x, &constants[10], std::cmp::Ordering::Less, true).unwrap();
      let is_within_y = FpVar::is_cmp(&new_y, &constants[10], std::cmp::Ordering::Less, true).unwrap();

      let valid_requirement = Boolean::conditionally_select(&is_vertical, &is_within_y,& is_within_x).unwrap();
      let _ = Boolean::enforce_equal(&valid_requirement, &Boolean::TRUE);
    });

    //TODO
    // Check if ships don't cross each other


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
