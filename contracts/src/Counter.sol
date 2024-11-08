// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

contract IntentManager {
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
    struct ERC20CrossChainOrder {
        /// @dev Source chain id
        uint32 sourceChainId;
        /// @dev Destination chain id
        uint32 destinationChainId;
        /// @dev Token address to be sent on the source chain
        address tokenIn;
        /// @dev Token address to be received on the destination chain
        address tokenOut;
        /// @dev The address of the user that will send the tokens on the source chain
        address sender;
        /// @dev The address of the user that will receive the tokens on the destination chain
        address recipient;
        /// @dev The amount of tokens to be sent on the source chain
        uint256 amountIn;
        /// @dev The minimum amount of tokens to be received on the destination chain
        uint256 amountOutMin;
        /// @dev Nonce of the order
        uint256 nonce;
    }

    mapping(uint32 => address) public chainIdToContract;


}
