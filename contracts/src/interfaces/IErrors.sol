// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

error InsufficientBond(uint256 amount);
error InvalidDeadline(uint256 deadline);
error InvalidOrder(bytes32 orderHash);
error InvalidRollupContract(address rollup);
error InvalidSignature();
error InvalidSigner();
error InvalidStateRoot(uint256 chainId, bytes32 stateRoot);
error NonExistentStateRoot(uint256 chainId, bytes32 stateRoot);
error OnlyController(address sender);
error OrderExpired(uint256 deadline);
error OrderPendingOrFulfilled(bytes32 orderHash);
error OrderAlreadyFulfilled(bytes32 orderHash);
error OrderNotFulfilled(bytes32 orderHash);
error RollupAlreadyExists(uint256 chainId);
error SignatureExpired(uint256 timestamp);
error UnauthorizedRollup(uint256 chainId);
error ZeroAddress();
