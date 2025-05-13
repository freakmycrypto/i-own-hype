# Token Ownership Zero-Knowledge Proof

This repository contains everything needed to verify a zero-knowledge proof of $HYPE token ownership. The proof demonstrates control of Ethereum addresses containing a balance of $HYPE tokens between 700,000 and 701,000 HYPE on Hyperliquid, without revealing which addresses are controlled.

## What's Being Verified

The proof verifies that:
1. The prover controls Ethereum addresses that appear in a specific Merkle tree (state snapshot from Hyperliquid)
2. These addresses collectively contain a balance of $HYPE tokens between 700,000 and 701,000 HYPE
3. All without revealing which addresses are controlled or their individual balances

## Prerequisites

- Rust and Cargo
- SP1 toolchain (install with `curl -L https://sp1up.succinct.xyz | bash && sp1up`)
- Docker (for reproducible builds)
- Node.js (for helper scripts, optional)
- Python 3.x (for state snapshot processing, optional)

## Repository Structure

- `program/`: The SP1 program that performs the verification
- `script/`: Contains the verification script
- `helpers/`: Scripts to reproduce Merkle tree from state snapshot (optional)
  - `get_hype_balances.py`: Extract $HYPE balances from Hyperliquid state snapshot
  - `build_merkle.js`: Generate Merkle tree from balances
- `data/`: Contains processed data from Hyperliquid block #561930000:
  - `balances.json`: Processed $HYPE balances (already generated)
  - `merkle_root.json`: Generated Merkle root from this state snapshot (already generated)
  - `merkle_proofs.json`: Generated Merkle proofs for each address (already generated)

## Verifying the State Snapshot (Optional)

The processed files in `data/` are already included and ready to use. However, if you want to verify the state snapshot and regenerate the Merkle root yourself:

1. Install dependencies:
```bash
cd helpers
# Install Node.js dependencies
npm install
# Install Python dependencies
pip install -r requirements.txt
```

2. Process the state snapshot to extract balances:
```bash
# Place your state_561930000.json file in the data directory first
python get_hype_balances.py
```
This will:
- Read the state snapshot from block #561930000
- Extract address:$HYPE balance pairs from Hyperliquid state
- Sort by balance (largest to smallest)
- Take the top 10,000 addresses
- Save to `data/balances.json`

3. Generate Merkle tree from balances:
```bash
node build_merkle.js
```
This will:
- Read the processed balances from `data/balances.json`
- Generate the Merkle tree and root
- Save the root data to `data/merkle_root.json`
- Save individual proofs to `data/merkle_proofs.json`

## Building the Program

For development (faster, but not reproducible):
```bash
cd program
cargo prove build
```

For production (reproducible builds using Docker):
```bash
cd program
cargo prove build --docker --tag v4.0.0
```

The Docker build is recommended for production use as it generates reproducible ELF files that will be identical across all platforms.

## Security Notes

1. The Merkle root represents the Hyperliquid state at block #561930000 - verify this matches your expected state
2. The program code is public and builds reproducibly with Docker - you can verify exactly what's being proven
3. No private data (addresses, individual balances, or signatures) is included in this repository
4. You can optionally reproduce the Merkle tree from the state snapshot using the helper scripts, though the processed files are already provided 