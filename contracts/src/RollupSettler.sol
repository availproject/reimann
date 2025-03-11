// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import {SafeERC20} from "lib/openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC20} from "lib/openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {DepositContract} from "./lib/DepositContract.sol";
import {RollupFiller} from "./RollupFiller.sol";
import "./interfaces/IErrors.sol";
import "./interfaces/IEvents.sol";
import "./interfaces/IStructs.sol";

contract RollupSettler is DepositContract {
    using SafeERC20 for IERC20;
    /// @dev chainId => authorisation status
    mapping(uint256 => bool) public authorizedRollups;
    /// @dev order hash => fulfilment status
    mapping(bytes32 => OrderStatus) public orders;
    /// @dev order hash => escrow info
    mapping(bytes32 => EscrowInfo) public escrows;
    /// @dev chainId => sent order roots
    mapping(uint256 => bytes32) public orderRoots;
    /// @dev chainId => filled order roots
    mapping(uint256 => bytes32) public fillRoots;

    uint256 private constant MAX_ORDER_LIFETIME = 1 hours;
    bytes32 public orderRoot;
    bytes32 public fillRoot;
    RollupFiller public filler;

    constructor() {
        filler = new RollupFiller();
    }

    function authorizeRollup(uint256 chainId) external {
        authorizedRollups[chainId] = true;
    }

    function unauthorizeRollup(uint256 chainId) external {
        authorizedRollups[chainId] = false;
    }

    function send(uint32 fillDeadline, IERC20 fromToken, IERC20 toToken, address recipient, uint256 amountIn, uint256 minAmountOut, uint256 destination, address resolver) external {
        require(authorizedRollups[destination], UnauthorizedRollup(destination));
        require(fillDeadline > block.timestamp && fillDeadline <= block.timestamp + MAX_ORDER_LIFETIME, InvalidDeadline(fillDeadline));
        bytes memory orderData = abi.encode(ERC20BridgeOrder({
                source: block.chainid,
                destination: destination,
                tokenIn: fromToken,
                tokenOut: toToken,
                sender: msg.sender,
                recipient: recipient,
                amountIn: amountIn,
                amountOutMin: minAmountOut,
                nonce: depositCount,
                resolver: resolver
            }));
        bytes32 orderHash = getLeafHash(fillDeadline, keccak256("ERC20BridgeOrder"), orderData);
        orderRoot = _deposit(orderHash); // depositCount gets incremented here
        emit OrderSent(orderHash);
        fromToken.safeTransferFrom(msg.sender, address(this), amountIn);
    }

    function fulfil(uint32 fillDeadline, IERC20 fromToken, IERC20 toToken, address sender, address recipient, uint256 amountIn, uint256 minAmountOut, uint256 source, uint32 orderNonce, address resolver, bytes32[_DEPOSIT_CONTRACT_TREE_DEPTH] calldata smtProof) external {
        require(authorizedRollups[source], UnauthorizedRollup(source));
        require(fillDeadline >= block.timestamp, OrderExpired(fillDeadline));
        bytes memory orderData = abi.encode(ERC20BridgeOrder({
                source: source,
                destination: block.chainid,
                tokenIn: fromToken,
                tokenOut: toToken,
                sender: sender,
                recipient: recipient,
                amountIn: amountIn,
                amountOutMin: minAmountOut,
                nonce: orderNonce,
                resolver: resolver
        }));
        bytes32 orderHash = getLeafHash(fillDeadline, keccak256("ERC20BridgeOrder"), orderData);
        require(verifyMerkleProof(orderHash, smtProof, orderNonce, orderRoots[source]), InvalidOrder(orderHash));
        fillRoot = filler.deposit(orderHash);
        emit OrderFilled(orderHash);
        toToken.safeTransferFrom(msg.sender, recipient, minAmountOut);
    }

    function claim() external {

    }

    function release() external {

    }

    function updateRollupOrderRoot(uint256 chainId, bytes32 root) external {
        require(authorizedRollups[chainId], UnauthorizedRollup(chainId));
        orderRoots[chainId] = root;
    }

    function updateRollupFillRoot(uint256 chainId, bytes32 root) external {
        require(authorizedRollups[chainId], UnauthorizedRollup(chainId));
        fillRoots[chainId] = root;
    }
}
