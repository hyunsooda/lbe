// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.28;

contract TestContract {
    function getOddEvenDiff(int256[] memory arr) public pure returns (int256) {
        int256 evenCnt =0;
        int256 oddCnt =0;
        for (uint i=0; i<arr.length; i++) {
            if (arr[i] % 2 == 0) {
                evenCnt++;
            } else {
                oddCnt++;
            }
        }
        return oddCnt - evenCnt;
    }
}
