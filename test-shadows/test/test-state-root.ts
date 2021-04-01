import {expect} from "chai";
import {step} from "mocha-steps";

import {createAndFinalizeBlock, describeWithShadows} from "./util";

describeWithShadows("Shadows RPC (State root hash)", (context) => {

    // @ts-ignore
    let block;
    step("should calculate a valid intermediate state root hash", async function () {
        await createAndFinalizeBlock(context.polkadotApi);
        block = await context.web3.eth.getBlock(1);
        expect(block.stateRoot.length).to.be.equal(66); // 0x prefixed
        expect(block.stateRoot).to.not.be.equal(
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        );
    });

    step("hash should be unique between blocks", async function () {
        await createAndFinalizeBlock(context.polkadotApi);
        const anotherBlock = await context.web3.eth.getBlock(2);
        // @ts-ignore
        expect(block.stateRoot).to.not.be.equal(anotherBlock.stateRoot);
    });
});