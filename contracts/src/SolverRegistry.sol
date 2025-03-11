// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import {SafeERC20} from "lib/openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC20} from "lib/openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {Ownable2Step, Ownable} from "lib/openzeppelin-contracts/contracts/access/Ownable2Step.sol";
import {MessageHashUtils} from "lib/openzeppelin-contracts/contracts/utils/cryptography/MessageHashUtils.sol";
import {ECDSA} from "lib/openzeppelin-contracts/contracts/utils/cryptography/ECDSA.sol";
import {EIP712} from "lib/openzeppelin-contracts/contracts/utils/cryptography/EIP712.sol";
import {Nonces} from "lib/openzeppelin-contracts/contracts/utils/Nonces.sol";
import {Math} from "lib/openzeppelin-contracts/contracts/utils/math/Math.sol";
import {Solver} from "./interfaces/IStructs.sol";
import "./interfaces/IErrors.sol";
import "./interfaces/IEvents.sol";
import "./interfaces/IStructs.sol";

contract SolverRegistry is Ownable2Step, EIP712, Nonces {
    using MessageHashUtils for bytes;
    using ECDSA for bytes32;
    using SafeERC20 for IERC20;
    using Math for uint256;

    IERC20 public token;
    uint256 public minBond;
    uint256 public nonce;
    uint256 public slashingPercentage;
    uint256 public unbondingPeriod;
    mapping(address => Solver) public solvers;
    mapping(address => UnbondingSolver) public unbondingSolvers;

    bytes32 private constant SOLVER_TYPEHASH = keccak256("SolverSignature(address controller,uint256 chainId,uint256 deadline,uint256 nonce)");

    constructor(string memory name, IERC20 _token, uint256 _minBond, uint256 _slashingPercentage, uint256 _unbondingPeriod, address governance) Ownable(governance) EIP712(name, "1") {
        token = _token;
        minBond = _minBond;
        slashingPercentage = _slashingPercentage;
        unbondingPeriod = _unbondingPeriod;
    }

    function updateMinBond(uint256 _minBond) external onlyOwner {
        minBond = _minBond;
    }

    function updateToken(IERC20 _token) external onlyOwner {
        token = _token;
    }

    function updateController(address solver, address controller) external {
        require(solvers[solver].controller == msg.sender, OnlyController(msg.sender));
        require(controller != address(0), InvalidSigner());
        solvers[solver].controller = controller;
    }

    function bond(address solver, uint256 deadline, uint256 amount, bytes calldata signature) external {
        require(solver != address(0), InvalidSigner());
        require(amount >= minBond, InsufficientBond(amount));
        bytes32 structHash = keccak256(abi.encode(SOLVER_TYPEHASH, solver, msg.sender, block.chainid, deadline, _useNonce(solver)));
        if (deadline < block.timestamp) {
            revert SignatureExpired(block.timestamp);
        }
        bytes32 messageHash = _hashTypedDataV4(structHash);
        address signer = messageHash.recover(signature);
        require(signer == solver, InvalidSignature());
        solvers[solver] = Solver(msg.sender, amount, false);
        token.safeTransferFrom(msg.sender, address(this), amount);
    }

    function unbond(address solver) external {
        Solver memory solverInfo = solvers[solver];
        require(solverInfo.controller == msg.sender, OnlyController(msg.sender));
        uint256 exitTimestamp = block.timestamp + unbondingPeriod;
        emit UnbondingStarted(solver, exitTimestamp);
        unbondingSolvers[solver] = UnbondingSolver(msg.sender, solverInfo.amount, exitTimestamp);
        delete solvers[solver];
    }

    function exit(address solver) external {
        UnbondingSolver memory unbondingSolver = unbondingSolvers[solver];
        require(block.timestamp >= unbondingSolver.exitTimestamp, InvalidDeadline(unbondingSolver.exitTimestamp));
        emit Unbonded(solver);
        delete unbondingSolvers[solver];
        token.safeTransfer(unbondingSolver.controller, unbondingSolver.amount);
    }

    function slash(address solver) external {
        Solver memory solverInfo = solvers[solver];
        require(solverInfo.controller == msg.sender, OnlyController(msg.sender));
        uint256 slashAmount = minBond.mulDiv(slashingPercentage, 100, Math.Rounding.Ceil);
        uint256 newAmount = solverInfo.amount - slashAmount;
        bool toBeEvicted = newAmount < minBond;
        if (toBeEvicted) {
            // evict solver if bond falls below minimum
            delete solvers[solver];
        } else {
            solvers[solver].amount = newAmount;
        }
        emit Slashed(solver, slashAmount);
        token.safeTransfer(owner(), slashAmount);
        if (toBeEvicted) { // used to preseve CEI pattern
            emit Evicted(solver);
            token.safeTransfer(solverInfo.controller, newAmount);
        }
    }
}
