// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import {IERC20} from "lib/openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";

struct OnchainCrossChainOrder {
    /// @dev The timestamp by which the order must be filled on the destination chain
    uint32 fillDeadline;
    /// @dev Type identifier for the order data. This is an EIP-712 typehash.
    bytes32 orderDataType;
    /// @dev Arbitrary implementation-specific data
    /// Can be used to define tokens, amounts, destination chains, fees, settlement parameters,
    /// or any other order-type specific information
    bytes orderData;
}
    
struct ERC20BridgeOrder {
    /// @dev Source chain id
    uint256 source;
    /// @dev Destination chain id
    uint256 destination;
    /// @dev Token address to be sent on the source chain
    IERC20 tokenIn;
    /// @dev Token address to be received on the destination chain
    IERC20 tokenOut;
    /// @dev The address of the user that will send the tokens on the source chain
    address sender;
    /// @dev The address of the user that will receive the tokens on the destination chain
    address recipient;
    /// @dev The amount of tokens to be sent on the source chain
    uint256 amountIn;
    /// @dev The minimum amount of tokens to be received on the destination chain
    uint256 amountOutMin;
    /// @dev Nonce of the order
    uint32 nonce;
}

struct FillerInfo {
    /// @dev Source chain id
    uint256 source;
    /// @dev Destination chain id
    uint256 destination;
    /// @dev Token address to be sent on the source chain
    IERC20 tokenIn;
    /// @dev Token address to be received on the destination chain
    IERC20 tokenOut;
    /// @dev The address of the user that will send the tokens on the source chain
    address sender;
    /// @dev The address of the user that will receive the tokens on the destination chain
    address recipient;
    /// @dev The amount of tokens to be sent on the source chain
    uint256 amountIn;
    /// @dev The minimum amount of tokens to be received on the destination chain
    uint256 amountOutMin;
    /// @dev Nonce of the order
    uint32 nonce;
    /// @dev Filler address on source chain
    address filler;
}

struct EscrowInfo {
    /// @dev The address of the user that will send the tokens on the source chain
    address sender;
    /// @dev The address of the token locked
    IERC20 token;
    /// @dev The amount of tokens locked
    uint256 amount;
}

enum OrderStatus {
    EMPTY,
    PENDING,
    FULFILLED
}
