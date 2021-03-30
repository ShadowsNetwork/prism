// @ts-ignore
import {expect} from "chai";
import {
    FIRST_CONTRACT_ADDRESS,
    TEST_CONTRACT_BYTECODE,
} from "./constants";


// @ts-ignore
import Token from "../artifacts/contracts/Token.sol/Token.json";
import {createAndFinalizeBlock, customRequest, describeWithShadows} from "./util";

describeWithShadows("Shadows RPC (Contract)", (context) => {

    it("contract creation should return transaction hash", async function () {

        const GENESIS_ACCOUNT = "0xAA7358886fd6FEc1d64323D9da340FD3c0B9a9E4";
        const GENESIS_ACCOUNT_PRIVATE_KEY = "0x665c5c10437cc1220b805b3b6d015c82f476e1d8144f08ba85840eddf4b903a5";

        this.timeout(15000);
        const tx = await context.web3.eth.accounts.signTransaction(
            {
                from: GENESIS_ACCOUNT,
                data: TEST_CONTRACT_BYTECODE,
                value: "0x00",
                gasPrice: "0x01",
                gas: "0x100000",
            },
            GENESIS_ACCOUNT_PRIVATE_KEY
        );

        expect(
            await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction])
        ).to.deep.equal({
            id: 1,
            jsonrpc: "2.0",
            result: "0x688a55a501b220fdb5d939e6ab880c9c8c8966b8082103b9d8757c017a0f16fc",
        });

        // Verify the contract is not yet stored
        expect(
            await customRequest(context.web3, "eth_getCode", [FIRST_CONTRACT_ADDRESS])
        ).to.deep.equal({
            id: 1,
            jsonrpc: "2.0",
            result: "0x",
        });

        // Verify the contract is stored after the block is produced
        await createAndFinalizeBlock(context.polkadotApi);

        const receipt = await context.web3.eth.getTransactionReceipt("0x688a55a501b220fdb5d939e6ab880c9c8c8966b8082103b9d8757c017a0f16fc");
        expect(receipt).to.include({
            contractAddress: '0x22b7265E52943D5A2F610bCf075F6AC307BcC706'
        });

        expect(
            await customRequest(context.web3, "eth_getCode", [FIRST_CONTRACT_ADDRESS])
        ).to.deep.equal({
            id: 1,
            jsonrpc: "2.0",
            result:
                "0x6080604052348015600f57600080fd5b506004361060285760003560e01c8063c6888fa114602d575b60" +
                "0080fd5b605660048036036020811015604157600080fd5b8101908080359060200190929190505050606c" +
                "565b6040518082815260200191505060405180910390f35b600060078202905091905056fea265627a7a72" +
                "315820f06085b229f27f9ad48b2ff3dd9714350c1698a37853a30136fa6c5a7762af7364736f6c63430005" +
                "110032",
        });

    });

});
