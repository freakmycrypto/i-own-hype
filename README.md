# $HYPE Token Ownership Proof Verification

This repository allows you to **verify a zero-knowledge proof** that the prover controls Ethereum addresses holding a total of **700,000–701,000 $HYPE tokens** on Hyperliquid, as of block snapshot [#594790000 (May 14, 2025)](https://app.hyperliquid.xyz/explorer/block/594790000).

## What Does This Prove?
- The prover controls addresses included in a Merkle tree of address:balance pairs, as created by [`helpers/build_merkle.js`](helpers/build_merkle.js), corresponding to $HYPE balances at block 594790000.
- The total $HYPE balance controlled is **between 700,000 and 701,000**.
- The proof **does not reveal which addresses** are controlled.
- The proof publicly commits to:
  - **Merkle root:** `1b09fe46084cde399ccc6823c68fc74ce94aafc30e0a781eab0de9033b2cfe30` (can be recreated with `build_merkle.js`)
  - **Message digest:** `0x4e563b86ca0109cd9d73da502a033f70352f1c1b8722a2b6c33fb9e672d93ef6` (corresponds to `'I am @freakmycrypto and I am proving my $HYPE holdings'`)

## Prerequisites
Install the following before verifying the proof:
- **Rust & Cargo:** [Install instructions](https://www.rust-lang.org/tools/install)
- **SP1 toolchain:**
  ```bash
  curl -L https://sp1up.succinct.xyz | bash && sp1up
  ```
- **Docker:** [Install instructions](https://docs.docker.com/get-docker/)
- **Node.js & npm:** (for optional helper scripts)
- **Python 3.x & pip:** (for optional data processing)

## How to Verify the Proof

1. **Build the ELF (reproducible, required for verification):**
   ```bash
   cd program
   cargo prove build --docker --tag v4.0.0
   # This creates: target/elf-compilation/docker/riscv32im-succinct-zkvm-elf/release/token-ownership-program
   ```

2. **Verify the proof:**
   ```bash
   cd ../script
   cargo run --release verify --proof-file proof.bin
   ```
   - This checks that `proof.bin` was generated against the exact ELF you just built.
   - The output will confirm the $HYPE balance range and print the committed Merkle root and message digest.

## Auditing and Data Reproducibility
- **Proof Logic:**
  The full logic of the proof is open source and can be audited in [`program/src/main.rs`](program/src/main.rs).
- **Reproducibility:**
  - The file [`data/balance.json`](data/balance.json) can be **recreated** by running the provided `get_hype_balances.py` script on the official Hyperliquid block 594790000 snapshot. (See helper scripts and comments in the repo for details.)
  - The **Merkle root** can be independently recomputed by running the provided `build_merkle.js` script on the processed balances.
  - The **message digest** is recomputed and displayed as part of the verification script output (see the printed digest check in the terminal).
  - The **ELF binary** used for proof generation and verification should be byte-for-byte identical. You can verify its SHA-512 hash:
    ```bash
    shasum -a 512 target/elf-compilation/docker/riscv32im-succinct-zkvm-elf/release/token-ownership-program
    ```
    The output should be:
    ```
    db7cd0f44663d3f8139eca7808ab551d34d063cd1efa6d9a5212844cc4af81a29db7a4923afb68cde0b9cec6c7a7c92cb2425058e6697725debd161c4686a7ee  target/elf-compilation/docker/riscv32im-succinct-zkvm-elf/release/token-ownership-program
    ```

---

