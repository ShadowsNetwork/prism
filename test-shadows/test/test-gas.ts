import {expect} from "chai";

// @ts-ignore
import Test from "../artifacts/contracts/Test.sol/Test.json";
import {describeWithShadows, createAndFinalizeBlock} from "./util";
import {AbiItem} from "web3-utils";

describeWithShadows("Shadows RPC (Gas)", (context) => {
    const GENESIS_ACCOUNT = "0xAA7358886fd6FEc1d64323D9da340FD3c0B9a9E4";

    const TEST_CONTRACT_BYTECODE = Test.bytecode;
    const TEST_CONTRACT_ABI = Test.abi as AbiItem[];
    const FIRST_CONTRACT_ADDRESS = "0xc2bf5f29a4384b1ab0c063e1c666f02121b6084a"; // Those test are ordered. In general this should be avoided, but due to the time it takes	// to spin up a Shadows node, it saves a lot of time.

    it("eth_estimateGas for contract creation", async function () {
        expect(
            await context.web3.eth.estimateGas({
                from: GENESIS_ACCOUNT,
                data: Test.bytecode,
            })
        ).to.equal(91235);
    });

    it.skip("block gas limit over 5M", async function () {
        expect((await context.web3.eth.getBlock("latest")).gasLimit).to.be.above(5000000);
    });

    // Testing the gas limit protection, hardcoded to 25M
    it.skip("gas limit should decrease on next block if gas unused", async function () {
        this.timeout(15000);

        const gasLimit = (await context.web3.eth.getBlock("latest")).gasLimit;
        await createAndFinalizeBlock(context.polkadotApi);

        // Gas limit is expected to have decreased as the gasUsed by the block is lower than 2/3 of the previous gas limit
        const newGasLimit = (await context.web3.eth.getBlock("latest")).gasLimit;
        expect(newGasLimit).to.be.below(gasLimit);
    });


    it("eth_estimateGas for contract call", async function () {
        const contract = new context.web3.eth.Contract(TEST_CONTRACT_ABI, FIRST_CONTRACT_ADDRESS, {
            from: GENESIS_ACCOUNT,
            gasPrice: "0x01",
        });

        expect(await contract.methods.multiply(3).estimateGas()).to.equal(21204);
    });

    it("eth_estimateGas without gas_limit should pass", async function () {
        const contract = new context.web3.eth.Contract(TEST_CONTRACT_ABI, FIRST_CONTRACT_ADDRESS, {
            from: GENESIS_ACCOUNT
        });

        expect(await contract.methods.multiply(3).estimateGas()).to.equal(21204);
    });

});
