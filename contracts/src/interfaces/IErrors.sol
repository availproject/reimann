// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

error UnauthorizedRollup(uint32 chainId);
error OrderExpired(bytes32 orderHash);
error OrderPendingOrFulfilled(bytes32 orderHash);
error OrderAlreadyFulfilled(bytes32 orderHash);
error OrderNotFulfilled(bytes32 orderHash);
error RollupAlreadyExists(uint32 chainId);
