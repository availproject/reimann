// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import {Rollup} from "./IStructs.sol";

interface INexusSettler {
    function rollups(uint256 chainId) external view returns (Rollup memory rollup);
    function stateRoots(uint256 chainId, bytes32 stateRoot) external view returns (uint256 timestamp);
}
