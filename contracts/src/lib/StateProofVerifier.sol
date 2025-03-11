// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import "./MerklePatriciaProof.sol";
import "./RLPReader.sol";

/**
 * @title StateProofVerifier
 * @notice This library verifies that a given account’s storage slot holds a particular value and is part of the state trie.
 *
 * The verification process is as follows:
 * 1. Verify the account inclusion in the state trie using the account proof.
 * 2. Decode the account RLP (which is [nonce, balance, storageRoot, codeHash]) to extract the storage root.
 * 3. Verify that the provided storage slot (key/value) is included in the account's storage trie using the storage proof.
 *
 * Note:
 * - For the account trie the key is keccak256(abi.encodePacked(account)).
 * - For the storage trie the key is keccak256(abi.encodePacked(storageSlot)).
 */
library StateProofVerifier {
    using RLPReader for RLPReader.RLPItem;
    using RLPReader for bytes;

    /**
     * @notice Verifies that an account’s storage slot has a particular value and is included in the state trie.
     * @param stateRoot The root hash of the global state trie.
     * @param account The account address.
     * @param accountRlp The RLP‑encoded account value (array of [nonce, balance, storageRoot, codeHash]).
     * @param accountProof An array of RLP‑encoded nodes proving the account’s inclusion in the state trie.
     * @param storageSlot The storage slot key (as bytes32).
     * @param storageValue The expected RLP‑encoded value stored at the given storage slot.
     * @param storageProof An array of RLP‑encoded nodes proving the storage slot’s inclusion in the account’s storage trie.
     * @return valid True if both the account and storage proofs are valid.
     */
    function verifyStateProof(
        bytes32 stateRoot,
        address account,
        bytes memory accountRlp,
        bytes[] memory accountProof,
        bytes32 storageSlot,
        bytes memory storageValue,
        bytes[] memory storageProof
    ) internal pure returns (bool valid) {
        // Convert the accountProof bytes array into an array of RLPItems.
        RLPReader.RLPItem[] memory accountProofItems = new RLPReader.RLPItem[](accountProof.length);
        for (uint256 i = 0; i < accountProof.length; i++) {
            accountProofItems[i] = accountProof[i].toRlpItem();
        }
        
        // Compute the key for the account in the state trie:
        // In Ethereum the key is the keccak256 hash of the account address.
        bytes memory accountKey = abi.encodePacked(keccak256(abi.encodePacked(account)));
        
        // Verify the account proof against the provided state root.
        bool accountValid = MerklePatriciaProof.verifyProof(
            accountRlp,
            accountKey,
            accountProofItems,
            stateRoot
        );
        if (!accountValid) {
            return false;
        }
        
        // Decode the account RLP to extract the storage root.
        // According to the Ethereum state trie, the account RLP has four fields: 
        // [nonce, balance, storageRoot, codeHash].
        RLPReader.RLPItem[] memory accountFields = accountRlp.toRlpItem().toList();
        // We assume that the storage root is at index 2.
        bytes memory storageRootBytes = accountFields[2].toBytes();
        require(storageRootBytes.length == 32, "Invalid storage root length");
        bytes32 storageRoot;
        assembly {
            storageRoot := mload(add(storageRootBytes, 32))
        }
        
        // Convert the storageProof bytes array into an array of RLPItems.
        RLPReader.RLPItem[] memory storageProofItems = new RLPReader.RLPItem[](storageProof.length);
        for (uint256 i = 0; i < storageProof.length; i++) {
            storageProofItems[i] = storageProof[i].toRlpItem();
        }
        
        // Compute the key for the storage trie:
        // In Ethereum the storage trie key is the keccak256 hash of the storage slot.
        bytes memory storageKey = abi.encodePacked(keccak256(abi.encodePacked(storageSlot)));
        
        // Verify the storage proof using the account's storage root.
        bool storageValid = MerklePatriciaProof.verifyProof(
            storageValue,
            storageKey,
            storageProofItems,
            storageRoot
        );
        
        return storageValid;
    }
}
