// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import {ERC20Permit} from "openzeppelin-contracts/contracts/token/ERC20/extensions/ERC20Permit.sol";

contract MockERC20 is ERC20Permit {
    constructor(string memory name, string memory symbol) ERC20Permit(name, symbol) {
        _mint(msg.sender, 1000000 * 10 ** decimals());
    }
}
