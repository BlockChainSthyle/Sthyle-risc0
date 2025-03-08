use std::collections::{HashMap, HashSet};
use borsh::{io::Error, BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use sdk::{Digestable, HyleContract, RunResult};

/// Struct to store image metadata
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
pub struct ImageMetadata {
    pub previous_image_hash: Option<String>,
    pub owner_pk: String,
    pub publishers: HashSet<String>,
    pub is_root: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
pub struct ImageState {
    pub hash_map: HashMap<String, ImageMetadata>,
}

impl HyleContract for ImageState {
    fn execute(&mut self, contract_input: &sdk::ContractInput) -> RunResult {
        let (action, ctx) = sdk::utils::parse_raw_contract_input::<ImageAction>(contract_input)?;

        let program_output = match action {
            ImageAction::RegisterImage { image_hash, image_signature: _, owner_pk } => {
                println!("Trying register");
                println!("Existing keys: {:?}", self.hash_map.keys());

                if !self.hash_map.contains_key(&image_hash) {
                    self.hash_map.insert(
                        image_hash.clone(),
                        ImageMetadata {
                            previous_image_hash: None,
                            owner_pk,
                            publishers: HashSet::new(),
                            is_root: true,
                        },
                    );
                    println!("Image registered");
                    format!("Image registered: {}", image_hash)
                } else {
                    "Nothing added...".to_string()
                }
            }

            ImageAction::RegisterEdit {
                original_image_hash,
                edited_image_hash,
                original_edit_signature,
            } => {
                println!("Checking for existing image keys: {:?}", self.hash_map.keys());

                if self.hash_map.contains_key(&edited_image_hash) {
                    "Edited hash already exists!".to_string()
                } else if !self.hash_map.contains_key(&original_image_hash) {
                    "Original image does not exist!".to_string()
                } else {
                    let owner_pk = self.hash_map.get(&original_image_hash).unwrap().owner_pk.clone();
                    let message = format!("{}{}", original_image_hash, edited_image_hash);
                    let signature_verification = dummy_verify_signature(owner_pk.clone(), message, original_edit_signature);

                    if signature_verification {
                        self.hash_map.insert(
                            edited_image_hash.clone(),
                            ImageMetadata {
                                previous_image_hash: Some(original_image_hash.clone()),
                                owner_pk,
                                publishers: HashSet::new(),
                                is_root: false,
                            },
                        );
                        format!("Edit registered successfully: {}", edited_image_hash)
                    } else {
                        "Invalid signature! Edit not registered.".to_string()
                    }
                }
            }

            ImageAction::AddPublisher {
                original_image_hash,
                original_image_signature,
                publisher_pk,
            } => {
                if let Some(image_metadata) = self.hash_map.get_mut(&original_image_hash) {
                    let owner_pk = image_metadata.owner_pk.clone();
                    let is_correct = dummy_verify_signature(owner_pk.clone(), original_image_hash.clone(), original_image_signature);

                    if is_correct && image_metadata.is_root {
                        if !image_metadata.publishers.insert(publisher_pk.clone()) {
                            format!("Publisher {} is already registered!", publisher_pk)
                        } else {
                            format!("Publisher {} added successfully!", publisher_pk)
                        }
                    } else {
                        "Invalid signature or not an original image!".to_string()
                    }
                } else {
                    "Original image does not exist!".to_string()
                }
            }
        };

        println!("Execution result: {}", program_output);
        Ok((program_output, ctx, vec![]))
    }
}

fn dummy_verify_signature(_pk: String, _message: String, signature: String) -> bool {
    signature.to_lowercase() == "true"
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum ImageAction {
    RegisterImage { image_hash: String, image_signature: String, owner_pk: String },
    RegisterEdit { original_image_hash: String, edited_image_hash: String, original_edit_signature: String },
    AddPublisher { original_image_hash: String, original_image_signature: String, publisher_pk: String },
}

/// Utils function for the host
impl ImageState {
    pub fn as_bytes(&self) -> Result<Vec<u8>, Error> {
        borsh::to_vec(self)
    }

    pub fn is_original_image(&self, img_hash: String) -> Result<bool, Error> {
        Ok(self.hash_map.contains_key(&img_hash) && self.hash_map[&img_hash].is_root)
    }
}

impl ImageAction {
    pub fn as_blob(&self, contract_name: &str) -> sdk::Blob {
        sdk::Blob {
            contract_name: contract_name.into(),
            data: sdk::BlobData(borsh::to_vec(self).expect("Failed to encode BlobData")),
        }
    }
}

/// Helpers to transform the contrat's state in its on-chain state digest version.
/// In an optimal version, you would here only returns a hash of the state,
/// while storing the full-state off-chain
impl Digestable for ImageState {
    fn as_digest(&self) -> sdk::StateDigest {
        sdk::StateDigest(borsh::to_vec(self).expect("Failed to encode ImageState"))
    }
}

impl From<sdk::StateDigest> for ImageState {
    fn from(state: sdk::StateDigest) -> Self {
        borsh::from_slice(&state.0)
            .map_err(|_| "Could not decode hyllar state".to_string())
            .unwrap()
    }
}
