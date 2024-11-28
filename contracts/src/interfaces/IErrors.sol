// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

error UnauthorizedRollup(uint256 chainId);
error InvalidOrder(bytes32 orderHash);
error OrderExpired(uint256 deadline);
error OrderPendingOrFulfilled(bytes32 orderHash);
error OrderAlreadyFulfilled(bytes32 orderHash);
error OrderNotFulfilled(bytes32 orderHash);
error RollupAlreadyExists(uint256 chainId);
error InvalidDeadline(uint256 deadline);
