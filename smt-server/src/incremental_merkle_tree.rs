use alloy_primitives::B256;
use sha3::{Digest, Keccak256};
pub struct MerkleTree {
    tree: Vec<Vec<B256>>,
    zero_hashes: Vec<B256>,
    height: usize,
    count: usize,
}

impl MerkleTree {
    pub fn new(height: usize) -> Self {
        assert!(height > 1);
        let mut tree = Vec::with_capacity(height + 1);
        for _ in 0..=height {
            tree.push(Vec::new());
        }
        
        let zero_hashes = Self::generate_zero_hashes(height);
        
        Self {
            tree,
            zero_hashes,
            height,
            count: 0,
        }
    }

    fn generate_zero_hashes(height: usize) -> Vec<B256> {
        let mut zero_hashes = vec![B256::ZERO; height];
        for i in 0..height - 1 {
            zero_hashes[i + 1] = hash_pair(zero_hashes[i], zero_hashes[i]);
        }
        zero_hashes
    }

    pub fn len(&self) -> usize {
        self.count
    }

    #[inline]
    pub fn append(&mut self, leaf: B256) -> B256 {
        self.tree[0].push(leaf);
        self.count += 1;
        self.calc_branches();
        self.root()
    }

    #[inline]
    fn calc_branches(&mut self) {
        for i in 0..self.height {
            self.tree[i + 1].clear();
            let child = &self.tree[i].clone();
            
            for j in (0..child.len()).step_by(2) {
                let left_node = child[j];
                let right_node = if j + 1 < child.len() {
                    child[j + 1]
                } else {
                    self.zero_hashes[i]
                };
                
                self.tree[i + 1].push(hash_pair(left_node, right_node));
            }
        }
    }

    #[inline]
    pub fn generate_proof(&self, index: usize) -> Vec<B256> {
        let mut proof = Vec::with_capacity(self.height);
        let mut current_index = index;

        for i in 0..self.height {
            current_index = if current_index % 2 == 1 {
                current_index - 1
            } else {
                current_index + 1
            };

            let sibling = if current_index < self.tree[i].len() {
                self.tree[i][current_index]
            } else {
                self.zero_hashes[i]
            };
            
            proof.push(sibling);
            current_index /= 2;
        }

        proof
    }

    pub fn root(&self) -> B256 {
        if self.tree[self.height].is_empty() {
            let last_zero = self.zero_hashes[self.height - 1];
            hash_pair(last_zero, last_zero)
        } else {
            self.tree[self.height][0]
        }
    }

    pub fn verify_proof(leaf: B256, index: usize, root: B256, proof: Vec<B256>) -> bool {
        let mut current = leaf;
        let mut current_index = index;

        for (i, sibling) in proof.iter().enumerate() {
            let (left, right) = if current_index % 2 == 0 {
                (current, *sibling)
            } else {
                (*sibling, current)
            };
            
            current = hash_pair(left, right);
            current_index /= 2;
        }

        current == root
    }
}

#[inline]
fn hash_pair(left: B256, right: B256) -> B256 {
    let mut hasher = Keccak256::new();
    hasher.update(left.as_slice());
    hasher.update(right.as_slice());
    B256::from_slice(&hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_leaf(value: u8) -> B256 {
        let mut bytes = [1u8; 32];
        bytes[0] = value;
        B256::from(bytes)
    }

    #[test]
    fn test_basic_tree_operations() {
        let mut tree = MerkleTree::new(3);  // Height of 3 for simpler testing
        
        // Add first leaf
        let leaf1 = create_test_leaf(1);
        let root1 = tree.append(leaf1);
        
        // Generate and verify proof for first leaf
        let proof1 = tree.generate_proof(0);
        assert!(MerkleTree::verify_proof(leaf1, 0, root1, proof1));
        
        // Add second leaf
        let leaf2 = create_test_leaf(2);
        let root2 = tree.append(leaf2);
        
        // Verify both leaves
        let proof1_after = tree.generate_proof(0);
        let proof2 = tree.generate_proof(1);
        
        assert!(MerkleTree::verify_proof(leaf1, 0, root2, proof1_after));
        assert!(MerkleTree::verify_proof(leaf2, 1, root2, proof2));
    }

    #[test]
    fn test_zero_hashes() {
        let tree = MerkleTree::new(3);
        assert_eq!(tree.zero_hashes[0], B256::ZERO);
        assert_eq!(tree.zero_hashes[1], hash_pair(B256::ZERO, B256::ZERO));
    }

    #[test]
    fn test_multiple_leaves() {
        let mut tree = MerkleTree::new(4);
        let leaves: Vec<B256> = (0..5).map(|i| create_test_leaf(i as u8)).collect();
        
        // Add all leaves
        for leaf in &leaves {
            tree.append(*leaf);
        }
        
        let root = tree.root();
        
        // Verify all proofs
        for (i, leaf) in leaves.iter().enumerate() {
            let proof = tree.generate_proof(i);
            assert!(MerkleTree::verify_proof(*leaf, i, root, proof));
        }
    }

    #[test]
    fn test_proof_verification_fails_with_wrong_index() {
        let mut tree = MerkleTree::new(3);
        let leaf = create_test_leaf(1);
        let root = tree.append(leaf);
        let proof = tree.generate_proof(0);
        
        // Try to verify with wrong index
        assert!(!MerkleTree::verify_proof(leaf, 1, root, proof));
    }

    #[test]
    fn test_empty_tree() {
        let mut tree = MerkleTree::new(3);
        let root = tree.root();
        
        // An empty tree's root should match the hash of the highest level zero hash with itself
        let expected_root = {
            let zero_hashes = MerkleTree::generate_zero_hashes(3);
            let last_zero = zero_hashes[zero_hashes.len() - 1];
            hash_pair(last_zero, last_zero)
        };
        assert_eq!(root, expected_root);
    }

    #[test]
fn test_larger_tree_sequence() {
    let mut tree = MerkleTree::new(5);  // Height of 5, allowing 31 leaves
    let mut prev_root = tree.root();
    let mut roots = vec![prev_root];
    let mut leaves = vec![];  // Store leaves
    
    // Add 10 leaves and store all intermediate roots
    for i in 0..10 {
        let leaf = create_test_leaf(i as u8);
        leaves.push(leaf);  // Store the leaf
        let new_root = tree.append(leaf);
        assert_ne!(prev_root, new_root, "Root should change after adding leaf {}", i);
        roots.push(new_root);
        prev_root = new_root;
    }

    // Verify all historical roots still validate against their respective proofs
    for i in 0..10 {
        let leaf = leaves[i];  // Use stored leaf instead of creating new one
        let proof = tree.generate_proof(i);
        assert!(MerkleTree::verify_proof(leaf, i, prev_root, proof.clone()));
        assert!(!MerkleTree::verify_proof(leaf, i, roots[i], proof));
    }
}

    #[test]
    fn test_full_binary_subtree() {
        let mut tree = MerkleTree::new(3);  // Height 3 allows 7 leaves
        let leaves: Vec<B256> = (0..4).map(|i| create_test_leaf(i as u8)).collect();
        
        // Fill first 4 leaves (making a full binary subtree of height 2)
        for leaf in &leaves {
            tree.append(*leaf);
        }
        
        let root = tree.root();
        
        // All proofs should be valid
        for (i, &leaf) in leaves.iter().enumerate() {
            let proof = tree.generate_proof(i);
            assert_eq!(proof.len(), 3, "Proof length should match tree height");
            assert!(MerkleTree::verify_proof(leaf, i, root, proof));
        }
    }

    #[test]
    fn test_proof_reuse() {
        let mut tree = MerkleTree::new(4);
        let leaf1 = create_test_leaf(1);
        let leaf2 = create_test_leaf(2);
        
        tree.append(leaf1);
        let root1 = tree.append(leaf2);
        
        // Get proofs for both leaves
        let proof1 = tree.generate_proof(0);
        let proof2 = tree.generate_proof(1);
        
        // Verify proofs can't be reused for wrong indices
        assert!(!MerkleTree::verify_proof(leaf1, 1, root1, proof1.clone()));
        assert!(!MerkleTree::verify_proof(leaf2, 0, root1, proof2.clone()));
        
        // Verify proofs can't be reused for wrong leaves
        assert!(!MerkleTree::verify_proof(leaf2, 0, root1, proof1));
        assert!(!MerkleTree::verify_proof(leaf1, 1, root1, proof2));
    }

    #[test]
    fn test_zero_hash_consistency() {
        let tree1 = MerkleTree::new(4);
        let tree2 = MerkleTree::new(4);
        
        assert_eq!(tree1.zero_hashes, tree2.zero_hashes, "Zero hashes should be deterministic");
        
        // Verify each level is hash of previous level
        for i in 1..tree1.zero_hashes.len() {
            assert_eq!(
                tree1.zero_hashes[i],
                hash_pair(tree1.zero_hashes[i-1], tree1.zero_hashes[i-1])
            );
        }
    }

    #[test]
    fn test_sequence_boundaries() {
        let mut tree = MerkleTree::new(3);  // Height 3 allows 7 leaves
        
        // Add leaves one by one and verify all previous proofs
        let mut leaves = vec![];
        let mut all_roots = vec![tree.root()];
        
        for i in 0..7 {
            let leaf = create_test_leaf(i as u8);
            leaves.push(leaf);
            let new_root = tree.append(leaf);
            all_roots.push(new_root);
            
            // Verify all leaves up to now
            for (j, &prev_leaf) in leaves.iter().enumerate() {
                let proof = tree.generate_proof(j);
                assert!(
                    MerkleTree::verify_proof(prev_leaf, j, new_root, proof),
                    "Failed to verify leaf {} after adding leaf {}", j, i
                );
            }
        }
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn test_invalid_height() {
        MerkleTree::new(1); // Should panic as height must be > 1
    }

    #[test]
    fn test_height_32_tree() {
        // Create tree with same height as Ethereum deposit contract
        let mut tree = MerkleTree::new(32);
        
        // Test initial state
        let initial_root = tree.root();
        let expected_initial_root = {
            let zero_hashes = MerkleTree::generate_zero_hashes(32);
            let last_zero = zero_hashes[zero_hashes.len() - 1];
            hash_pair(last_zero, last_zero)
        };
        assert_eq!(initial_root, expected_initial_root, "Empty tree root should match");

        // Add 100 leaves and verify all proofs after each addition
        let mut leaves = vec![];
        let mut roots = vec![initial_root];

        for i in 0..100 {
            let leaf = create_test_leaf(i as u8);
            leaves.push(leaf);
            let new_root = tree.append(leaf);
            roots.push(new_root);

            // Verify all previous leaves with latest root
            for (j, &prev_leaf) in leaves.iter().enumerate() {
                let proof = tree.generate_proof(j);
                assert_eq!(proof.len(), 32, "Proof length should be 32");
                assert!(
                    MerkleTree::verify_proof(prev_leaf, j, new_root, proof),
                    "Failed to verify leaf {} after adding leaf {}", j, i
                );
            }

            // Also verify current leaf with all historical roots
            if i > 0 {
                let current_proof = tree.generate_proof(i);
                // Current leaf should not verify against any previous root
                for previous_root in &roots[0..roots.len()-1] {
                    assert!(
                        !MerkleTree::verify_proof(leaf, i, *previous_root, current_proof.clone()),
                        "Leaf {} should not verify against historical root", i
                    );
                }
            }
        }

        // Verify all proofs one final time
        let final_root = tree.root();
        for (i, &leaf) in leaves.iter().enumerate() {
            let proof = tree.generate_proof(i);
            assert_eq!(proof.len(), 32, "Final proof length should be 32");
            assert!(
                MerkleTree::verify_proof(leaf, i, final_root, proof),
                "Failed to verify leaf {} against final root", i
            );
        }

        // Test some random indices far apart
        let indices = [0, 50, 99];
        for &i in &indices {
            let leaf = create_test_leaf(i as u8);
            let proof = tree.generate_proof(i);
            assert!(
                MerkleTree::verify_proof(leaf, i, final_root, proof),
                "Failed to verify distant leaf {}", i
            );
        }

        // Verify that proofs fail for indices beyond what we've added
        let bad_index = 100;
        let leaf = create_test_leaf(1);
        let proof = tree.generate_proof(99); // Use last valid proof
        assert!(
            !MerkleTree::verify_proof(leaf, bad_index, final_root, proof),
            "Should not verify leaf at invalid index"
        );
    }
}
