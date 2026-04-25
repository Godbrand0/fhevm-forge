// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import { Test }          from "forge-std/Test.sol";
import { FHEVMTestBase } from "forge-fhevm/FHEVMTestBase.sol";
import { Counter }       from "../src/Counter.sol";

contract CounterTest is FHEVMTestBase {
    Counter counter;

    function setUp() public override {
        super.setUp();
        counter = new Counter();
    }

    function test_add_increases_encrypted_count() public {
        address user = makeAddr("user");
        (bytes32 handle, bytes memory proof) =
            encryptUint64(10, address(counter), user);

        vm.prank(user);
        counter.add(einput.wrap(handle), proof);

        uint256 resultHandle = counter.getHandle();
        // Decrypt using forge-fhevm test helper (only works in tests)
        uint64 decrypted = decryptUint64(euint64.wrap(bytes32(resultHandle)));
        assertEq(decrypted, 10);
    }

    function test_add_multiple_increments() public {
        address user = makeAddr("user");

        (bytes32 h1, bytes memory p1) = encryptUint64(5, address(counter), user);
        vm.prank(user);
        counter.add(einput.wrap(h1), p1);

        (bytes32 h2, bytes memory p2) = encryptUint64(3, address(counter), user);
        vm.prank(user);
        counter.add(einput.wrap(h2), p2);

        uint64 decrypted = decryptUint64(euint64.wrap(bytes32(counter.getHandle())));
        assertEq(decrypted, 8);
    }
}
