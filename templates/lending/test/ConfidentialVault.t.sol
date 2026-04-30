// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import { FhevmTest }             from "forge-fhevm/FhevmTest.sol";
import { ConfidentialVault }      from "../src/ConfidentialVault.sol";
import { ConfidentialCollateral } from "../src/tokens/ConfidentialCollateral.sol";
import { ConfidentialDebt }       from "../src/tokens/ConfidentialDebt.sol";
import { externalEuint64, ebool } from "encrypted-types/EncryptedTypes.sol";

contract ConfidentialVaultTest is FhevmTest {
    ConfidentialVault      vault;
    ConfidentialCollateral collateral;
    ConfidentialDebt       debt;

    address borrower = makeAddr("borrower");
    address lender   = makeAddr("lender");
    address agent    = makeAddr("agent");

    function setUp() public override {
        super.setUp(); // deploys all forge-fhevm host contracts

        collateral = new ConfidentialCollateral();
        debt       = new ConfidentialDebt();
        vault      = new ConfidentialVault(address(collateral), address(debt));

        collateral.setVault(address(vault));
        debt.setVault(address(vault));
    }

    function test_borrow_opens_position() public {
        uint64 borrowAmountUsdc = 2000_000000; // $2000

        (externalEuint64 encHandle, bytes memory inputProof) =
            encryptUint64(borrowAmountUsdc, borrower, address(vault));

        vm.deal(borrower, 1.5 ether);
        vm.prank(borrower);
        vault.borrow{value: 1.5 ether}(encHandle, inputProof);

        address[] memory positions = vault.getActivePositions();
        assertEq(positions.length, 1);
        assertEq(positions[0], borrower);
    }

    function test_get_position_handles_returns_exactly_two_values() public {
        test_borrow_opens_position();

        (bytes32 collHandle, bytes32 debtHandle) = vault.getPositionHandles(borrower);
        assertTrue(collHandle != bytes32(0), "Collateral handle should be non-zero");
        assertTrue(debtHandle  != bytes32(0), "Debt handle should be non-zero");
    }

    function test_has_no_pending_check_initially() public {
        test_borrow_opens_position();
        assertFalse(vault.hasPendingCheck(borrower));
    }

    function test_health_check_request_marks_handle() public {
        test_borrow_opens_position();

        vm.prank(agent);
        ebool unhealthyHandle = vault.requestHealthCheck(borrower);
        assertTrue(ebool.unwrap(unhealthyHandle) != bytes32(0));
        assertTrue(vault.hasPendingCheck(borrower));
    }

    function test_health_check_execute_healthy_position() public {
        test_borrow_opens_position();

        vm.prank(agent);
        ebool unhealthyHandle = vault.requestHealthCheck(borrower);

        bytes32[] memory handles = new bytes32[](1);
        handles[0] = ebool.unwrap(unhealthyHandle);

        (uint256[] memory cleartexts, bytes memory proof) = publicDecrypt(handles);
        vault.executeHealthCheck(borrower, handles, abi.encode(cleartexts), proof);

        // Well-collateralized position should not be liquidated
        (bool active, ) = vault.getLoanInfo(borrower);
        assertTrue(active);
        assertFalse(vault.hasPendingCheck(borrower));
    }

    function test_grant_agent_access() public {
        test_borrow_opens_position();

        vault.grantAgentAccess(borrower, agent);

        (bytes32 collHandle, ) = vault.getPositionHandles(borrower);
        assertTrue(collHandle != bytes32(0), "Should have non-zero collateral handle");
    }

    function test_loan_info_returns_active() public {
        test_borrow_opens_position();

        (bool active, uint256 openedAt) = vault.getLoanInfo(borrower);
        assertTrue(active);
        assertGt(openedAt, 0);
    }
}
