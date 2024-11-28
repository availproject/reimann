// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import {DepositContract} from "./lib/DepositContract.sol";
import "./interfaces/IErrors.sol";
import "./interfaces/IEvents.sol";
import "./interfaces/IStructs.sol";

contract RollupFiller is DepositContract {
    function deposit(bytes32 leaf) external returns (bytes32 root) {
        return _deposit(leaf);
    }
}
