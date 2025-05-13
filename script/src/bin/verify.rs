use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use sp1_sdk::{ProverClient, SP1ProofWithPublicValues, utils};
use std::fs;
use std::path::PathBuf;
use sha3::{Digest, Keccak256};
use hex;

// Public inputs structure
#[derive(Deserialize, Serialize, Debug)]
struct PublicInputs {
    message_digest: String,
    merkle_root: String,
}

// Balance range result
#[derive(Debug, Serialize, Deserialize)]
struct BalanceRange {
    in_range: bool,
    min_balance: u64,
    max_balance: u64,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Verify a previously generated proof
    Verify {
        /// Path to the binary proof file generated with the 'prove' command
        #[arg(short, long)]
        proof_file: PathBuf,
    },
}

fn compute_message_digest(message: &str) -> String {
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
    let mut hasher = Keccak256::new();
    hasher.update(prefix.as_bytes());
    hasher.update(message.as_bytes());
    let hash = hasher.finalize();
    format!("0x{}", hex::encode(hash))
}

fn main() {
    // Setup logging
    utils::setup_logger();
    
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Verify { proof_file } => {
            println!("Verifying token ownership proof...");
            
            // Get the ELF file
            let elf_path = std::env::var("SP1_ELF_token-ownership-program")
                .expect("ELF path not found. Did you run 'cargo prove build' in the program directory?");
            let elf = fs::read(elf_path).expect("Failed to read ELF file");
            
            // Create a ProverClient
            let client = ProverClient::from_env();
            
            // Load the proof (binary format)
            let proof = SP1ProofWithPublicValues::load(proof_file).expect("Failed to load proof");
            
            // Setup the verification key
            let (_, vk) = client.setup(&elf);
            
            // Verify the proof
            client.verify(&proof, &vk).expect("Proof verification failed");
            
            // Read public outputs
            let mut public_values = proof.public_values.clone();
            let balance_range: BalanceRange = public_values.read();
            let committed_public_inputs: PublicInputs = public_values.read();
            
            println!("\n=== Proof Successfully Verified ===");
            if balance_range.in_range {
                println!("✓ Verified: Balance is between {} and {} HYPE", 
                    balance_range.min_balance as f64 / 100_000_000.0,
                    balance_range.max_balance as f64 / 100_000_000.0);
            } else {
                println!("✗ Balance is NOT within the required range");
            }
            println!("\nCommitted Public Values:");
            println!("- Message Digest: {}", committed_public_inputs.message_digest);
            println!("- Merkle Root: {}", committed_public_inputs.merkle_root);

            // --- Added logic to show the digest of the known message ---
            let message = "I am @freakmycrypto and I am proving my $HYPE holdings";
            let computed_digest = compute_message_digest(message);
            println!("\nKnown message: '{}" , message);
            println!("Computed digest from message: {}", computed_digest);
            if computed_digest == committed_public_inputs.message_digest {
                println!("✓ The committed message_digest matches the digest of the known message.");
            } else {
                println!("✗ The committed message_digest does NOT match the digest of the known message.");
            }
        }
    }
} 