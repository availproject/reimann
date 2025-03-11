// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import {Address} from "openzeppelin-contracts/contracts/utils/Address.sol";
import {Ownable2Step, Ownable} from "lib/openzeppelin-contracts/contracts/access/Ownable2Step.sol";
import {INexusSettler} from "./interfaces/INexusSettler.sol";
import "./interfaces/IErrors.sol";
import "./interfaces/IEvents.sol";
import "./interfaces/IStructs.sol";

contract NexusSettler is Ownable2Step, INexusSettler {
    using Address for address;
    /// @dev chainId => rollup contract address
    mapping(uint256 => Rollup) public rollups;
    /// @dev chainId => (state root => timestamp)
    mapping(uint256 => mapping (bytes32 => uint256)) public stateRoots;

    constructor(address governance) Ownable(governance) {}

    function createRollup(uint256 chainId, address rollupContract, address settlerContract, bytes4 stateRootFn) external onlyOwner {
        require(rollups[chainId].rollupContract == address(0), RollupAlreadyExists(chainId));
        require(rollupContract.code.length != 0, InvalidRollupContract(rollupContract));
        require(settlerContract != address(0), ZeroAddress());
        bytes memory result = rollupContract.functionStaticCall(abi.encodeWithSelector(stateRootFn));
        rollups[chainId] = Rollup(rollupContract, settlerContract, chainId, stateRootFn);
        bytes32 stateRoot = abi.decode(result, (bytes32));
        require(stateRoot != bytes32(0), InvalidStateRoot(chainId, stateRoot));
        stateRoots[chainId][stateRoot] = block.timestamp;
    }

    function updateRollupRoot(uint256 chainId) external {
        Rollup memory rollup = rollups[chainId];
        require(rollup.rollupContract != address(0), UnauthorizedRollup(chainId));
        bytes memory result = rollup.rollupContract.functionStaticCall(abi.encodeWithSelector(rollup.stateRootFn));
        bytes32 stateRoot = abi.decode(result, (bytes32));
        require(stateRoot != bytes32(0), InvalidStateRoot(chainId, stateRoot));
        stateRoots[chainId][stateRoot] = block.timestamp;
    }
}
