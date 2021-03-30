import { expect } from "chai";
import { step } from "mocha-steps";

import { createAndFinalizeBlock, describeWithShadows, customRequest } from "./util";

describeWithShadows("Shadows RPC (Balance)", (context) => {
	const GENESIS_ACCOUNT = "0xAA7358886fd6FEc1d64323D9da340FD3c0B9a9E4";
	const GENESIS_ACCOUNT_PRIVATE_KEY = "0x665c5c10437cc1220b805b3b6d015c82f476e1d8144f08ba85840eddf4b903a5";
	const GENESIS_ACCOUNT_BALANCE = "999999999999999000000000000000000";
	const TEST_ACCOUNT = "0x1111111111111111111111111111111111111111";

	step("genesis balance is setup correctly", async function () {
		expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT)).to.equal(GENESIS_ACCOUNT_BALANCE);
	});

	step("balance to be updated after transfer", async function () {
		this.timeout(15000);

		const tx = await context.web3.eth.accounts.signTransaction({
			from: GENESIS_ACCOUNT,
			to: TEST_ACCOUNT,
			value: "0x200", // Must me higher than ExistentialDeposit (500)
			gasPrice: "0x01",
			gas: "0x100000",
		}, GENESIS_ACCOUNT_PRIVATE_KEY);
		await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
		await createAndFinalizeBlock(context.polkadotApi);
		expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT)).to.equal("999999999999998999999999999978488");
		expect(await context.web3.eth.getBalance(TEST_ACCOUNT)).to.equal("512");
	});
});
