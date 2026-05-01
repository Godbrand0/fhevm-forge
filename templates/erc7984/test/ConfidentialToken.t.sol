// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import { Test }              from "forge-std/Test.sol";
import { FHEVMTestBase }     from "forge-fhevm/FHEVMTestBase.sol";
import { ConfidentialToken } from "../src/ConfidentialToken.sol";

contract ConfidentialTokenTest is FhevmTest {
    ConfidentialToken token;

    address alice = makeAddr("alice");
    address bob   = makeAddr("bob");

    function setUp() public override {
        super.setUp();
        token = new ConfidentialToken("Confidential USD", "cUSD", 6, 1_000_000_000000);
    }

    function test_initial_supply_goes_to_deployer() public {
        uint256 handle = token.balanceOf(address(this));
        uint64 balance = decryptUint64(euint64.wrap(bytes32(handle)));
        assertEq(balance, 1_000_000_000000);
    }

    function test_transfer_reduces_sender_balance() public {
        uint64 transferAmount = 100_000000;

        (bytes32 handle, bytes memory proof) =
            encryptUint64(transferAmount, address(token), address(this));
        token.transfer(alice, einput.wrap(handle), proof);

        uint64 aliceBalance = decryptUint64(euint64.wrap(bytes32(token.balanceOf(alice))));
        assertEq(aliceBalance, transferAmount);
    }

    function test_approve_and_transferFrom() public {
        // Approve alice to spend 500 tokens
        uint64 approveAmount = 500_000000;
        (externalEuint32 h1, bytes memory p1) = encryptUint64(approveAmount, address(token), address(this));
        token.approve(alice, einput.wrap(h1), p1);

        // Alice transfersFrom on behalf of deployer → bob
        uint64 transferAmount = 200_000000;
        (externalEuint32 h2, bytes memory p2) = encryptUint64(transferAmount, address(token), alice);
        vm.prank(alice);
        token.transferFrom(address(this), bob, einput.wrap(h2), p2);

        uint64 bobBalance = decryptUint64(euint64.wrap(bytes32(token.balanceOf(bob))));
        assertEq(bobBalance, transferAmount);
    }

    function test_mint_increases_total_supply() public {
        uint64 before = token.totalSupply();
        token.mint(alice, 1_000_000000);
        assertEq(token.totalSupply(), before + 1_000_000000);
    }
}
