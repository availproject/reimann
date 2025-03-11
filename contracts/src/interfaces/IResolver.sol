// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import {ERC20BridgeOrder, OnchainCrossChainOrder} from "./IStructs.sol";

interface IResolver {
    function resolve(ERC20BridgeOrder memory order, OnchainCrossChainOrder memory inner) external;
}
