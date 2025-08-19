const { expect } = require("chai");

describe("Test", function () {
  it("Test", async function () {
    const factory = await ethers.getContractFactory("TestContract");
    const contract = await factory.deploy();

    // expect(await contract.getOddEvenDiff([1,2,3,4,5])).to.equal(1);
    // expect(await contract.getOddEvenDiff([1,3,5,7,9])).to.equal(5);
    expect(await contract.getOddEvenDiff([2,4,6,8,10])).to.equal(-5);
  })
});
