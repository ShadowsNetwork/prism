// @ts-ignore
import { expect } from "chai";
// @ts-ignore
import { step } from "mocha-steps";

import { createAndFinalizeBlock, describeWithFrontier } from "./util";

describeWithFrontier("Frontier RPC (Block)", `simple-specs.json`, (context) => {
	// Those tests are dependant of each other in the given order.
	// The reason is to avoid having to restart the node each time
	// Running them individually will result in failure

	step("should be at block 0 at genesis", async function () {
		expect(await context.web3.eth.getBlockNumber()).to.equal(0);
	});

	// @ts-ignore
	it("should return genesis block by number", async function () {
		expect(await context.web3.eth.getBlockNumber()).to.equal(0);

		const block = await context.web3.eth.getBlock(0);

		console.error(`block: ${block}`);

		expect(block).to.include({
			author: "0x0000000000000000000000000000000000000000",
			difficulty: "0",
			extraData: "0x",
			gasLimit: 4294967295,
			gasUsed: 0,
			logsBloom:
				"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
			miner: "0x0000000000000000000000000000000000000000",
			number: 0,
			receiptsRoot: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
			size: 505,
			timestamp: 0,
			totalDifficulty: "0",
		});

		expect((block as any).sealFields).to.eql([
			"0x0000000000000000000000000000000000000000000000000000000000000000",
			"0x0000000000000000",
		]);
		expect(block.hash).to.be.a("string").lengthOf(66);
		expect(block.parentHash).to.be.a("string").lengthOf(66);
		expect(block.timestamp).to.be.a("number");
	});

});
