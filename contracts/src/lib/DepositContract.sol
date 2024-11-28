// SPDX-License-Identifier: AGPL-3.0
pragma solidity ^0.8.28;
import {console} from "forge-std/console.sol";
import "../interfaces/IStructs.sol";

/**
 * Based on the implementation of the deposit eth2.0 contract https://github.com/ethereum/consensus-specs/blob/dev/solidity_deposit_contract/deposit_contract.sol
 */
contract DepositContract {
    /**
     * @dev Thrown when the merkle tree is full
     */
    error MerkleTreeFull();

    // Merkle tree levels
    uint256 internal constant _DEPOSIT_CONTRACT_TREE_DEPTH = 32;

    // This ensures `depositCount` will fit into 32-bits
    uint256 internal constant _MAX_DEPOSIT_COUNT =
        2 ** _DEPOSIT_CONTRACT_TREE_DEPTH - 1;

    // Branch array which contains the necessary sibilings to compute the next root when a new
    // leaf is inserted
    bytes32[_DEPOSIT_CONTRACT_TREE_DEPTH] internal _branch;

    // Counter of current deposits
    uint32 public depositCount;
    
    /**
     * @notice Computes and returns the merkle root
     */
    function getDepositRoot() public view returns (bytes32) {
        bytes32 node;
        uint256 size = depositCount;
        bytes32 currentZeroHashHeight = 0;

        for (
            uint256 height = 0;
            height < _DEPOSIT_CONTRACT_TREE_DEPTH;
            height++
        ) {
            if (((size >> height) & 1) == 1)
                node = keccak256(abi.encodePacked(_branch[height], node));
            else
                node = keccak256(abi.encodePacked(node, currentZeroHashHeight));
            console.log("node");
            console.logBytes32(node);
            currentZeroHashHeight = keccak256(
                abi.encodePacked(currentZeroHashHeight, currentZeroHashHeight)
            );
            console.log("currentZeroHashHeight");
            console.logBytes32(currentZeroHashHeight);
        }
        return node;
    }

    /**
     * @notice Add a new leaf to the merkle tree
     * @param leafHash Leaf hash
     */
    function _deposit(bytes32 leafHash) internal returns (bytes32 root) {
        bytes32 node = leafHash;

        // Add deposit data root to Merkle tree (update a single `_branch` node)
        uint256 size = ++depositCount;
        for (
            uint256 height = 0;
            height < _DEPOSIT_CONTRACT_TREE_DEPTH;
            height++
        ) {
            if (((size >> height) & 1) == 1) {
                _branch[height] = node;
                return getDepositRoot();
            }
            node = keccak256(abi.encodePacked(_branch[height], node));
        }
        // As the loop should always end prematurely with the `return` statement,
        // this code should be unreachable. We assert `false` just to be safe.
        assert(false);
    }

    /**
     * @notice Verify merkle proof
     * @param leafHash Leaf hash
     * @param smtProof Smt proof
     * @param index Index of the leaf
     * @param root Merkle root
     */
    function verifyMerkleProof(
        bytes32 leafHash,
        bytes32[_DEPOSIT_CONTRACT_TREE_DEPTH] calldata smtProof,
        uint32 index,
        bytes32 root
    ) public pure returns (bool) {
        bytes32 node = leafHash;

        // Check merkle proof
        for (
            uint256 height = 0;
            height < _DEPOSIT_CONTRACT_TREE_DEPTH;
            height++
        ) {
            if (((index >> height) & 1) == 1)
                node = keccak256(abi.encodePacked(smtProof[height], node));
            else node = keccak256(abi.encodePacked(node, smtProof[height]));
        }

        return node == root;
    }

    function getLeafHash(
        uint256 fillDeadline,
        bytes32 orderDataType,
        bytes memory orderData
    ) public pure returns (bytes32) {
        return
            keccak256(
                abi.encodePacked(
                    fillDeadline,
                    orderDataType,
                    orderData
                )
            );
    }
}
