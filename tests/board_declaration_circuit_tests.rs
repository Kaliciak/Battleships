#[cfg(test)]
mod tests {
    use battleships::{
        circuit::board_declaration_circuit::BoardDeclarationCircuit,
        model::{Board, Direction, Ship},
    };

    use ark_bls12_381::{Config, Fr};
    use ark_ec::bls12::Bls12;
    use ark_ff::{One, Zero};
    use ark_groth16::r1cs_to_qap::LibsnarkReduction;
    use ark_groth16::{Groth16, ProvingKey, VerifyingKey};
    use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem};
    use ark_serialize::CanonicalDeserialize;
    use ark_snark::SNARK;
    use ark_std::rand::SeedableRng;
    use ark_std::{iterable::Iterable, rand::rngs::StdRng};
    use sha2::{Digest, Sha256};
    use std::fs::File;

    pub type CircuitField = Fr;

    #[test]
    fn correct_board_with_keys_test() {
        let board = get_correct_board();

        let salt = [1; 32];

        // create a Sha256 object
        let mut hasher = Sha256::new();

        board
            .ships
            .iter()
            .for_each(|ship| hasher.update([ship.x, ship.y, ship.size, ship.direction as u8]));
        hasher.update(salt);

        // read hash digest and consume hasher
        let hash_result = hasher.finalize();

        let real_circuit = BoardDeclarationCircuit {
            board: board,
            salt: salt,
            hash: hash_result.into(),
        };

        let (vk, pk) = read_keys();

        let now = std::time::Instant::now();
        let mut rng: StdRng = StdRng::seed_from_u64(1);
        let proof =
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
        let board = get_correct_board();

        let salt = [1; 32];

        // create a Sha256 object
        let mut hasher = Sha256::new();

        board
            .ships
            .iter()
            .for_each(|ship| hasher.update([ship.x, ship.y, ship.size, ship.direction as u8]));
        hasher.update(salt);

        // read hash digest and consume hasher
        let hash_result = hasher.finalize();

        let real_circuit = BoardDeclarationCircuit {
            board: board,
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
        let mut board = get_correct_board();
        board.ships[12].x = 6;

        let salt = [1; 32];

        // create a Sha256 object
        let mut hasher = Sha256::new();

        board
            .ships
            .iter()
            .for_each(|ship| hasher.update([ship.x, ship.y, ship.size, ship.direction as u8]));
        hasher.update(salt);

        // read hash digest and consume hasher
        let hash_result = hasher.finalize();

        let real_circuit = BoardDeclarationCircuit {
            board: board,
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
        let mut board = get_correct_board();
        board.ships[2].size = 2;

        let salt = [1; 32];

        // create a Sha256 object
        let mut hasher = Sha256::new();

        board
            .ships
            .iter()
            .for_each(|ship| hasher.update([ship.x, ship.y, ship.size, ship.direction as u8]));
        hasher.update(salt);

        // read hash digest and consume hasher
        let hash_result = hasher.finalize();

        let real_circuit = BoardDeclarationCircuit {
            board: board,
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
        let board = get_correct_board();

        let salt = [1; 32];

        // create a Sha256 object
        let mut hasher = Sha256::new();

        board
            .ships
            .iter()
            .for_each(|ship| hasher.update([ship.x, ship.y, ship.size, ship.direction as u8]));
        hasher.update(salt);

        // read hash digest and consume hasher
        let hash_result = hasher.finalize();

        let real_circuit = BoardDeclarationCircuit {
            board: board,
            salt: salt,
            hash: hash_result.into(),
        };

        let (vk, pk) = read_keys();

        let mut rng: StdRng = StdRng::seed_from_u64(1);
        let proof =
            Groth16::<_, LibsnarkReduction>::prove(&pk, real_circuit.clone(), &mut rng).unwrap();
        println!("Proof generated");

        let input = [CircuitField::zero(); 8 * 32];

        let valid_proof = Groth16::<_, LibsnarkReduction>::verify(&vk, &input, &proof).unwrap();
        assert!(!valid_proof);
    }

    fn read_keys() -> (VerifyingKey<Bls12<Config>>, ProvingKey<Bls12<Config>>) {
        let now = std::time::Instant::now();

        let vk_file = File::open("keys/board_declaration/vk.bin").unwrap();
        let vk = VerifyingKey::deserialize_uncompressed_unchecked(vk_file).unwrap();
        println!("vk deserialized");

        let pk_file = File::open("keys/board_declaration/pk.bin").unwrap();
        let pk = ProvingKey::deserialize_uncompressed_unchecked(pk_file).unwrap();

        println!("keys deserialized");
        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);

        (vk, pk)
    }

    fn get_correct_board() -> Board {
        let ships = [
            Ship {
                x: 1,
                y: 1,
                size: 1,
                direction: Direction::Vertical,
            },
            Ship {
                x: 1,
                y: 3,
                size: 1,
                direction: Direction::Vertical,
            },
            Ship {
                x: 1,
                y: 5,
                size: 1,
                direction: Direction::Vertical,
            },
            Ship {
                x: 1,
                y: 7,
                size: 1,
                direction: Direction::Vertical,
            },
            Ship {
                x: 1,
                y: 9,
                size: 1,
                direction: Direction::Vertical,
            },
            Ship {
                x: 3,
                y: 1,
                size: 2,
                direction: Direction::Vertical,
            },
            Ship {
                x: 3,
                y: 4,
                size: 2,
                direction: Direction::Vertical,
            },
            Ship {
                x: 3,
                y: 7,
                size: 2,
                direction: Direction::Vertical,
            },
            Ship {
                x: 3,
                y: 10,
                size: 2,
                direction: Direction::Horizontal,
            },
            Ship {
                x: 5,
                y: 1,
                size: 3,
                direction: Direction::Vertical,
            },
            Ship {
                x: 5,
                y: 5,
                size: 3,
                direction: Direction::Vertical,
            },
            Ship {
                x: 6,
                y: 10,
                size: 3,
                direction: Direction::Horizontal,
            },
            Ship {
                x: 7,
                y: 1,
                size: 4,
                direction: Direction::Vertical,
            },
            Ship {
                x: 9,
                y: 1,
                size: 4,
                direction: Direction::Vertical,
            },
            Ship {
                x: 10,
                y: 6,
                size: 5,
                direction: Direction::Vertical,
            },
        ];
        Board { ships: ships }
    }
}
