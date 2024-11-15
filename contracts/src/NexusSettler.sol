// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import "./interfaces/IErrors.sol";
import "./interfaces/IEvents.sol";
import "./interfaces/IStructs.sol";

contract NexusSettler {
    /// @dev chainId => rollup contract address
    mapping(uint32 => address) public rollups;
    /// @dev chainId => order hash
    mapping(uint32 => bytes32) public orders;

    function createRollup(uint32 chainId, address rollup) external {
        require(rollups[chainId] == address(0), RollupAlreadyExists(chainId));
        rollups[chainId] = rollup;
    }

    function createOrder(OnchainCrossChainOrder calldata order) external {
        ERC20CrossChainOrder memory inner = abi.decode(order.orderData, (ERC20CrossChainOrder));
        require(order.fillDeadline > block.timestamp, OrderExpired(keccak256(abi.encode(order))));
        require(rollups[inner.destination] != address(0), UnauthorizedRollup(inner.destination));
        require(rollups[inner.source] != address(0), UnauthorizedRollup(inner.source));
        bytes32 orderHash = keccak256(abi.encode(order));
        require(orders[inner.source] == bytes32(0), OrderPendingOrFulfilled(orderHash));
        orders[inner.source] = orderHash;
        emit OrderSent(orderHash);
    }

    function release() external {
        // Implementation omitted
    }
}
