// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

/// @title BridgeToken
/// @notice ERC20 representation of a token minted by the OtterBridge on the
/// destination side of a cross-chain lock/mint setup. The bridge is the only
/// address allowed to mint or burn.
contract BridgeToken is ERC20, Ownable {
    address public bridge;

    error NotBridge();

    modifier onlyBridge() {
        if (msg.sender != bridge) revert NotBridge();
        _;
    }

    constructor(string memory name_, string memory symbol_, address bridge_) ERC20(name_, symbol_) Ownable(msg.sender) {
        bridge = bridge_;
    }

    function mint(address to, uint256 amount) external onlyBridge {
        _mint(to, amount);
    }

    function burn(address from, uint256 amount) external onlyBridge {
        _burn(from, amount);
    }
}
