// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;
import {Test} from "forge-std/Test.sol";
import {console} from "forge-std/console.sol";
import {DepositContract} from "src/lib/DepositContract.sol";

contract DepositContractTest is Test {
    DepositContractUser depositContractUser;

    function setUp() external {
        depositContractUser = new DepositContractUser();
    }

    function test_addLeaf() external {
        depositContractUser.deposit(0x0000000000000000000000000000000000000000000000000000000000000001);
        depositContractUser.deposit(0x0000000000000000000000000000000000000000000000000000000000000002);
        depositContractUser.deposit(0x0000000000000000000000000000000000000000000000000000000000000003);
        console.logBytes32(depositContractUser.getDepositRoot());
    }
}

contract DepositContractUser is DepositContract {
    function deposit(bytes32 leafHash) external {
        _deposit(leafHash);
    }
}
