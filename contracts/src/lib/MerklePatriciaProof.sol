// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import "./RLPReader.sol";

/**
 * @title MerklePatriciaProof
 * @dev A library for verifying Ethereum storage proofs using Merkle Patricia tries.
 * The implementation is based on academic descriptions of the Ethereum state trie :contentReference[oaicite:3]{index=3}
 * and on proven open-source implementations (e.g. :contentReference[oaicite:4]{index=4}, :contentReference[oaicite:5]{index=5}).
 */
library MerklePatriciaProof {
    using RLPReader for RLPReader.RLPItem;
    using RLPReader for bytes;

    /**
     * @notice Verifies a Merkle Patricia proof.
     * @param value The expected RLP-encoded value stored at the key (empty if no value).
     * @param encodedPath The key (storage slot) as a byte array (before compact encoding).
     * @param proof An array of RLP-encoded nodes that form the proof.
     * @param root The root hash of the trie.
     * @return True if the proof is valid and the value is correctly proved.
     */
    function verifyProof(
        bytes memory value,
        bytes memory encodedPath,
        RLPReader.RLPItem[] memory proof,
        bytes32 root
    ) internal pure returns (bool) {
        // Convert the key into a nibble array.
        bytes memory path = _getNibbleArray(encodedPath);
        uint pathPtr = 0;

        // The first node in the proof should hash to the provided root.
        bytes memory currentNodeRlp = proof[0].toRlpBytes();
        if (keccak256(currentNodeRlp) != root) {
            return false;
        }

        // Traverse each node in the proof.
        for (uint i = 0; i < proof.length; i++) {
            RLPReader.RLPItem[] memory node = proof[i].toList();

            if (node.length == 17) {
                // -- Branch node --
                if (pathPtr == path.length) {
                    // If no more nibbles remain, the value should be stored at index 16.
                    if (keccak256(node[16].toBytes()) == keccak256(value)) {
                        return true;
                    } else {
                        return false;
                    }
                }
                // Get the next nibble in the key.
                uint8 nibble = uint8(path[pathPtr]);
                if (node[nibble].toBytes().length == 0) {
                    return false;
                }
                currentNodeRlp = node[nibble].toRlpBytes();
                // If not at the end of the proof, ensure consistency with the next node.
                if (i < proof.length - 1 && keccak256(currentNodeRlp) != keccak256(proof[i + 1].toRlpBytes())) {
                    return false;
                }
                pathPtr += 1;
            } else if (node.length == 2) {
                // -- Extension or Leaf node --
                bytes memory nodePath = _getNibbleArray(node[0].toBytes());
                uint prefix = _getHexPrefix(node[0].toBytes());
                uint shared = _nibblesEqual(path, pathPtr, nodePath);
                if (shared < nodePath.length) {
                    return false;
                }
                pathPtr += shared;
                if (isLeaf(prefix)) {
                    // Leaf node: all nibbles must have been consumed.
                    if (pathPtr != path.length) {
                        return false;
                    }
                    if (keccak256(node[1].toBytes()) == keccak256(value)) {
                        return true;
                    } else {
                        return false;
                    }
                } else {
                    // Extension node: continue traversal.
                    currentNodeRlp = node[1].toRlpBytes();
                    if (i < proof.length - 1 && keccak256(currentNodeRlp) != keccak256(proof[i + 1].toRlpBytes())) {
                        return false;
                    }
                }
            } else {
                // Invalid node type.
                return false;
            }
        }
        return false;
    }

    /// @dev Returns true if the hex-prefix indicates a leaf node.
    function isLeaf(uint prefix) private pure returns (bool) {
        return (prefix == 2 || prefix == 3);
    }

    /// @dev Extracts the hex prefix (the flag nibble) from a compact-encoded path.
    function _getHexPrefix(bytes memory compact) private pure returns (uint) {
        if (compact.length == 0) return 0;
        uint8 firstByte = uint8(compact[0]);
        // The lower nibble of the first byte contains the flag.
        return firstByte & 0x0F;
    }

    /**
     * @dev Converts a compact-encoded byte array into a nibble array.
     * See the Ethereum yellow paper for details on compact encoding.
     */
    function _getNibbleArray(bytes memory b) private pure returns (bytes memory) {
        if (b.length == 0) return "";
        uint8 offset;
        uint8 firstNibble = uint8(b[0]) >> 4;
        // If the flag indicates an odd-length path, skip the first nibble.
        if (firstNibble == 1 || firstNibble == 3) {
            offset = 1;
        } else {
            offset = 0;
        }
        uint nibblesLength = b.length * 2 - offset;
        bytes memory nibbles = new bytes(nibblesLength);
        uint nibbleIdx = 0;
        if (offset == 1) {
            // Use the lower nibble of the first byte.
            nibbles[0] = bytes1(uint8(b[0]) & 0x0F);
            nibbleIdx = 1;
        }
        for (uint i = 1; i < b.length; i++) {
            nibbles[nibbleIdx] = bytes1(uint8(b[i]) >> 4);
            nibbles[nibbleIdx + 1] = bytes1(uint8(b[i]) & 0x0F);
            nibbleIdx += 2;
        }
        return nibbles;
    }

    /**
     * @dev Compares two nibble arrays starting at a given index.
     * Returns the number of matching nibbles.
     */
    function _nibblesEqual(
        bytes memory a,
        uint start,
        bytes memory b
    ) private pure returns (uint) {
        uint i = 0;
        while (i < b.length && (start + i) < a.length) {
            if (a[start + i] != b[i]) break;
            i++;
        }
        return i;
    }
}
