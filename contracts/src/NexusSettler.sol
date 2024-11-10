// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import "./interfaces/IErrors.sol";
import "./interfaces/IEvents.sol";
import "./interfaces/IStructs.sol";

contract NexusSettler {
    /// @dev chainId => rollup contract address
    mapping(uint32 => address) public rollups;
    /// @dev chainId => order hash => fulfilment status
    mapping(uint32 => mapping(bytes32 => bool)) public orders;

    function requestFulfilment(OnchainCrossChainOrder memory order) external {
        // Implementation omitted
    }
}
