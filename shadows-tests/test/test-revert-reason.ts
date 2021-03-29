import {expect} from "chai";
// @ts-ignore
import ExplicitRevertReason from '../artifacts/contracts/ExplicitRevertReason.sol/ExplicitRevertReason.json';
import {createAndFinalizeBlock, customRequest, describeWithShadows} from "./util";
import {AbiItem} from "web3-utils";

describeWithShadows("Shadows RPC (Revert Reason)", (context) => {

    // @ts-ignore
    let contractAddress;

    const GENESIS_ACCOUNT = "0xAA7358886fd6FEc1d64323D9da340FD3c0B9a9E4";
    const GENESIS_ACCOUNT_PRIVATE_KEY = "0x665c5c10437cc1220b805b3b6d015c82f476e1d8144f08ba85840eddf4b903a5";

    const REVERT_W_MESSAGE_BYTECODE = ExplicitRevertReason.bytecode;

    const TEST_CONTRACT_ABI = ExplicitRevertReason.abi as AbiItem[];

    before("create the contract", async function () {
        this.timeout(15000);
        const tx = await context.web3.eth.accounts.signTransaction(
            {
                from: GENESIS_ACCOUNT,
                data: REVERT_W_MESSAGE_BYTECODE,
                value: "0x00",
                gasPrice: "0x01",
                gas: "0x100000",
            },
            GENESIS_ACCOUNT_PRIVATE_KEY
        );
        const r = await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
        await createAndFinalizeBlock(context.polkadotApi);
        const receipt = await context.web3.eth.getTransactionReceipt(r.result);
        contractAddress = receipt.contractAddress;
    });

    it("should fail with revert reason", async function () {
        // @ts-ignore
        const contract = new context.web3.eth.Contract(TEST_CONTRACT_ABI, contractAddress, {
            from: GENESIS_ACCOUNT,
            gasPrice: "0x01",
        });
        try {
            await contract.methods.max10(30).call();
        } catch (error) {
            expect(error.message).to.be.eq(
                "Returned error: VM Exception while processing transaction: revert Value must not be greater than 10."
            );
        }
    });
});
