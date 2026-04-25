// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import { Test }          from "forge-std/Test.sol";
import { FHEVMTestBase } from "forge-fhevm/FHEVMTestBase.sol";
import { BlindAuction }  from "../src/BlindAuction.sol";

contract BlindAuctionTest is FHEVMTestBase {
    BlindAuction auction;

    address seller = makeAddr("seller");
    address bidder1 = makeAddr("bidder1");
    address bidder2 = makeAddr("bidder2");

    uint64 constant START_PRICE   = 1_000_000000; // $1000 USDC
    uint64 constant RESERVE_PRICE = 500_000000;   // $500  USDC
    uint256 constant DURATION     = 1 days;

    function setUp() public override {
        super.setUp();
        auction = new BlindAuction();
    }

    function test_create_auction() public {
        vm.prank(seller);
        uint256 auctionId = auction.createAuction(START_PRICE, RESERVE_PRICE, DURATION);
        assertEq(auctionId, 1);
        assertTrue(auction.isAuctionActive(auctionId));
    }

    function test_submit_encrypted_bid() public {
        vm.prank(seller);
        uint256 auctionId = auction.createAuction(START_PRICE, RESERVE_PRICE, DURATION);

        uint64 bidAmount = 750_000000; // $750
        (bytes32 handle, bytes memory proof) =
            encryptUint64(bidAmount, address(auction), bidder1);

        vm.prank(bidder1);
        auction.submitBid(auctionId, einput.wrap(handle), proof);

        // Bid handle should be stored and non-zero
        uint256 storedHandle = auction.getPendingBidHandle(auctionId, bidder1);
        assertTrue(storedHandle != 0);
    }

    function test_submit_multiple_bids() public {
        vm.prank(seller);
        uint256 auctionId = auction.createAuction(START_PRICE, RESERVE_PRICE, DURATION);

        // bidder1 bids $750
        (bytes32 h1, bytes memory p1) = encryptUint64(750_000000, address(auction), bidder1);
        vm.prank(bidder1);
        auction.submitBid(auctionId, einput.wrap(h1), p1);

        // bidder2 bids $900
        (bytes32 h2, bytes memory p2) = encryptUint64(900_000000, address(auction), bidder2);
        vm.prank(bidder2);
        auction.submitBid(auctionId, einput.wrap(h2), p2);

        // Both bids stored — FHE selects winner without revealing amounts
        assertTrue(auction.getPendingBidHandle(auctionId, bidder1) != 0);
        assertTrue(auction.getPendingBidHandle(auctionId, bidder2) != 0);
    }

    function test_bid_rejected_after_end_time() public {
        vm.prank(seller);
        uint256 auctionId = auction.createAuction(START_PRICE, RESERVE_PRICE, DURATION);

        vm.warp(block.timestamp + DURATION + 1);
        assertFalse(auction.isAuctionActive(auctionId));

        (bytes32 handle, bytes memory proof) =
            encryptUint64(750_000000, address(auction), bidder1);

        vm.prank(bidder1);
        vm.expectRevert("Auction ended");
        auction.submitBid(auctionId, einput.wrap(handle), proof);
    }

    function test_settlement_request() public {
        vm.prank(seller);
        uint256 auctionId = auction.createAuction(START_PRICE, RESERVE_PRICE, DURATION);

        (bytes32 h, bytes memory p) = encryptUint64(750_000000, address(auction), bidder1);
        vm.prank(bidder1);
        auction.submitBid(auctionId, einput.wrap(h), p);

        vm.warp(block.timestamp + DURATION + 1);
        uint256 requestId = auction.requestSettlement(auctionId);
        assertTrue(requestId > 0);
    }
}
