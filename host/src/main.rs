use anyhow::Result;
use clap::{Parser, Subcommand};
use client_sdk::helpers::risc0::Risc0Prover;
use contract::{ImageRegistry, ImageAction};
use sdk::api::APIRegisterContract;
use sdk::{BlobTransaction, ProofTransaction, ContractInput, Digestable, HyleContract};
use sdk::Identity; // Import Identity for proper type conversion

// RISC-V ELF and image ID for zk proofs
use methods::{GUEST_ELF, GUEST_ID};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, default_value = "http://localhost:4321")]
    pub host: String,

    #[arg(long, default_value = "image_registry")]
    pub contract_name: String,
}

#[derive(Subcommand)]
enum Commands {
    RegisterContract {},
    RegisterImage {
        #[arg(short = 'x', long)]
        hash: String,

        #[arg(short, long)]
        owner: String,
    },
    VerifyImage {
        #[arg(short, long)]
        hash: String,
    },
    RegisterEdit {
        #[arg(short, long)]
        original_hash: String,

        #[arg(short, long)]
        edited_hash: String,

        #[arg(short, long)]
        owner: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Client to send requests to the node
    let client = client_sdk::rest_client::NodeApiHttpClient::new(cli.host)?;
    let contract_name = &cli.contract_name;

    // zk-Prover (now with explicit lifetime)
    let prover = Risc0Prover::new(GUEST_ELF);

    // Identity setup - Convert properly to `Identity`
    let identity = Identity::from(format!("none.{}", contract_name));

    match cli.command {
        // Register the contract on-chain
        Commands::RegisterContract {} => {
            let initial_state = ImageRegistry::default();

            let res = client
                .register_contract(&APIRegisterContract {
                    verifier: "risc0".into(),
                    program_id: sdk::ProgramId(sdk::to_u8_array(&GUEST_ID).to_vec()),
                    state_digest: initial_state.as_digest(),
                    contract_name: contract_name.clone().into(),
                })
                .await?;
            println!("✅ Register contract tx sent. Tx hash: {}", res);
        }

        // Register a new image on-chain
        Commands::RegisterImage { hash, owner } => {
            let action = ImageAction::RegisterImage {
                hash: hex::decode(hash).expect("Invalid hash format"),
                owner,
            };
            process_transaction(&client, contract_name, identity.clone(), action, &prover).await?;
        }

        // Verify an image's authenticity
        Commands::VerifyImage { hash } => {
            let action = ImageAction::VerifyImage {
                hash: hex::decode(hash).expect("Invalid hash format"),
            };
            process_transaction(&client, contract_name, identity.clone(), action, &prover).await?;
        }

        // Register an edited version of an image
        Commands::RegisterEdit {
            original_hash,
            edited_hash,
            owner,
        } => {
            let action = ImageAction::RegisterEdit {
                original_hash: hex::decode(original_hash).expect("Invalid hash format"),
                edited_hash: hex::decode(edited_hash).expect("Invalid hash format"),
                owner,
            };
            process_transaction(&client, contract_name, identity.clone(), action, &prover).await?;
        }
    }

    Ok(())
}

/// Handles sending the transaction and proving the state transition.
async fn process_transaction<'a>(
    client: &client_sdk::rest_client::NodeApiHttpClient,
    contract_name: &str,
    identity: Identity, // Corrected type
    action: ImageAction,
    prover: &'a Risc0Prover<'a>,
) -> Result<()> {
    // Fetch contract state
    let mut initial_state: ImageRegistry = client
        .get_contract(&contract_name.into())
        .await
        .unwrap()
        .state
        .into();

    // Prepare blob transaction
    let blobs = vec![sdk::Blob {
        contract_name: contract_name.into(),
        data: sdk::BlobData(borsh::to_vec(&action).expect("failed to encode BlobData")),
    }];

    // ✅ Clone `identity` before passing it to avoid move error
    let blob_tx = BlobTransaction::new(identity.clone(), blobs.clone());

    // Send blob transaction
    let blob_tx_hash = client.send_tx_blob(&blob_tx).await.unwrap();
    println!("✅ Blob tx sent. Tx hash: {}", blob_tx_hash);

    // Serialize contract state correctly
    let state_bytes = borsh::to_vec(&initial_state).expect("Failed to serialize state");

    // ✅ Clone `identity` before `.into()` to avoid move error
    let inputs = ContractInput {
        state: state_bytes,
        identity: identity.clone().into(), // Convert explicitly if needed
        tx_hash: blob_tx_hash,
        private_input: vec![],
        tx_ctx: None,
        blobs: blobs.clone(),
        index: sdk::BlobIndex(0),
    };

    let (program_outputs, _, _) = initial_state.execute(&inputs).unwrap();
    println!("🚀 Executed: {}", program_outputs);

    // Generate zk proof
    let proof = prover.prove(inputs).await.unwrap();
    assert!(prover.verify(&proof).is_ok(), "Proof failed verification!");

    // Submit proof transaction
    let proof_tx = ProofTransaction {
        proof,
        contract_name: contract_name.into(),
    };

    let proof_tx_hash = client.send_tx_proof(&proof_tx).await.unwrap();
    println!("✅ Proof tx sent. Tx hash: {}", proof_tx_hash);

    Ok(())
}
