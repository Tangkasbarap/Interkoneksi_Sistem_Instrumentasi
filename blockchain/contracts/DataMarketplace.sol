// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
import "@openzeppelin/contracts/access/Ownable.sol";

contract DataMarketplace is Ownable {
    uint256 public accessPrice;
    mapping(address => bool) public hasAccess;
    event AccessGranted(address indexed buyer, uint256 amountPaid);

    constructor() Ownable(msg.sender) {
        accessPrice = 0.01 ether;
    }

    function purchaseAccess() public payable {
        require(msg.value >= accessPrice, "Pembayaran tidak cukup!");
        hasAccess[msg.sender] = true;
        emit AccessGranted(msg.sender, msg.value);
    }

    function withdrawFunds() public onlyOwner {
        (bool success, ) = owner().call{value: address(this).balance}("");
        require(success, "Gagal menarik dana.");
    }
}
