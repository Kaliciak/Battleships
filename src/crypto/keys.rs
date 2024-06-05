use std::{
    fs::File,
    sync::{Arc, Condvar, Mutex},
    thread,
};

use ark_bls12_381::Config;
use ark_ec::bls12::Bls12;
use ark_groth16::{ProvingKey, VerifyingKey};
use ark_serialize::CanonicalDeserialize;

use crate::utils::{
    log::{Log, Logger},
    result::Res,
};

type Vk = VerifyingKey<Bls12<Config>>;
type Pk = ProvingKey<Bls12<Config>>;

#[derive(Debug, Clone)]
pub struct ArkKeys {
    mut_cond: Arc<(Mutex<Option<Res<Arc<(Vk, Pk)>>>>, Condvar)>,
}

impl ArkKeys {
    pub fn load(logger: Logger) -> Self {
        let keys1 = ArkKeys {
            mut_cond: Arc::new((Mutex::new(None), Condvar::new())),
        };
        let keys2 = keys1.clone();

        thread::spawn(move || {
            let keys = read_keys(logger);
            let mut l = keys2.mut_cond.0.lock().unwrap();
            *l = Some(match keys {
                Ok(ks) => Ok(Arc::new(ks)),
                Err(e) => Err(e),
            });
            keys2.mut_cond.1.notify_all();
        });

        keys1
    }

    pub fn acquire(&mut self) -> Res<Arc<(Vk, Pk)>> {
        let mut mutex = self.mut_cond.0.lock().unwrap();

        while mutex.is_none() {
            mutex = self.mut_cond.1.wait(mutex).unwrap();
        }

        match mutex.as_ref().unwrap() {
            Ok(arc) => Ok(Arc::clone(arc)),
            Err(e) => Err(e.clone()),
        }
    }
}

pub fn read_keys(logger: Logger) -> Res<(Vk, Pk)> {
    let now = std::time::Instant::now();

    let vk_file = File::open("keys/vk_file.key")?;
    let vk = VerifyingKey::deserialize_uncompressed_unchecked(vk_file)?;
    logger.log_message("vk deserialized")?;

    let pk_file = File::open("keys/pk_file.key")?;
    let pk: ProvingKey<ark_ec::bls12::Bls12<ark_bls12_381::Config>> =
        ProvingKey::deserialize_uncompressed_unchecked(pk_file)?;

    let elapsed = now.elapsed();
    logger.log_message(&format!("Keys deserialized. Time: {:.2?}", elapsed))?;

    Ok((vk, pk))
}
