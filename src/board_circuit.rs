use ark_bls12_381::Fr;
use ark_r1cs_std::alloc::AllocVar;
use ark_r1cs_std::boolean::Boolean;
use ark_r1cs_std::eq::EqGadget;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::fields::FieldVar;
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
	pub size_numerical: Option<usize>,
	pub direction: FpVar<CircuitField>,
    pub is_vertical: Boolean<CircuitField>,
}

const SHIPS_SIZES: [usize; 15] = [
	1, 1, 1, 1, 1,
	2, 2, 2, 2,
	3, 3, 3,
	4, 4,
	5,
];

impl ark_relations::r1cs::ConstraintSynthesizer<CircuitField> for BoardCircuit {
  fn generate_constraints(self, cs: ConstraintSystemRef<CircuitField>) -> Result<()> {
	// Generate needed constants
    // constans[i] -- constant representing i
    let constants: Vec<FpVar<CircuitField>> = (0..=5).map(|number| FpVar::new_constant(ns!(cs, "constant"), CircuitField::from(number)).unwrap()).collect();
	let ten = FpVar::new_constant(ns!(cs, "10"), CircuitField::from(10))?;

    // Create private variables for each ship
    let mut ships_vars: [ShipVars; 15] = self.board.ships.map(|ship| create_ship_vars(&ship, &cs).unwrap());
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

    // Check if all values are from the correct range
    for ship_index in 0..15 {
		// 1 <= x, y <= 10
		FpVar::enforce_cmp(&ships_vars[ship_index].x, &FpVar::one(), std::cmp::Ordering::Greater, true)?;
		FpVar::enforce_cmp(&ships_vars[ship_index].x, &ten, std::cmp::Ordering::Less, true)?;
		FpVar::enforce_cmp(&ships_vars[ship_index].y, &FpVar::one(), std::cmp::Ordering::Greater, true)?;
		FpVar::enforce_cmp(&ships_vars[ship_index].y, &ten, std::cmp::Ordering::Less, true)?;
    }

    // Check lengths of the ships
    // Require ships sorted by the length
    // 5 of len 1, 4 of 2, 3 of 3, 2 of 4, 1 of 5
	SHIPS_SIZES.iter().zip(0..15).for_each(|(ship_size, ship_index) | {
		for _size_count in 0..(6-ship_size) {
			FpVar::enforce_equal(&ships_vars[ship_index].size, &constants[*ship_size]).unwrap();
			// Set size_numerical
			ships_vars[ship_index].size_numerical = Some(*ship_size);
		}
	});

    // Check if every ship is placed within a 10x10 board
    ships_vars.iter().for_each(|ship_vars| {
		let is_vertical = FpVar::is_eq(&ship_vars.direction, &FpVar::zero()).unwrap();

		let new_x = &ship_vars.x + &ship_vars.size - &FpVar::one();
		let new_y = &ship_vars.y + &ship_vars.size - &FpVar::one();

		// new_x, new_y <= 10
		let is_within_x = FpVar::is_cmp(&new_x, &ten, std::cmp::Ordering::Less, true).unwrap();
		let is_within_y = FpVar::is_cmp(&new_y, &ten, std::cmp::Ordering::Less, true).unwrap();

		let valid_requirement = Boolean::conditionally_select(&is_vertical, &is_within_y,& is_within_x).unwrap();
		let _ = Boolean::enforce_equal(&valid_requirement, &Boolean::TRUE);
    });

    // Check if ships don't touch each other
    for i in 0..14 {
		for j in (i+1)..15 {
			enforce_ships_not_touching(&ships_vars[i], &ships_vars[j])?;
		}
    }
	
    Ok(())
  }
}

fn create_ship_vars(ship: &Ship, cs: &ConstraintSystemRef<CircuitField>) -> Result<ShipVars> {
	let direction = FpVar::new_witness(ns!(cs, "shipDirection"), || Ok(CircuitField::from(ship.direction as u8)))?;
	// direction <= 1
	FpVar::enforce_cmp(&direction, &FpVar::one(), std::cmp::Ordering::Less, true)?;

	let is_vertical = FpVar::is_eq(&direction, &FpVar::zero()).unwrap();

	Ok(ShipVars {
		x: FpVar::new_witness(ns!(cs, "shipX"), || Ok(CircuitField::from(ship.x)))?,
		y: FpVar::new_witness(ns!(cs, "shipY"), || Ok(CircuitField::from(ship.y)))?,
		size: FpVar::new_witness(ns!(cs, "shipSize"), || Ok(CircuitField::from(ship.size)))?,
		// To be set later
		size_numerical: None,
		direction: direction,
		is_vertical: is_vertical,
	})
}

// Need to also check if the var is less than 8 bytes long
fn cast_fp_var_to_uint8(var: &FpVar<CircuitField>) -> Result<UInt8<CircuitField>> {
	let bytes = FpVar::to_bytes(&var)?;
	// to_bytes function returns [var, 0, 0, 0, ...] -- a vector of UInt8 of len 32 (in case of not too large var)
	Ok(bytes[0].clone())
}

fn enforce_ships_not_touching(ship1: &ShipVars, ship2: &ShipVars) -> Result<()> {
	// Ship1 needs to be either above, below, left or right the forbidden zone of ship2

	//-------------------
	// Compute for ship1

	// Compute right coordinate of ship1
	// If vertical then rect_ru_x = ship1.x
	// If horizontal then rect_ru_x = ship1.x + ship1.size - 1
	let ship1_right_x = FpVar::conditionally_select(&ship1.is_vertical, &ship1.x, &(&ship1.x  + &ship1.size - &FpVar::one()))?;

	// Compute lower coordinate of ship1
	// If vertical then rect_ll_y = ship1.y + ship1.size - 1
	// If horizontal then rect_ll_y = ship1.y
	let ship1_lower_y = FpVar::conditionally_select(&ship1.is_vertical, &(&ship1.y + &ship1.size - &FpVar::one()), &ship1.y)?;

	//--------------------------------
	// Compute forbidden zone of ship2

	// Compute left upper corner of forbidden rectangle
	// rect_lu_x = ship.x - 1
	// rect_lu_y = ship.y - 1
	let rect_lu_x = &ship2.x - FpVar::one();
	let rect_lu_y = &ship2.y - FpVar::one();

	// Compute right upper corner x
	// If vertical then rect_ru_x = ship.x + 1
	// If horizontal then rect_ru_x = ship.x + ship.size
	let rect_ru_x = FpVar::conditionally_select(&ship2.is_vertical, &(&ship2.x + &FpVar::one()), &(&ship2.x  + &ship2.size))?;
	
	// Compute left lower corner y
	// If vertical then rect_ll_y = ship.y + ship.size
	// If horizontal then rect_ll_y = ship.y + 1
	let rect_ll_y = FpVar::conditionally_select(&ship2.is_vertical, &(&ship2.y + &ship2.size), &(&ship2.y + &FpVar::one()))?;

	// ------------
	// Touch check

	// Check postition of ship1 relative to zone
	let is_above = FpVar::is_cmp(&ship1_lower_y, &rect_lu_y, std::cmp::Ordering::Less, false)?;
	let is_below = FpVar::is_cmp(&ship1.y, &rect_ll_y, std::cmp::Ordering::Greater, false)?;
	let is_left = FpVar::is_cmp(&ship1_right_x, &rect_lu_x, std::cmp::Ordering::Less, false)?;
	let is_right = FpVar::is_cmp(&ship1.x, &rect_ru_x, std::cmp::Ordering::Greater, false)?;

	// Ship1 must be in either of one of these positions
	let vertical_condition = Boolean::or(&is_above, &is_below)?;
	let horizontal_condition = Boolean::or(&is_left, &is_right)?;
	let result_condition = Boolean::or(&vertical_condition, &horizontal_condition)?;
	Boolean::enforce_equal(&result_condition, &Boolean::TRUE)
}
