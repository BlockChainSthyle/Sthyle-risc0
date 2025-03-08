use std::collections::{BTreeMap, HashMap, LinkedList};
use borsh::{io::Error, BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use c2pa::Manifest;
use sdk::{Digestable, HyleContract, RunResult};

impl HyleContract for ImageState {
    /// Entry point of the contract's logic
    fn execute(&mut self, contract_input: &sdk::ContractInput) -> RunResult {
        // Parse contract inputs
        let (action, ctx) = sdk::utils::parse_raw_contract_input::<ImageAction>(contract_input)?;

        // Execute the contract logic
        let program_output = match action {
            ImageAction::RegisterImage { image, c2pa_sign } => {
                let manifest = Manifest::from_bytes(&c2pa_sign)
                    .map_err(|e| format!("Failed to parse c2pa manifest: {:?}", e))?;
                // Verify the manifest against the provided image bytes.
                manifest
                    .verify(&image)
                    .map_err(|e| format!("c2pa manifest verification failed: {:?}", e))?;
                let image_hash = hash_image(&image);
                self.images.insert(image_hash.clone(), None);
                format!("Authentic image added with hash: {}", image_hash)
            }
            ImageAction::VerifyImage { image } => {
                let mut current_hash = hash_image(&image);
                // Walk the chain until we find an authenticated original image.
                while let Some(Some(parent_hash)) = self.images.get(&current_hash) {
                    current_hash = parent_hash.clone();
                }
                if self.images.contains_key(&current_hash) {
                    // In a complete solution, an off‑chain lookup can fetch the authentic image data.
                    format!("Authentic image hash is: {}", current_hash)
                } else {
                    return Err("Image not found on-chain".into());
                }
            }
            ImageAction::RegisterEdit { original_image, edited_image } => {
                let original_hash = hash_image(&original_image);
                if self.images.contains_key(&original_hash) {
                    let edited_hash = hash_image(&edited_image);
                    self.images.insert(edited_hash.clone(), Some(original_hash));
                    format!("Edited image added with hash: {}", edited_hash)
                } else {
                    return Err("Original image not found on-chain".into());
                }
            }
        };

        // program_output might be used to give feedback to the user
        let program_output = "new value:".to_string();
        Ok((program_output, ctx, vec![]))
    }
}

/// The action represents the different operations that can be done on the contract
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum ImageAction {
    RegisterImage { image: Vec<u8>, c2pa_sign: Vec<u8>, },
    VerifyImage { image: Vec<u8> },
    RegisterEdit { original_image: Vec<u8>, edited_image: Vec<u8> },
}

fn hash_image(image: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(image);
    hex::encode(hasher.finalize())
}

/// The state of the contract, in this example it is fully serialized on-chain
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
pub struct ImageState {
    pub images: BTreeMap<String, Option<String>>, // Mapping of hashes to hash or None.
}

/// Utils function for the host
impl ImageState {
    pub fn new() -> Self{
        ImageState{
            images: BTreeMap::new(),
        }
    }
    pub fn as_bytes(&self) -> Result<Vec<u8>, Error> {
        borsh::to_vec(self)
    }
}

/// Utils function for the host
impl ImageAction {
    pub fn as_blob(&self, contract_name: &str) -> sdk::Blob {
        sdk::Blob {
            contract_name: contract_name.into(),
            data: sdk::BlobData(borsh::to_vec(self).expect("failed to encode BlobData")),
        }
    }
}

/// Helpers to transform the contrat's state in its on-chain state digest version.
/// In an optimal version, you would here only returns a hash of the state,
/// while storing the full-state off-chain
impl Digestable for ImageState {
    fn as_digest(&self) -> sdk::StateDigest {
        sdk::StateDigest(borsh::to_vec(self).expect("Failed to encode Balances"))
    }
}
impl From<sdk::StateDigest> for ImageState {
    fn from(state: sdk::StateDigest) -> Self {
        borsh::from_slice(&state.0)
            .map_err(|_| "Could not decode hyllar state".to_string())
            .unwrap()
    }
}
