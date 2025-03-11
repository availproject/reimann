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

/// @title GaslessCrossChainOrder CrossChainOrder type
/// @notice Standard order struct to be signed by users, disseminated to fillers, and submitted to origin settler contracts by fillers
struct GaslessCrossChainOrder {
	/// @dev The contract address that the order is meant to be settled by.
	/// Fillers send this order to this contract address on the origin chain
	address originSettler;
	/// @dev The address of the user who is initiating the swap,
	/// whose input tokens will be taken and escrowed
	address user;
	/// @dev Nonce to be used as replay protection for the order
	uint256 nonce;
	/// @dev The chainId of the origin chain
	uint256 originChainId;
	/// @dev The timestamp by which the order must be opened
	uint32 openDeadline;
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
    /// @dev Address of resolver on L1
    address resolver;
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

struct Solver {
    /// @dev The address of the controller
    address controller;
    /// @dev The amount of tokens locked
    uint256 amount;
    /// @dev If solver is trusted
    bool trusted;
}

struct UnbondingSolver {
    /// @dev The address of the controller
    address controller;
    /// @dev The amount of tokens locked
    uint256 amount;
    /// @dev The timestamp by which the unbonding period ends
    uint256 exitTimestamp;
}

struct SolverSignature {
    /// @dev The address of the solver bonding
    address solver;
    /// @dev The address of the controller
    address controller;
    /// @dev Chain ID
    uint256 chainId;
    /// @dev Nonce
    uint256 nonce;
    /// @dev Deadline of the signature
    uint256 deadline;
}

struct Rollup {
    /// @dev The address of the rollup contract
    address rollupContract;
    /// @dev The address of the settler contract on L2
    address settlerContract;
    /// @dev Chain ID
    uint256 chainId;
    /// @dev Function signature for fetching state root
    bytes4 stateRootFn;
}

/// @title ResolvedCrossChainOrder type
/// @notice An implementation-generic representation of an order intended for filler consumption
/// @dev Defines all requirements for filling an order by unbundling the implementation-specific orderData.
/// @dev Intended to improve integration generalization by allowing fillers to compute the exact input and output information of any order
struct ResolvedCrossChainOrder {
	/// @dev The address of the user who is initiating the transfer
	address user;
	/// @dev The chainId of the origin chain
	uint256 originChainId;
	/// @dev The timestamp by which the order must be opened
	uint32 openDeadline;
	/// @dev The timestamp by which the order must be filled on the destination chain(s)
	uint32 fillDeadline;
	/// @dev The unique identifier for this order within this settlement system
	bytes32 orderId;

	/// @dev The max outputs that the filler will send. It's possible the actual amount depends on the state of the destination
	///      chain (destination dutch auction, for instance), so these outputs should be considered a cap on filler liabilities.
	Output[] maxSpent;
	/// @dev The minimum outputs that must be given to the filler as part of order settlement. Similar to maxSpent, it's possible
	///      that special order types may not be able to guarantee the exact amount at open time, so this should be considered
	///      a floor on filler receipts. Setting the `recipient` of an `Output` to address(0) indicates that the filler is not
        ///      known when creating this order.
	Output[] minReceived;
	/// @dev Each instruction in this array is parameterizes a single leg of the fill. This provides the filler with the information
	///      necessary to perform the fill on the destination(s).
	FillInstruction[] fillInstructions;
}

/// @notice Tokens that must be received for a valid order fulfillment
struct Output {
	/// @dev The address of the ERC20 token on the destination chain
	/// @dev address(0) used as a sentinel for the native token
	bytes32 token;
	/// @dev The amount of the token to be sent
	uint256 amount;
	/// @dev The address to receive the output tokens
	bytes32 recipient;
	/// @dev The destination chain for this output
	uint256 chainId;
}

/// @title FillInstruction type
/// @notice Instructions to parameterize each leg of the fill
/// @dev Provides all the origin-generated information required to produce a valid fill leg
struct FillInstruction {
	/// @dev The chain that this instruction is intended to be filled on
	uint256 destinationChainId;
	/// @dev The contract address that the instruction is intended to be filled on
	bytes32 destinationSettler;
	/// @dev The data generated on the origin chain needed by the destinationSettler to process the fill
	bytes originData;
}
