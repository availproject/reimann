// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import {ERC20, ERC20Permit} from "lib/openzeppelin-contracts/contracts/token/ERC20/extensions/ERC20Permit.sol";

contract MockERC20 is ERC20Permit {
    constructor() ERC20("Test ERC20", "TERC20") ERC20Permit("Test ERC20") {
    }

    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }

    function burn(address from, uint256 amount) external {
        _burn(from, amount);
    }
}
