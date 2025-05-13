const fs = require('fs-extra');
const keccak256 = require('keccak256');

// Define paths
const BALANCES_FILE = '../data/balances.json';
const MERKLE_ROOT_FILE = '../data/merkle_root.json';
const MERKLE_PROOFS_FILE = '../data/merkle_proofs.json';

// Hash a leaf (address:balance pair)
function hashLeaf(address, balance) {
    const leafStr = address.toLowerCase() + ':' + balance.toString();
    return keccak256(leafStr);
}

// Build a level of the tree
function buildLevel(nodes) {
    const newLevel = [];
    for (let i = 0; i < nodes.length; i += 2) {
        const left = nodes[i];
        const right = i + 1 < nodes.length ? nodes[i + 1] : nodes[i];
        
        const input = Buffer.alloc(64);
        left.copy(input, 0, 0, 32);
        right.copy(input, 32, 0, 32);
        
        newLevel.push(keccak256(input));
    }
    return newLevel;
}

// Generate Merkle proof for a leaf
function generateProof(leafIndex, leaves) {
    const proof = [];
    let currentIndex = leafIndex;
    let currentLevel = leaves.map(leaf => Buffer.from(leaf.hash, 'hex'));
    
    while (currentLevel.length > 1) {
        const siblingIndex = currentIndex % 2 === 0 ? currentIndex + 1 : currentIndex - 1;
        
        if (siblingIndex < currentLevel.length) {
            proof.push(currentLevel[siblingIndex].toString('hex'));
        } else {
            proof.push(currentLevel[currentIndex].toString('hex'));
        }
        
        currentLevel = buildLevel(currentLevel);
        currentIndex = Math.floor(currentIndex / 2);
    }
    
    return proof;
}

async function buildMerkleRoot() {
    try {
        const addresses = await fs.readJson(BALANCES_FILE);
        console.log(`Building merkle tree for ${addresses.length} addresses...`);
        
        // Create leaves
        const leaves = addresses.map(item => ({
            address: item.address,
            balance: Number(item.balance),
            hash: hashLeaf(item.address, item.balance).toString('hex')
        }));
        
        // Build tree and get root
        let currentLevel = leaves.map(leaf => Buffer.from(leaf.hash, 'hex'));
        while (currentLevel.length > 1) {
            currentLevel = buildLevel(currentLevel);
        }
        const merkleRoot = currentLevel[0].toString('hex');
        
        // Save root and leaves
        const rootData = {
            merkle_root: merkleRoot,
            leaves: leaves
        };
        
        await fs.writeJson(MERKLE_ROOT_FILE, rootData, { spaces: 2 });
        return leaves;
    } catch (error) {
        console.error('Error in buildMerkleRoot:', error);
        throw error;
    }
}

async function generateProofs(leaves) {
    try {
        const proofs = {};
        let processed = 0;
        
        for (let idx = 0; idx < leaves.length; idx++) {
            const leaf = leaves[idx];
            const proof = generateProof(idx, leaves);
            proofs[leaf.address] = {
                address: leaf.address,
                balance: leaf.balance,
                leaf_hash: '0x' + leaf.hash,
                inclusion_branches: {
                    index: idx,
                    proof: proof
                }
            };
            
            processed++;
            if (processed % 1000 === 0) {
                console.log(`Processed ${processed}/${leaves.length} proofs...`);
                await fs.writeJson(MERKLE_PROOFS_FILE, proofs, { spaces: 2 });
            }
        }
        
        // Final save
        await fs.writeJson(MERKLE_PROOFS_FILE, proofs, { spaces: 2 });
        console.log('Done! Merkle tree data saved.');
    } catch (error) {
        console.error('Error in generateProofs:', error);
        throw error;
    }
}

async function main() {
    try {
        const leaves = await buildMerkleRoot();
        await generateProofs(leaves);
    } catch (error) {
        console.error('Fatal error:', error);
        process.exit(1);
    }
}

main(); 