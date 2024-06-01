use ark_bls12_381::Fr;
use ark_groth16::Groth16;
use ark_r1cs_std::alloc::AllocVar;
use ark_r1cs_std::boolean::Boolean;
use ark_r1cs_std::eq::EqGadget;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::fields::FieldVar;
use ark_r1cs_std::select::CondSelectGadget;
use ark_r1cs_std::uint8::UInt8;
use ark_relations::ns;
use ark_relations::r1cs::{ConstraintSystemRef, Result};
use ark_serialize::CanonicalSerialize;
use ark_snark::CircuitSpecificSetupSNARK;
use ark_std::rand::SeedableRng;
use ark_std::{iterable::Iterable, rand::rngs::StdRng};
use std::cmp::Ordering;
use std::fs::File;

pub type Curve = ark_bls12_381::Bls12_381;
pub type CircuitField = Fr;

use crate::circuit::commons::ShipVars;
use crate::model::{Board, Direction, FieldState, Ship};

use super::commons::SHIPS_SIZES;
use super::commons::{compute_hash, create_ship_vars};

#[derive(Copy, Clone, Debug)]
pub struct FieldDeclarationCircuit {
    pub board: Board,
    pub salt: [u8; 32],
    pub hash: [u8; 32],
    pub field_x: u8,
    pub field_y: u8,
    pub field_state: FieldState,
}

impl ark_relations::r1cs::ConstraintSynthesizer<CircuitField> for FieldDeclarationCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<CircuitField>) -> Result<()> {
        // Generate needed constant
        let ten = FpVar::new_constant(ns!(cs, "10"), CircuitField::from(10))?;

        // Create private variables for each ship
        let mut ships_vars: [ShipVars; 15] = self
            .board
            .ships
            .map(|ship| create_ship_vars(&ship, &cs).unwrap());
        // Create private variable for hash salt
        let salt_vars: [UInt8<CircuitField>; 32] = self
            .salt
            .map(|bit| UInt8::new_witness(ns!(cs, "salt"), || Ok(bit)).unwrap());
        // Create input for hash of ships
        let hash_vars: [UInt8<CircuitField>; 32] = self
            .hash
            .map(|bit| UInt8::new_input(ns!(cs, "hash"), || Ok(bit)).unwrap());

        // Create input for field coordinates
        let field_x_var =
            FpVar::new_input(ns!(cs, "field_x"), || Ok(CircuitField::from(self.field_x)))?;
        let field_y_var =
            FpVar::new_input(ns!(cs, "field_y"), || Ok(CircuitField::from(self.field_y)))?;

        // Create input for the field state
        let field_state_var = FpVar::new_input(ns!(cs, "field_state"), || {
            Ok(CircuitField::from(self.field_state as u8))
        })?;

        //--------------------------
        // Check if hash is correct

        // Compute the hash
        let digest_var = compute_hash(&ships_vars, &salt_vars)?;

        // Compare the hashes
        hash_vars
            .iter()
            .zip(digest_var.0)
            .for_each(|(h1, h2)| h1.enforce_equal(&h2).unwrap());

        //------------------------------------------------------
        // Check if the field values are from the correct range
        // 1 <= field_x, field_y <= 10
        FpVar::enforce_cmp(&field_x_var, &FpVar::one(), Ordering::Greater, true)?;
        FpVar::enforce_cmp(&field_x_var, &ten, Ordering::Less, true)?;
        FpVar::enforce_cmp(&field_y_var, &FpVar::one(), Ordering::Greater, true)?;
        FpVar::enforce_cmp(&field_y_var, &ten, Ordering::Less, true)?;

        // field_state <= 1
        FpVar::enforce_cmp(&field_state_var, &FpVar::one(), Ordering::Less, true)?;

        let is_field_declaration_empty = FpVar::is_eq(&field_state_var, &FpVar::zero()).unwrap();

        //-------------------------------
        // Check if the field state is correct
        let mut is_field_occupied: Boolean<CircuitField> = Boolean::FALSE;

        // For every ship check if it occupies given field
        ships_vars.iter().for_each(|ship_vars| {
            is_field_occupied = Boolean::or(
                &is_field_occupied,
                &is_ship_occupying_field(ship_vars, &field_x_var, &field_y_var).unwrap(),
            )
            .unwrap();
        });

        Boolean::enforce_equal(&is_field_declaration_empty, &is_field_occupied.not())?;

        Ok(())
    }
}

fn is_ship_occupying_field(
    ship_vars: &ShipVars,
    field_x_var: &FpVar<CircuitField>,
    field_y_var: &FpVar<CircuitField>,
) -> Result<Boolean<CircuitField>> {
    // ship.x <= field_x <= ship.right_x && ship.y <= field_y <= ship.lower_y
    // If vertical then ship.right_x = ship.x, ship.lower_y = ship.y + ship.size - 1;
    // If horizontal then ship.right_x = ship.x + ship.size - 1, ship.lower_y = ship.y,

    let ship_right_x = FpVar::conditionally_select(
        &ship_vars.is_vertical,
        &ship_vars.x,
        &(&ship_vars.x + &ship_vars.size - &FpVar::one()),
    )?;

    let ship_lower_y = FpVar::conditionally_select(
        &ship_vars.is_vertical,
        &(&ship_vars.y + &ship_vars.size - &FpVar::one()),
        &ship_vars.y,
    )?;

    // ship.x <= field_x <= ship.right_x
    let ship_x_le_field_x = FpVar::is_cmp(&ship_vars.x, &field_x_var, Ordering::Less, true)?;
    let right_x_ge_field_x = FpVar::is_cmp(&ship_right_x, &field_x_var, Ordering::Greater, true)?;
    let x_condition = Boolean::and(&ship_x_le_field_x, &right_x_ge_field_x)?;

    // ship.y <= field_y <= ship.lower_y
    let ship_y_le_field_y = FpVar::is_cmp(&ship_vars.y, &field_y_var, Ordering::Less, true)?;
    let lower_y_ge_field_y = FpVar::is_cmp(&ship_lower_y, &field_y_var, Ordering::Greater, true)?;
    let y_condition = Boolean::and(&ship_y_le_field_y, &lower_y_ge_field_y)?;

    // If the ship occupies given field
    Boolean::and(&x_condition, &y_condition)
}

pub fn generate_keys() {
    let mut rng = StdRng::seed_from_u64(1);

    let mut ships = [Ship {
        x: 1,
        y: 1,
        size: 1,
        direction: Direction::Vertical,
    }; 15];

    let dummy_circuit = FieldDeclarationCircuit {
        board: Board { ships },
        salt: [0; 32],
        hash: [0; 32],
        field_x: 1,
        field_y: 1,
        field_state: FieldState::Empty,
    };

    let now = std::time::Instant::now();

    let (pk, vk) = Groth16::<Curve>::setup(dummy_circuit, &mut rng).unwrap();

    println!("Keys generated");
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    let vk_file = File::create("keys/field_declaration/vk.bin").unwrap();
    vk.serialize_uncompressed(vk_file).unwrap();

    let pk_file = File::create("keys/field_declaration/pk.bin").unwrap();
    pk.serialize_uncompressed(pk_file).unwrap();
}
