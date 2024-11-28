// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import "./interfaces/IErrors.sol";
import "./interfaces/IEvents.sol";
import "./interfaces/IStructs.sol";

contract NexusSettler {
    /// @dev chainId => rollup contract address
    mapping(uint256 => address) public rollups;
    /// @dev chainId => order root
    mapping(uint256 => bytes32) public orderRoots;
    /// @dev chainId => fill root
    mapping(uint256 => bytes32) public fillRoots;

    function createRollup(uint256 chainId, address rollup) external {
        require(rollups[chainId] == address(0), RollupAlreadyExists(chainId));
        rollups[chainId] = rollup;
    }

    function updateRollupOrderRoot(uint256 chainId, bytes32 root) external {
        require(rollups[chainId] != address(0), UnauthorizedRollup(chainId));
        orderRoots[chainId] = root;
    }

    function updateRollupFillRoot(uint256 chainId, bytes32 root) external {
        require(rollups[chainId] != address(0), UnauthorizedRollup(chainId));
        fillRoots[chainId] = root;
    }
}
