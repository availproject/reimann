// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import "./interfaces/IErrors.sol";
import "./interfaces/IEvents.sol";
import "./interfaces/IStructs.sol";

contract RollupSettler {
    /// @dev chainId => authorisation status
    mapping(uint32 => bool) public authorizedRollups;
    /// @dev order hash => fulfilment status
    mapping(bytes32 => OrderStatus) public orders;

    function authorizeRollup(uint32 chainId) external {
        authorizedRollups[chainId] = true;
    }

    function unauthorizeRollup(uint32 chainId) external {
        authorizedRollups[chainId] = false;
    }

    function requestFulfilment(OnchainCrossChainOrder calldata order) external {
        (ERC20CrossChainOrder memory inner) = abi.decode(order.orderData, (ERC20CrossChainOrder));
        require(authorizedRollups[inner.destination], UnauthorizedRollup(inner.destination));
        bytes32 orderHash = keccak256(abi.encode(order));
        require(order.fillDeadline > block.timestamp, OrderExpired(orderHash));
        require(orders[orderHash] == OrderStatus.EMPTY, OrderPendingOrFulfilled(orderHash));
        emit OrderSent(orderHash);
        orders[orderHash] = OrderStatus.PENDING;
    }

    function fulfil(OnchainCrossChainOrder calldata order) external {
        (ERC20CrossChainOrder memory inner) = abi.decode(order.orderData, (ERC20CrossChainOrder));
        require(authorizedRollups[inner.source], UnauthorizedRollup(inner.source));
        bytes32 orderHash = keccak256(abi.encode(order));
        require(order.fillDeadline > block.timestamp, OrderExpired(orderHash));
        require(orders[orderHash] == OrderStatus.EMPTY, OrderPendingOrFulfilled(orderHash));
        emit OrderFulfilled(orderHash);
        orders[orderHash] = OrderStatus.FULFILLED;
    }
}
