use std::collections::HashMap;
use borsh::{io::Error, BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use sdk::{Digestable, HyleContract, RunResult};

impl HyleContract for ImageState {
    /// Entry point of the contract's logic
    fn execute(&mut self, contract_input: &sdk::ContractInput) -> RunResult {
        // Parse contract inputs
        let (action, ctx) = sdk::utils::parse_raw_contract_input::<ImageAction>(contract_input)?;

        // Execute the contract logic
        let program_output = match action {
            ImageAction::RegisterImage { image_hash, image_signature } => {
                if !self.hash_map.contains_key(&image_hash) {
                    self.hash_map.insert(image_hash, None);
                    "Image registered by ".to_string()
                }else{
                    "Nothing added...".to_string()
                }
            }
        };
        println!("Arriving to ok");
        Ok((program_output, ctx, vec![]))
    }
}

/// The action represents the different operations that can be done on the contract
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum ImageAction {
    RegisterImage { image_hash: String, image_signature: String },
    //VerifyOriginalImage { image_hash: String },
    // RegisterEdit { original_hash: Vec<u8>, edited_hash: Vec<u8>, owner: String },
}

/// The state of the contract, in this example it is fully serialized on-chain
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
pub struct ImageState {
    pub hash_map: HashMap<String, Option<String>>,
}

/// Utils function for the host
impl ImageState {
    pub fn as_bytes(&self) -> Result<Vec<u8>, Error> {
        borsh::to_vec(self)
    }

    pub fn is_original_image(&self, img_hash: String) -> Result<bool, Error> {
        match self.hash_map.get(&img_hash){
            Some(x)=> Ok(true),
            None => Ok(false),
        }
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
