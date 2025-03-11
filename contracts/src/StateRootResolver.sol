// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import {INexusSettler} from "./interfaces/INexusSettler.sol";
import {StateProofVerifier} from "./lib/StateProofVerifier.sol";
import "./interfaces/IErrors.sol";
import "./interfaces/IEvents.sol";
import "./interfaces/IStructs.sol";

contract StateRootResolver {
    using StateProofVerifier for bytes32;
    INexusSettler public settler;

    constructor(INexusSettler _settler) {
        settler = _settler;
    }

    function dispute(uint256 chainId, bytes32 stateRoot, bytes[] calldata accountProof, bytes memory  accountRlp, bytes[] calldata storageProof, bytes32[] calldata orderProof, uint32 fillDeadline, IERC20 fromToken, IERC20 toToken, address sender, address recipient, uint256 amountIn, uint256 minAmountOut, uint256 source, uint32 orderNonce, address resolver) external {
        Rollup memory rollup = settler.rollups(chainId);
        require(settler.stateRoots(chainId, stateRoot) != 0, NonExistentStateRoot(chainId, stateRoot));
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
        bytes32 slot = keccak256(abi.encode(orderHash, uint256(1)));
        stateRoot.verifyStateProof(
            rollup.settlerContract,
            accountRlp,
            accountProof,
            slot,
            abi.encode(OrderStatus.FULFILLED),
            storageProof
        );
    }

    function getLeafHash(
        uint256 fillDeadline,
        bytes32 orderDataType,
        bytes memory orderData
    ) private pure returns (bytes32) {
        return
            keccak256(
                abi.encodePacked(
                    fillDeadline,
                    orderDataType,
                    orderData
                )
            );
    }
}
