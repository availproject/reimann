// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import {ResolvedCrossChainOrder} from "./IStructs.sol";

event Evicted(address solver);
/// @notice Signals that an order has been opened
/// @param orderId a unique order identifier within this settlement system
/// @param resolvedOrder resolved order that would be returned by resolve if called instead of Open
event Open(bytes32 indexed orderId, ResolvedCrossChainOrder resolvedOrder);
event OrderSent(bytes32 orderHash);
event OrderFilled(bytes32 orderHash);
event Slashed(address solver, uint256 amount);
event Unbonded(address solver);
event UnbondingStarted(address solver, uint256 exitTimestamp);
