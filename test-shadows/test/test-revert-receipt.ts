import {expect} from "chai";

import {createAndFinalizeBlock, customRequest, describeWithShadows} from "./util";

describeWithShadows("Shadows RPC (Constructor Revert)", (context) => {

    const GENESIS_ACCOUNT = "0xAA7358886fd6FEc1d64323D9da340FD3c0B9a9E4";
    const GENESIS_ACCOUNT_PRIVATE_KEY = "0x665c5c10437cc1220b805b3b6d015c82f476e1d8144f08ba85840eddf4b903a5";

    // ```
    // pragma solidity >=0.4.22 <0.7.0;
    //
    // contract WillFail {
    //		 constructor() public {
    //				 require(false);
    //		 }
    // }
    // ```
    const FAIL_BYTECODE = '6080604052348015600f57600080fd5b506000601a57600080fd5b603f8060276000396000f3fe6080604052600080fdfea26469706673582212209f2bb2a4cf155a0e7b26bd34bb01e9b645a92c82e55c5dbdb4b37f8c326edbee64736f6c63430006060033';
    const GOOD_BYTECODE = '6080604052348015600f57600080fd5b506001601a57600080fd5b603f8060276000396000f3fe6080604052600080fdfea2646970667358221220c70bc8b03cdfdf57b5f6c4131b836f9c2c4df01b8202f530555333f2a00e4b8364736f6c63430006060033';

    it("should provide a tx receipt after successful deployment", async function () {
        this.timeout(15000);
        const GOOD_TX_HASH = '0x914cc09f94737d708cfb17913e7ba11efc425be8a89320c42cb99f7f13f25724';

        const tx = await context.web3.eth.accounts.signTransaction(
            {
                from: GENESIS_ACCOUNT,
                data: GOOD_BYTECODE,
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
            result: GOOD_TX_HASH,
        });

        // Verify the receipt exists after the block is created
        await createAndFinalizeBlock(context.polkadotApi);
        const receipt = await context.web3.eth.getTransactionReceipt(GOOD_TX_HASH);
        expect(receipt).to.include({
            blockNumber: 1,
            contractAddress: '0x22b7265E52943D5A2F610bCf075F6AC307BcC706',
            cumulativeGasUsed: 67231,
            from: '0xaa7358886fd6fec1d64323d9da340fd3c0b9a9e4',
            gasUsed: 67231,
            to: null,
            transactionHash: '0x914cc09f94737d708cfb17913e7ba11efc425be8a89320c42cb99f7f13f25724',
            transactionIndex: 0,
            status: true
        });
    });

    it("should provide a tx receipt after failed deployment", async function () {
        this.timeout(15000);
        // Transaction hash depends on which nonce we're using
        //const FAIL_TX_HASH = '0x89a956c4631822f407b3af11f9251796c276655860c892919f848699ed570a8d'; //nonce 1
        const FAIL_TX_HASH = '0x775a8c4b030e55a871f813e051b43288e8c3c29cdbe0769f8e6b1bde457650c3'; //nonce 2

        const tx = await context.web3.eth.accounts.signTransaction(
            {
                from: GENESIS_ACCOUNT,
                data: FAIL_BYTECODE,
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
            result: FAIL_TX_HASH,
        });

        await createAndFinalizeBlock(context.polkadotApi);
        const receipt = await context.web3.eth.getTransactionReceipt(FAIL_TX_HASH);
        expect(receipt).to.include({
            blockNumber: 2,
            contractAddress: '0x122838b1F1759D0Cb9EdC1A74772CE6b66d1f44d',
            cumulativeGasUsed: 54600,
            from: '0xaa7358886fd6fec1d64323d9da340fd3c0b9a9e4',
            gasUsed: 54600,
            to: null,
            transactionHash: '0x775a8c4b030e55a871f813e051b43288e8c3c29cdbe0769f8e6b1bde457650c3',
            transactionIndex: 0,
            status: false
        });
    });
});
