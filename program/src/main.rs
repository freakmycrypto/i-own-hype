//! Token Ownership Zero-Knowledge Proof
//! 
//! This program proves ownership of tokens in zero-knowledge:
//! 1. Verifies signatures prove ownership of Ethereum addresses
//! 2. Verifies Merkle proofs show these addresses are in the token distribution
//! 3. Proves the total balance is within a specific range without revealing exact amount

#![no_main]
sp1_zkvm::entrypoint!(main);

use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use k256::{
    ecdsa::{Signature, VerifyingKey, RecoveryId},
};
use std::collections::HashSet;
use alloy_primitives::Address;

// Constants for balance range (700k-701k HYPE)
const MIN_BALANCE: u64 = 70_000_000_000_000; // 700,000 HYPE
const MAX_BALANCE: u64 = 70_100_000_000_000; // 701,000 HYPE

// Public inputs structure
#[derive(Deserialize, Serialize, Debug)]
struct PublicInputs {
    message_digest: String,
    merkle_root: String,
}

// Structure for inclusion branches in Merkle proofs
#[derive(Debug, Clone, Serialize, Deserialize)]
struct InclusionBranches {
    pub index: u32,
    pub proof: Vec<String>,
}

// Structure for a single address claim - now without the address field
#[derive(Debug, Serialize, Deserialize)]
struct SignedMessage {
    signature: String,
    balance: u64,
    inclusion_branches: InclusionBranches,
}

// Private inputs structure
#[derive(Debug, Serialize, Deserialize)]
struct PrivateInputs {
    signed_messages: Vec<SignedMessage>,
}

// Balance range result
#[derive(Debug, Serialize, Deserialize)]
struct BalanceRange {
    in_range: bool,
    min_balance: u64,
    max_balance: u64,
}

// Convert a hex string to a 32-byte array
fn hex_to_bytes32(hex: &str) -> [u8; 32] {
    let hex_str = if hex.starts_with("0x") { &hex[2..] } else { hex };
    let bytes = hex::decode(hex_str).unwrap();
    let mut result = [0u8; 32];
    result.copy_from_slice(&bytes);
    result
}

// Recovers a public key from a signature and message digest, returns uncompressed pubkey bytes (no prefix)
fn recover_pubkey_with_digest(message_digest_hex: &str, signature: &str) -> Vec<u8> {
    let sig_bytes = hex::decode(&signature[2..]).unwrap();
    let recovery_byte = sig_bytes[64];
    let recovery_id = RecoveryId::try_from((recovery_byte - 27) as u8).unwrap();
    let signature = Signature::try_from(&sig_bytes[..64]).unwrap();
    let message_digest = hex_to_bytes32(message_digest_hex);
    let recovered_key = VerifyingKey::recover_from_prehash(&message_digest, &signature, recovery_id).unwrap();
    // Return the uncompressed public key bytes (skip the 0x04 prefix for Alloy)
    recovered_key.to_encoded_point(false).as_bytes()[1..].to_vec()
}

// Hash a leaf (address, balance) pair using keccak256
fn hash_leaf(address: &Address, balance: u64) -> [u8; 32] {
    let address_str = format!("0x{:x}", address);
    let balance = balance.to_string();
    let leaf_str = address_str.to_lowercase() + ":" + &balance;
    let mut hasher = Keccak256::new();
    hasher.update(leaf_str.as_bytes());
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

// Compute the Merkle root from a leaf hash and inclusion proof
fn compute_inclusion_root(commitment: [u8; 32], proof: &InclusionBranches) -> [u8; 32] {
    let bits = proof.index;
    let mut root = commitment;
    
    for (i, hash_hex) in proof.proof.iter().enumerate() {
        let hash = hex_to_bytes32(hash_hex);
        
        if bits & (1 << i) == 0 {
            let mut input = [0u8; 64];
            input[..32].copy_from_slice(&root);
            input[32..].copy_from_slice(&hash);
            let mut hasher = Keccak256::new();
            hasher.update(input);
            root.copy_from_slice(&hasher.finalize()[..32]);
        } else {
            let mut input = [0u8; 64];
            input[..32].copy_from_slice(&hash);
            input[32..].copy_from_slice(&root);
            let mut hasher = Keccak256::new();
            hasher.update(input);
            root.copy_from_slice(&hasher.finalize()[..32]);
        }
    }
    
    root
}

pub fn main() {
    // Read public and private inputs
    let public_inputs: PublicInputs = sp1_zkvm::io::read();
    let private_inputs: PrivateInputs = sp1_zkvm::io::read();
    
    // Get expected Merkle root
    let expected_merkle_root = hex_to_bytes32(&public_inputs.merkle_root);
    
    // Track which addresses we've already processed to prevent double-counting
    let mut seen_addresses = HashSet::new();
    
    // Verify all signatures and proofs
    let mut total_balance = 0u64;
    
    for signed_message in private_inputs.signed_messages.iter() {
        // Step 1: Recover the Ethereum address from the signature
        let pubkey_bytes = recover_pubkey_with_digest(&public_inputs.message_digest, &signed_message.signature);
        let recovered_address = Address::from_raw_public_key(&pubkey_bytes);
        
        // Skip if we've already processed this address
        if seen_addresses.contains(&recovered_address) {
            continue;
        }
        
        // Step 3: Compute the leaf hash using the recovered address
        let leaf_hash = hash_leaf(&recovered_address, signed_message.balance);
        
        // Step 4: Verify the Merkle proof
        let computed_root = compute_inclusion_root(leaf_hash, &signed_message.inclusion_branches);
        
        // Step 5: Verify the computed root matches the expected root
        if computed_root == expected_merkle_root {
            // Step 6: Add the balance to the total and mark this address as seen
            total_balance += signed_message.balance;
            seen_addresses.insert(recovered_address);
        }
    }
    
    // Check if total balance is in range and create result
    let balance_range = BalanceRange {
        in_range: total_balance >= MIN_BALANCE && total_balance <= MAX_BALANCE,
        min_balance: MIN_BALANCE,
        max_balance: MAX_BALANCE,
    };
    
    // Commit the balance range result and public inputs
    sp1_zkvm::io::commit(&balance_range);
    sp1_zkvm::io::commit(&public_inputs);
} 