#[cfg(test)]
mod tests {
    use battleships::{
        board_circuit::BoardCircuit,
        crypto::keys::read_keys,
        model::{Board, Direction, Ship},
        utils::log::get_print_logger,
    };

    use ark_bls12_381::Fr;

    use ark_ff::{One, Zero};
    use ark_groth16::r1cs_to_qap::LibsnarkReduction;
    use ark_groth16::Groth16;
    use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem};
    use ark_snark::SNARK;
    use ark_std::rand::SeedableRng;
    use ark_std::{iterable::Iterable, rand::rngs::StdRng};
    use sha2::{Digest, Sha256};

    pub type CircuitField = Fr;

    #[test]
    fn correct_board_with_keys_test() {
        let ships = [
            Ship {
                x: 1,
                y: 1,
                size: 1,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 1,
                y: 3,
                size: 1,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 1,
                y: 5,
                size: 1,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 1,
                y: 7,
                size: 1,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 1,
                y: 9,
                size: 1,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 3,
                y: 1,
                size: 2,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 3,
                y: 4,
                size: 2,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 3,
                y: 7,
                size: 2,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 3,
                y: 10,
                size: 2,
                direction: Direction::HORIZONTAL,
            },
            Ship {
                x: 5,
                y: 1,
                size: 3,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 5,
                y: 5,
                size: 3,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 6,
                y: 10,
                size: 3,
                direction: Direction::HORIZONTAL,
            },
            Ship {
                x: 7,
                y: 1,
                size: 4,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 9,
                y: 1,
                size: 4,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 10,
                y: 6,
                size: 5,
                direction: Direction::VERTICAL,
            },
        ];

        let salt = [1; 32];

        // create a Sha256 object
        let mut hasher = Sha256::new();

        ships
            .iter()
            .for_each(|ship| hasher.update([ship.x, ship.y, ship.size, ship.direction as u8]));
        hasher.update(salt);

        // read hash digest and consume hasher
        let hash_result = hasher.finalize();

        let real_circuit = BoardCircuit {
            board: Board { ships: ships },
            salt: salt,
            hash: hash_result.into(),
        };

        let (vk, pk) = read_keys(get_print_logger());

        let now = std::time::Instant::now();
        let mut rng: StdRng = StdRng::seed_from_u64(1);
        let proof: ark_groth16::Proof<ark_ec::bls12::Bls12<ark_bls12_381::Config>> =
            Groth16::<_, LibsnarkReduction>::prove(&pk, real_circuit.clone(), &mut rng).unwrap();
        println!("Proof generated");
        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);

        let mut input = [CircuitField::zero(); 8 * 32];
        for i in 0..32 {
            for j in 0..8 {
                if real_circuit.hash[i] >> j & 1 == 1 {
                    input[i * 8 + j] = CircuitField::one();
                }
            }
        }

        let valid_proof = Groth16::<_, LibsnarkReduction>::verify(&vk, &input, &proof).unwrap();
        println!("Proof verified");
        println!("{valid_proof}");
        assert!(valid_proof);
    }

    #[test]
    fn correct_board_test() {
        let ships = [
            Ship {
                x: 1,
                y: 1,
                size: 1,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 1,
                y: 3,
                size: 1,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 1,
                y: 5,
                size: 1,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 1,
                y: 7,
                size: 1,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 1,
                y: 9,
                size: 1,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 3,
                y: 1,
                size: 2,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 3,
                y: 4,
                size: 2,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 3,
                y: 7,
                size: 2,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 3,
                y: 10,
                size: 2,
                direction: Direction::HORIZONTAL,
            },
            Ship {
                x: 5,
                y: 1,
                size: 3,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 5,
                y: 5,
                size: 3,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 6,
                y: 10,
                size: 3,
                direction: Direction::HORIZONTAL,
            },
            Ship {
                x: 7,
                y: 1,
                size: 4,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 9,
                y: 1,
                size: 4,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 10,
                y: 6,
                size: 5,
                direction: Direction::VERTICAL,
            },
        ];

        let salt = [1; 32];

        // create a Sha256 object
        let mut hasher = Sha256::new();

        ships
            .iter()
            .for_each(|ship| hasher.update([ship.x, ship.y, ship.size, ship.direction as u8]));
        hasher.update(salt);

        // read hash digest and consume hasher
        let hash_result = hasher.finalize();

        let real_circuit = BoardCircuit {
            board: Board { ships: ships },
            salt: salt,
            hash: hash_result.into(),
        };

        let cs = ConstraintSystem::new_ref();
        real_circuit
            .clone()
            .generate_constraints(cs.clone())
            .unwrap();
        assert!(cs.is_satisfied().unwrap());
    }

    #[test]
    fn touching_ships_test() {
        let ships = [
            Ship {
                x: 1,
                y: 1,
                size: 1,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 1,
                y: 3,
                size: 1,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 1,
                y: 5,
                size: 1,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 1,
                y: 7,
                size: 1,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 1,
                y: 9,
                size: 1,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 3,
                y: 1,
                size: 2,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 3,
                y: 4,
                size: 2,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 3,
                y: 7,
                size: 2,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 3,
                y: 10,
                size: 2,
                direction: Direction::HORIZONTAL,
            },
            Ship {
                x: 5,
                y: 1,
                size: 3,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 5,
                y: 5,
                size: 3,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 6,
                y: 10,
                size: 3,
                direction: Direction::HORIZONTAL,
            },
            Ship {
                x: 6,
                y: 1,
                size: 4,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 9,
                y: 1,
                size: 4,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 10,
                y: 6,
                size: 5,
                direction: Direction::VERTICAL,
            },
        ];

        let salt = [1; 32];

        // create a Sha256 object
        let mut hasher = Sha256::new();

        ships
            .iter()
            .for_each(|ship| hasher.update([ship.x, ship.y, ship.size, ship.direction as u8]));
        hasher.update(salt);

        // read hash digest and consume hasher
        let hash_result = hasher.finalize();

        let real_circuit = BoardCircuit {
            board: Board { ships: ships },
            salt: salt,
            hash: hash_result.into(),
        };

        let cs = ConstraintSystem::new_ref();
        real_circuit
            .clone()
            .generate_constraints(cs.clone())
            .unwrap();
        assert!(!cs.is_satisfied().unwrap());
    }

    #[test]
    fn bad_ship_size_test() {
        let ships = [
            Ship {
                x: 1,
                y: 1,
                size: 1,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 1,
                y: 3,
                size: 1,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 1,
                y: 5,
                size: 2,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 1,
                y: 7,
                size: 1,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 1,
                y: 9,
                size: 1,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 3,
                y: 1,
                size: 2,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 3,
                y: 4,
                size: 2,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 3,
                y: 7,
                size: 2,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 3,
                y: 10,
                size: 2,
                direction: Direction::HORIZONTAL,
            },
            Ship {
                x: 5,
                y: 1,
                size: 3,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 5,
                y: 5,
                size: 3,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 6,
                y: 10,
                size: 3,
                direction: Direction::HORIZONTAL,
            },
            Ship {
                x: 6,
                y: 1,
                size: 4,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 9,
                y: 1,
                size: 4,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 10,
                y: 6,
                size: 5,
                direction: Direction::VERTICAL,
            },
        ];

        let salt = [1; 32];

        // create a Sha256 object
        let mut hasher = Sha256::new();

        ships
            .iter()
            .for_each(|ship| hasher.update([ship.x, ship.y, ship.size, ship.direction as u8]));
        hasher.update(salt);

        // read hash digest and consume hasher
        let hash_result = hasher.finalize();

        let real_circuit = BoardCircuit {
            board: Board { ships: ships },
            salt: salt,
            hash: hash_result.into(),
        };

        let cs = ConstraintSystem::new_ref();
        real_circuit
            .clone()
            .generate_constraints(cs.clone())
            .unwrap();
        assert!(!cs.is_satisfied().unwrap());
    }

    #[test]
    fn incorrect_input_test() {
        let ships = [
            Ship {
                x: 1,
                y: 1,
                size: 1,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 1,
                y: 3,
                size: 1,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 1,
                y: 5,
                size: 1,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 1,
                y: 7,
                size: 1,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 1,
                y: 9,
                size: 1,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 3,
                y: 1,
                size: 2,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 3,
                y: 4,
                size: 2,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 3,
                y: 7,
                size: 2,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 3,
                y: 10,
                size: 2,
                direction: Direction::HORIZONTAL,
            },
            Ship {
                x: 5,
                y: 1,
                size: 3,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 5,
                y: 5,
                size: 3,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 6,
                y: 10,
                size: 3,
                direction: Direction::HORIZONTAL,
            },
            Ship {
                x: 7,
                y: 1,
                size: 4,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 9,
                y: 1,
                size: 4,
                direction: Direction::VERTICAL,
            },
            Ship {
                x: 10,
                y: 6,
                size: 5,
                direction: Direction::VERTICAL,
            },
        ];

        let salt = [1; 32];

        // create a Sha256 object
        let mut hasher = Sha256::new();

        ships
            .iter()
            .for_each(|ship| hasher.update([ship.x, ship.y, ship.size, ship.direction as u8]));
        hasher.update(salt);

        // read hash digest and consume hasher
        let hash_result = hasher.finalize();

        let real_circuit = BoardCircuit {
            board: Board { ships: ships },
            salt: salt,
            hash: hash_result.into(),
        };

        let (vk, pk) = read_keys(get_print_logger());

        let mut rng: StdRng = StdRng::seed_from_u64(1);
        let proof: ark_groth16::Proof<ark_ec::bls12::Bls12<ark_bls12_381::Config>> =
            Groth16::<_, LibsnarkReduction>::prove(&pk, real_circuit.clone(), &mut rng).unwrap();
        println!("Proof generated");

        let input = [CircuitField::zero(); 8 * 32];

        let valid_proof = Groth16::<_, LibsnarkReduction>::verify(&vk, &input, &proof).unwrap();
        assert!(!valid_proof);
    }
}
