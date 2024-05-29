fn main() {
    example_proof();
    println!("Hello, world!");
}

// TODO przenieść do testów
use ark_groth16::Groth16;
use ark_groth16::r1cs_to_qap::LibsnarkReduction;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem};
use ark_snark::{CircuitSpecificSetupSNARK, SNARK};
use ark_std::{iterable::Iterable, rand::rngs::StdRng};
use ark_std::rand::SeedableRng;
use ark_bls12_381::Fr;
use ark_ff::{One, Zero};

pub type Curve = ark_bls12_381::Bls12_381;
pub type CircuitField = Fr;

pub mod model;
pub mod board_circuit;
pub use model::{Board, Direction, Ship};
pub use board_circuit::BoardCircuit;

use sha2::{Sha256, Digest};

fn example_proof() {
    let mut rng = StdRng::seed_from_u64(1);
    // let empty_circuit = BoardCircuit {
    //     board: None,
    //     salt: None,
    //     hash: None,
    // };
    
    let mut ships = [Ship {
        x: 2,
        y: 2,
        size: 2,
        direction: Direction::VERTICAL,
    }; 15];
    ships[7] = Ship {
        x: 2,
        y: 2,
        size: 2,
        direction: Direction::VERTICAL,
    };

    let salt = [1;32];
    // create a Sha256 object
    let mut hasher = Sha256::new();

    ships.iter().for_each(|ship| hasher.update([
        ship.x,
        ship.y,
        ship.size,
        ship.direction as u8,
    ]));
    hasher.update(salt);

    // read hash digest and consume hasher
    let hash_result = hasher.finalize();

    let real_circuit = BoardCircuit {
        board: Board {
            ships: ships
        },
        salt: salt,
        hash: hash_result.into(),
    };

    let now = std::time::Instant::now();

    let (pk, vk) = Groth16::<Curve>::setup(real_circuit.clone(), &mut rng).unwrap();

    println!("keys generated");
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    let now2 = std::time::Instant::now();
    let proof: ark_groth16::Proof<ark_ec::bls12::Bls12<ark_bls12_381::Config>> = Groth16::<_, LibsnarkReduction>::prove(&pk, real_circuit.clone(), &mut rng).unwrap();
    println!("proof generated");
    let elapsed2 = now2.elapsed();
    println!("Elapsed: {:.2?}", elapsed2);

    let mut input = [CircuitField::zero(); 8*32];
    for i in 0..32 {
        for j in 0..8 {
            if real_circuit.hash[i] >> j & 1 == 1 {
                input[i*8 + j] = CircuitField::one();
            }
        }
    }

    let now3 = std::time::Instant::now();
    let valid_proof = Groth16::<_, LibsnarkReduction>::verify(&vk, &input, &proof).unwrap();
    println!("proof verified");
    let elapsed3= now3.elapsed();
    println!("Elapsed: {:.2?}", elapsed3);
    println!("{valid_proof}")
}