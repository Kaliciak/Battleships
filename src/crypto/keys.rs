use std::fs::File;

use ark_bls12_381::Config;
use ark_ec::bls12::Bls12;
use ark_groth16::{ProvingKey, VerifyingKey};
use ark_serialize::CanonicalDeserialize;

use crate::utils::log::{Log, Logger};

pub fn read_keys(logger: Logger) -> (VerifyingKey<Bls12<Config>>, ProvingKey<Bls12<Config>>) {
    let now = std::time::Instant::now();

    let vk_file = File::open("keys/vk_file.key").unwrap();
    let vk = VerifyingKey::deserialize_uncompressed_unchecked(vk_file).unwrap();
    logger.log_message("vk deserialized");

    let pk_file = File::open("keys/pk_file.key").unwrap();
    let pk: ProvingKey<ark_ec::bls12::Bls12<ark_bls12_381::Config>> =
        ProvingKey::deserialize_uncompressed_unchecked(pk_file).unwrap();

    let elapsed = now.elapsed();
    logger.log_message(&format!("Keys deserialized. Time: {:.2?}", elapsed));

    (vk, pk)
}
