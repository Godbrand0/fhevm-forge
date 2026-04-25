// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import { Test }                   from "forge-std/Test.sol";
import { FHEVMTestBase }          from "forge-fhevm/FHEVMTestBase.sol";
import { ConfidentialVault }      from "../src/ConfidentialVault.sol";
import { ConfidentialCollateral } from "../src/tokens/ConfidentialCollateral.sol";
import { ConfidentialDebt }       from "../src/tokens/ConfidentialDebt.sol";
import { TFHE }                   from "fhevm/lib/TFHE.sol";

contract ConfidentialVaultTest is FHEVMTestBase {
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

        (bytes32 handle, bytes memory inputProof) =
            encryptUint64(borrowAmountUsdc, address(vault), borrower);

        vm.prank(borrower);
        vm.deal(borrower, 1.5 ether);
        vault.borrow{value: 1.5 ether}(einput.wrap(handle), inputProof);

        address[] memory positions = vault.getActivePositions();
        assertEq(positions.length, 1);
        assertEq(positions[0], borrower);
    }

    function test_get_position_handles_returns_exactly_two_values() public {
        test_borrow_opens_position();

        (uint256 collHandle, uint256 debtHandle) = vault.getPositionHandles(borrower);
        assertTrue(collHandle != 0, "Collateral handle should be non-zero");
        assertTrue(debtHandle  != 0, "Debt handle should be non-zero");
        // There is no third handle here — health handle is separate
    }

    function test_has_no_pending_check_initially() public {
        test_borrow_opens_position();
        assertFalse(vault.hasPendingCheck(borrower));
    }

    function test_health_check_request_triggers_gateway() public {
        test_borrow_opens_position();

        vm.prank(agent);
        uint256 requestId = vault.requestHealthCheck(borrower);
        assertTrue(requestId > 0);
        // forge-fhevm resolves Gateway callbacks synchronously in tests
    }

    function test_grant_agent_access() public {
        test_borrow_opens_position();

        vault.grantAgentAccess(borrower, agent);

        // After grantAgentAccess, agent can reencrypt the position handles
        (uint256 collHandle, ) = vault.getPositionHandles(borrower);
        assertTrue(collHandle != 0, "Should have non-zero collateral handle");
    }

    function test_loan_info_returns_active() public {
        test_borrow_opens_position();

        (bool active, uint256 openedAt) = vault.getLoanInfo(borrower);
        assertTrue(active);
        assertGt(openedAt, 0);
    }
}
