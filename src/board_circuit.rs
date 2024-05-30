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
    let constants: Vec<FpVar<CircuitField>> = (0..=11).map(|number| FpVar::new_constant(ns!(cs, "constant"), CircuitField::from(number)).unwrap()).collect();

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
		FpVar::enforce_cmp(&ships_vars[ship_index].x, &constants[1], std::cmp::Ordering::Greater, true)?;
		FpVar::enforce_cmp(&ships_vars[ship_index].x, &constants[10], std::cmp::Ordering::Less, true)?;
		FpVar::enforce_cmp(&ships_vars[ship_index].y, &constants[1], std::cmp::Ordering::Greater, true)?;
		FpVar::enforce_cmp(&ships_vars[ship_index].y, &constants[10], std::cmp::Ordering::Less, true)?;
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
		let is_vertical = FpVar::is_eq(&ship_vars.direction, &constants[0]).unwrap();

		let new_x = &ship_vars.x + &ship_vars.size;
		let new_y = &ship_vars.y + &ship_vars.size;

		// new_x, new_y < 10
		let is_within_x = FpVar::is_cmp(&new_x, &constants[10], std::cmp::Ordering::Less, true).unwrap();
		let is_within_y = FpVar::is_cmp(&new_y, &constants[10], std::cmp::Ordering::Less, true).unwrap();

		let valid_requirement = Boolean::conditionally_select(&is_vertical, &is_within_y,& is_within_x).unwrap();
		let _ = Boolean::enforce_equal(&valid_requirement, &Boolean::TRUE);
    });

    // Check if ships don't touch each other
    for i in 0..14 {
		for j in (i+1)..15 {
			enforce_ships_not_touching(&ships_vars[i], &ships_vars[j], &constants)?;
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

fn enforce_ships_not_touching(ship1: &ShipVars, ship2: &ShipVars, constants: &[FpVar<CircuitField>]) -> Result<()> {
	// For every field of ship1, check if it is in the forbidden area of ship2

	// First field is always the same, regardless of direction
	enforce_field_not_touching_ship(&ship1.x, &ship1.y, ship2)?;

	// Other fields
	for field_index in 1..ship1.size_numerical.unwrap() {
		// If vertical then field_x = x;
		// If horizontal then field_x = x + field_index
		let field_x = FpVar::conditionally_select(&ship1.is_vertical, &ship1.x, &(&ship1.x + &constants[field_index]))?; 
		// If vertical then field_y = y + field_index;
		// If horizontal then field_y = y
		let field_y = FpVar::conditionally_select(&ship1.is_vertical, &(&ship1.y + &constants[field_index]), &ship1.y)?; 

		enforce_field_not_touching_ship(&field_x, &field_y, ship2)?;
	}

	Ok(())
}

fn enforce_field_not_touching_ship(field_x: &FpVar<CircuitField>, field_y: &FpVar<CircuitField>, ship: &ShipVars) -> Result<()> {
	// Compute the corners of the forbidden rectangle

	// Compute left upper corner of forbidden rectangle
	// rect_lu_x = ship.x - 1
	// rect_lu_y = ship.y - 1
	let rect_lu_x = &ship.x - FpVar::one();
	let rect_lu_y = &ship.y - FpVar::one();

	// Compute right upper corner x
	// If vertical then rect_ru_x = ship.x + 1
	// If horizontal then rect_ru_x = ship.x + 1 + ship.size
	let rect_ru_x = FpVar::conditionally_select(&ship.is_vertical, &(&ship.x + &FpVar::one()), &(&ship.x  + &FpVar::one() + &ship.size))?;
	
	// Compute left lower corner y
	// If vertical then rect_ll_y = ship.y + 1 + ship.size
	// If horizontal then rect_ll_y = ship.y + 1
	let rect_ll_y = FpVar::conditionally_select(&ship.is_vertical, &(&ship.y + &FpVar::one() + &ship.size), &(&ship.y + &FpVar::one()))?;

	// field_x < rect_lu_x || rect_ru_x < field_x || /* if collision with x then check y*/ (field_y < rect_lu_y || rect_ll_y < field_y)
	let fx_lt_lux = FpVar::is_cmp(&field_x, &rect_lu_x, std::cmp::Ordering::Less, false)?;
	let fx_gt_rux = FpVar::is_cmp(&field_x, &rect_ru_x, std::cmp::Ordering::Greater, false)?;
	let fy_lt_luy = FpVar::is_cmp(&field_y, &rect_lu_y, std::cmp::Ordering::Less, false)?;
	let fy_gt_lly = FpVar::is_cmp(&field_y, &rect_ll_y, std::cmp::Ordering::Greater, false)?;

	let x_cond = Boolean::or(&fx_lt_lux, &fx_gt_rux)?;
	let y_cond = Boolean::or(&fy_lt_luy, &fy_gt_lly)?;
	let total_cond = Boolean::or(&x_cond, &y_cond)?;
	Boolean::enforce_equal(&total_cond, &Boolean::TRUE)?;

	Ok(())
}

// Statek albo jest cały nad albo cały pod, albo cały na prawo, albo cały na lewo I tyle