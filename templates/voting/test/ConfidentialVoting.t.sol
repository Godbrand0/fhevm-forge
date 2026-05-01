// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import { Test }               from "forge-std/Test.sol";
import { FhevmTest }          from "forge-fhevm/FhevmTest.sol";
import { ConfidentialVoting } from "../src/ConfidentialVoting.sol";
import { externalEuint64 }    from "encrypted-types/EncryptedTypes.sol";

contract ConfidentialVotingTest is FhevmTest {
    ConfidentialVoting voting;

    address admin  = makeAddr("admin");
    address voter1 = makeAddr("voter1");
    address voter2 = makeAddr("voter2");
    address voter3 = makeAddr("voter3");

    uint256 constant DURATION = 1 days;

    function setUp() public override {
        super.setUp();
        vm.prank(admin);
        voting = new ConfidentialVoting();

        // Register voters
        vm.startPrank(admin);
        voting.registerVoter(voter1);
        voting.registerVoter(voter2);
        voting.registerVoter(voter3);
        vm.stopPrank();
    }

    function test_create_proposal() public {
        vm.prank(admin);
        uint256 id = voting.createProposal("Should we adopt FHEVM?", DURATION);
        assertEq(id, 1);
        assertTrue(voting.isVotingActive(id));
    }

    function test_cast_vote_yes() public {
        vm.prank(admin);
        uint256 proposalId = voting.createProposal("Test proposal", DURATION);

        // Encrypt vote = 1 (yes)
        (externalEuint64 handle, bytes memory proof) =
            encryptUint64(1, address(voting), voter1);

        vm.prank(voter1);
        voting.castVote(proposalId, handle, proof);

        assertTrue(voting.hasVoted(proposalId, voter1));
    }

    function test_cannot_vote_twice() public {
        vm.prank(admin);
        uint256 proposalId = voting.createProposal("Test proposal", DURATION);

        (externalEuint64 h1, bytes memory p1) = encryptUint64(1, address(voting), voter1);
        vm.prank(voter1);
        voting.castVote(proposalId, h1, p1);

        (externalEuint64 h2, bytes memory p2) = encryptUint64(1, address(voting), voter1);
        vm.prank(voter1);
        vm.expectRevert("Already voted");
        voting.castVote(proposalId, h2, p2);
    }

    function test_unregistered_voter_cannot_vote() public {
        vm.prank(admin);
        uint256 proposalId = voting.createProposal("Test proposal", DURATION);

        address stranger = makeAddr("stranger");
        (externalEuint64 h, bytes memory p) = encryptUint64(1, address(voting), stranger);

        vm.prank(stranger);
        vm.expectRevert("Not a registered voter");
        voting.castVote(proposalId, h, p);
    }

    function test_tally_after_voting_ends() public {
        vm.prank(admin);
        uint256 proposalId = voting.createProposal("Test proposal", DURATION);

        // 2 yes votes, 1 no vote
        (externalEuint64 h1, bytes memory p1) = encryptUint64(1, address(voting), voter1);
        vm.prank(voter1); voting.castVote(proposalId, h1, p1);

        (externalEuint64 h2, bytes memory p2) = encryptUint64(1, address(voting), voter2);
        vm.prank(voter2); voting.castVote(proposalId, h2, p2);

        (externalEuint64 h3, bytes memory p3) = encryptUint64(0, address(voting), voter3);
        vm.prank(voter3); voting.castVote(proposalId, h3, p3);

        // Advance past voting period
        vm.warp(block.timestamp + DURATION + 1);
        assertFalse(voting.isVotingActive(proposalId));

        // Request tally — forge-fhevm resolves Gateway synchronously in tests
        uint256 requestId = voting.requestTally(proposalId);
        assertTrue(requestId > 0);
    }
}
