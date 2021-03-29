import {expect} from "chai";

import {createAndFinalizeBlock, customRequest, describeWithShadows} from "./util";

describeWithShadows("Shadows RPC (Pending Pool)", (context) => {
    const GENESIS_ACCOUNT = "0xAA7358886fd6FEc1d64323D9da340FD3c0B9a9E4";
    const GENESIS_ACCOUNT_PRIVATE_KEY = "0x665c5c10437cc1220b805b3b6d015c82f476e1d8144f08ba85840eddf4b903a5";

    // Solidity: contract test { function multiply(uint a) public pure returns(uint d) {return a * 7;}}
    const TEST_CONTRACT_BYTECODE =
        "0x6080604052348015600f57600080fd5b5060ae8061001e6000396000f3fe6080604052348015600f57600080fd5b506004361060285760003560e01c8063c6888fa114602d575b600080fd5b605660048036036020811015604157600080fd5b8101908080359060200190929190505050606c565b6040518082815260200191505060405180910390f35b600060078202905091905056fea265627a7a72315820f06085b229f27f9ad48b2ff3dd9714350c1698a37853a30136fa6c5a7762af7364736f6c63430005110032";
    const FIRST_CONTRACT_ADDRESS = "0xc2bf5f29a4384b1ab0c063e1c666f02121b6084a";

    it("should return a pending transaction", async function () {
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

        const tx_hash = (await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction])).result;

        const pending_transaction = (await customRequest(context.web3, "eth_getTransactionByHash", [tx_hash])).result;
        // pending transactions do not know yet to which block they belong to
        expect(pending_transaction).to.include({
            blockNumber: null,
            hash: tx_hash,
            publicKey: "0x53cb70c7b6405a4c16eee2fac23e242e555e62f38fc1e1ae801e4994f06620d5fb10ca2db58151a0f159edf48e5d216b3cd9300453ae2e626360f8f88302e70b",
            r: "0xf4b4f057b03ec97fba5b155152838b1a9b7ecd4aca34e40202a8cbb1b24a47c2",
            s: "0x7eca70fe699409ad51d5814b2d94151f2e38b339bc1c8b0161bf7ab8ecd3c01",
            v: "0x713",
        });

        await createAndFinalizeBlock(context.polkadotApi);

        const processed_transaction = (await customRequest(context.web3, "eth_getTransactionByHash", [tx_hash])).result;
        expect(processed_transaction).to.include({
            blockNumber: "0x1",
            hash: tx_hash,
            publicKey: "0x53cb70c7b6405a4c16eee2fac23e242e555e62f38fc1e1ae801e4994f06620d5fb10ca2db58151a0f159edf48e5d216b3cd9300453ae2e626360f8f88302e70b",
            r: "0xf4b4f057b03ec97fba5b155152838b1a9b7ecd4aca34e40202a8cbb1b24a47c2",
            s: "0x7eca70fe699409ad51d5814b2d94151f2e38b339bc1c8b0161bf7ab8ecd3c01",
            v: "0x713",
        });
    });
});
