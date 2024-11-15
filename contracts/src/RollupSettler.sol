// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import {SafeERC20} from "lib/openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC20} from "lib/openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import "./interfaces/IErrors.sol";
import "./interfaces/IEvents.sol";
import "./interfaces/IStructs.sol";

contract RollupSettler {
    using SafeERC20 for IERC20;
    /// @dev chainId => authorisation status
    mapping(uint32 => bool) public authorizedRollups;
    /// @dev order hash => fulfilment status
    mapping(bytes32 => OrderStatus) public orders;
    /// @dev order hash => escrow info
    mapping(bytes32 => EscrowInfo) public escrows;

    function authorizeRollup(uint32 chainId) external {
        authorizedRollups[chainId] = true;
    }

    function unauthorizeRollup(uint32 chainId) external {
        authorizedRollups[chainId] = false;
    }

    function send(address recipient, IERC20 token, uint256 amount) external {
        token.safeTransfer(recipient, amount);
    }

    function requestFulfilment(OnchainCrossChainOrder calldata order) external {
        ERC20CrossChainOrder memory inner = abi.decode(order.orderData, (ERC20CrossChainOrder));
        require(authorizedRollups[inner.destination], UnauthorizedRollup(inner.destination));
        bytes32 orderHash = keccak256(abi.encode(order));
        require(order.fillDeadline > block.timestamp, OrderExpired(orderHash));
        require(orders[orderHash] == OrderStatus.EMPTY, OrderPendingOrFulfilled(orderHash));
        emit OrderSent(orderHash);
        escrows[orderHash] = EscrowInfo(inner.sender, inner.tokenIn, inner.amountIn);
        orders[orderHash] = OrderStatus.PENDING;
        inner.tokenIn.safeTransferFrom(msg.sender, address(this), inner.amountIn);
    }

    function fulfil(OnchainCrossChainOrder calldata order) external {
        ERC20CrossChainOrder memory inner = abi.decode(order.orderData, (ERC20CrossChainOrder));
        require(authorizedRollups[inner.source], UnauthorizedRollup(inner.source));
        bytes32 orderHash = keccak256(abi.encode(order));
        require(order.fillDeadline > block.timestamp, OrderExpired(orderHash));
        require(orders[orderHash] == OrderStatus.EMPTY, OrderPendingOrFulfilled(orderHash));
        emit OrderFulfilled(orderHash);
        orders[orderHash] = OrderStatus.FULFILLED;
    }

    function release() external {
        // Implementation omitted
    }
}
