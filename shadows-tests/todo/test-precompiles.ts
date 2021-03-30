import {assert, expect} from "chai";
// @ts-ignore
import ECRecoverTests from "../artifacts/contracts/ECRecoverTests.sol/ECRecoverTests.json";
// @ts-ignore
import {createAndFinalizeBlock, customRequest, describeWithShadows} from "../test/util";
import {AbiItem} from "web3-utils";

describeWithShadows("Shadows RPC (Precompile)", (context) => {
    const GENESIS_ACCOUNT = "0xAA7358886fd6FEc1d64323D9da340FD3c0B9a9E4";
    const GENESIS_ACCOUNT_PRIVATE_KEY = "0x665c5c10437cc1220b805b3b6d015c82f476e1d8144f08ba85840eddf4b903a5";

    const TEST_CONTRACT_BYTECODE = ECRecoverTests.bytecode;
    const TEST_CONTRACT_ABI = ECRecoverTests.abi as AbiItem[];
    const FIRST_CONTRACT_ADDRESS = "0xc2bf5f29a4384b1ab0c063e1c666f02121b6084a";

    // @ts-ignore
    let web3;

    before(async () => {
        web3 = context.web3;
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
        await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
        await createAndFinalizeBlock(context.polkadotApi);
        // ensure native web3 sending works as well as truffle provider
        web3.eth.accounts.wallet.add(GENESIS_ACCOUNT_PRIVATE_KEY);
        web3.eth.defaultAccount = web3.eth.accounts.wallet[0].address;
    });

    // Those test are ordered. In general this should be avoided, but due to the time it takes
    // to spin up a Shadows node, it saves a lot of time.

    it.only('should perform ecrecover', async () => {
        const web3 = context.web3;

        const message = 'Lorem ipsum dolor sit amet, consectetur adipiscing elit. Tubulum fuisse, qua illum, cuius is condemnatus est rogatione, P. Eaedem res maneant alio modo.'
        const messageHex = '0x' + Buffer.from(message).toString('hex');
        const sig = (await web3.eth.sign(messageHex, GENESIS_ACCOUNT)).slice(2);
        const r = `${sig.slice(0, 64)}`
        const s = `${sig.slice(64, 128)}`
        const v = `${sig.slice(128, 130)}`
        const sigPart = `${Buffer.alloc(31).toString('hex')}${v}${r}${s}`;
        // @ts-ignore
        const hash = web3.utils.sha3('\x19Ethereum Signed Message:\n' + message.length + message).slice(2);
        const tx = await context.web3.eth.accounts.signTransaction(
            {
                from: GENESIS_ACCOUNT,
                to: '0000000000000000000000000000000000000005',
                data: `0x${hash.toString()}${sigPart}`,
                value: "0x00",
                gasPrice: "0x01",
                gas: "0x100000",
            },
            GENESIS_ACCOUNT_PRIVATE_KEY
        );

        const contract = new context.web3.eth.Contract(TEST_CONTRACT_ABI, FIRST_CONTRACT_ADDRESS, {
            from: GENESIS_ACCOUNT,
            gasPrice: "0x01",
        });

        await contract.methods
            .ecrecover(`0x${hash.toString()}${sigPart}`)
            .call()
    });

    it('should perform identity directly', async () => {
        const message = '0x1234567890'
        // @ts-ignore
        const callResult = await web3.eth.call({
            to: '0000000000000000000000000000000000000004',
            from: GENESIS_ACCOUNT,
            data: message,
        });
        assert.equal(callResult, message);
    });
});
