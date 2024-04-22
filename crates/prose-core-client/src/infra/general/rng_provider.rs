// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rand::rngs::OsRng;
use rand::{CryptoRng, RngCore};

pub trait RandomNumberGenerator: RngCore + CryptoRng {}

impl RandomNumberGenerator for OsRng {}

pub trait RngProvider: Send + Sync {
    fn rng(&self) -> Box<dyn RandomNumberGenerator>;
}

pub struct OsRngProvider;

impl RngProvider for OsRngProvider {
    fn rng(&self) -> Box<dyn RandomNumberGenerator> {
        Box::new(OsRng)
    }
}

#[cfg(feature = "test")]
pub mod mocks {
    pub use rand::rngs::mock::StepRng;
    use rand::Error;

    use super::*;

    pub struct StepRngProvider {
        rng: StepRng,
    }

    impl StepRngProvider {
        pub fn new(rng: StepRng) -> Self {
            Self { rng }
        }
    }

    impl Default for StepRngProvider {
        fn default() -> Self {
            Self {
                rng: StepRng::new(1, 1),
            }
        }
    }

    impl RngProvider for StepRngProvider {
        fn rng(&self) -> Box<dyn RandomNumberGenerator> {
            Box::new(StepRngWrapper(self.rng.clone()))
        }
    }

    struct StepRngWrapper(StepRng);

    impl RngCore for StepRngWrapper {
        fn next_u32(&mut self) -> u32 {
            self.0.next_u32()
        }

        fn next_u64(&mut self) -> u64 {
            self.0.next_u64()
        }

        fn fill_bytes(&mut self, dest: &mut [u8]) {
            self.0.fill_bytes(dest);
        }

        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
            self.0.try_fill_bytes(dest)
        }
    }

    impl CryptoRng for StepRngWrapper {}
    impl RandomNumberGenerator for StepRngWrapper {}
}
