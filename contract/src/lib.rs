use borsh::{io::Error, BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use sdk::{Digestable, HyleContract, RunResult};

/// Define contract actions for image authentication
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum ImageAction {
    RegisterImage { hash: Vec<u8>, owner: String },
    VerifyImage { hash: Vec<u8> },
    RegisterEdit { original_hash: Vec<u8>, edited_hash: Vec<u8>, owner: String },
}

/// The state of the contract, storing registered images and edits
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
pub struct ImageRegistry {
    pub images: Vec<(Vec<u8>, String)>, // (image_hash, owner)
    pub edits: Vec<(Vec<u8>, Vec<u8>, String)>, // (original_hash, edited_hash, owner)
}

impl ImageRegistry {
    pub fn default() -> Self {
        Self {
            images: Vec::new(),
            edits: Vec::new(),
        }
    }

    pub fn as_bytes(&self) -> Result<Vec<u8>, Error> {
        borsh::to_vec(self)
    }
}

impl HyleContract for ImageRegistry {
    fn execute(&mut self, contract_input: &sdk::ContractInput) -> RunResult {
        let (action, ctx) = sdk::utils::parse_raw_contract_input::<ImageAction>(contract_input)?;

        let program_output = match action {
            ImageAction::RegisterImage { hash, owner } => {
                if !self.images.iter().any(|(h, _)| *h == hash) {
                    self.images.push((hash.clone(), owner.clone()));
                    format!("Image registered by {}", owner)
                } else {
                    "Image already registered".to_string()
                }
            }
            ImageAction::VerifyImage { hash } => {
                if let Some((_, owner)) = self.images.iter().find(|(h, _)| *h == hash) {
                    format!("Image is authentic. Owner: {}", owner)
                } else {
                    "Image not found".to_string()
                }
            }
            ImageAction::RegisterEdit { original_hash, edited_hash, owner } => {
                if self.images.iter().any(|(h, _)| *h == original_hash) {
                    self.edits.push((original_hash.clone(), edited_hash.clone(), owner.clone()));
                    format!("Edit registered by {}", owner)
                } else {
                    "Original image not found".to_string()
                }
            }
        };

        Ok((program_output, ctx, vec![]))
    }
}

impl Digestable for ImageRegistry {
    fn as_digest(&self) -> sdk::StateDigest {
        sdk::StateDigest(borsh::to_vec(self).expect("Failed to encode ImageRegistry"))
    }
}

impl From<sdk::StateDigest> for ImageRegistry {
    fn from(state: sdk::StateDigest) -> Self {
        borsh::from_slice(&state.0)
            .map_err(|_| "Could not decode hyllar state".to_string())
            .unwrap()
    }
}
