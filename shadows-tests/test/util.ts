import Web3 from "web3";
import {JsonRpcResponse} from "web3-core-helpers";
import {spawn, ChildProcess} from "child_process";
import {ApiPromise, WsProvider} from "@polkadot/api";

export const PORT = 19931;
export const RPC_PORT = 19932;
export const WS_PORT = 19933;
export const SPECS_PATH = `./Shadows-test-specs`;

export const DISPLAY_LOG = process.env.Shadows_LOG || true;
export const Shadows_LOG = process.env.Shadows_LOG || "info";

export const BINARY_PATH = `../target/release/shadows-node`;
export const SPAWNING_TIME = 30000;

export async function customRequest(web3: Web3, method: string, params: any[]) {
    console.error(` <== custom request ${method} (${params.join(",")})`);
    return new Promise<JsonRpcResponse>((resolve, reject) => {
        (web3.currentProvider as any).send(
            {
                jsonrpc: "2.0",
                id: 1,
                method,
                params,
            },
            (error: Error | null, result?: JsonRpcResponse) => {
                if (error) {
                    reject(
                        `Failed to send custom request (${method} (${params.join(",")})): ${
                            error.message || error.toString()
                        }`
                    );
                } else {
                    console.error(` ==> custom response ${JSON.stringify(result)}`);
                }
                // @ts-ignore
                resolve(result);
            }
        );
    });
}

// Create a block and finalize it.
// It will include all previously executed transactions since the last finalized block.
export async function createAndFinalizeBlock(api: ApiPromise): Promise<number> {
    const startTime: number = Date.now();
    try {
        console.error(` <== createBlock(true, true)`);
        const response = await api.rpc.engine.createBlock(true, true);
        console.error(` ==> ${JSON.stringify(response)}`);
    } catch (e) {
        console.log("ERROR DURING BLOCK FINALIZATION", e);
    }
    return Date.now() - startTime;
}


// Create a block and finalize it.
// It will include all previously executed transactions since the last finalized block.
export async function createAndFinalizeBlockWeb3(web3: Web3) {
    const response = await customRequest(web3, "engine_createBlock", [true, true, null]);
    if (!response.result) {
        throw new Error(`Unexpected result: ${JSON.stringify(response)}`);
    } else {
        console.error(`createAndFinalizeBlock: ${JSON.stringify(response)}`);
    }
}

export interface Context {
    web3: Web3;
    // WsProvider for the PolkadotJs API
    wsProvider: WsProvider;
    polkadotApi: ApiPromise;
}

export async function startShadowsNode(provider?: string): Promise<{ context: Context; binary: ChildProcess }> {

    // @ts-ignore
    var web3;
    if (!provider || provider == 'http') {
        web3 = new Web3(`http://localhost:${RPC_PORT}`);
    }

    const cmd = BINARY_PATH;
    const args = [
        `--dev`,
        //`--chain=${SPECS_PATH}/${specFilename}`,
        //`--validator`, // Required by manual sealing to author the blocks
        `--execution=Native`, // Faster execution using native
        `--no-telemetry`,
        `--no-prometheus`,
        `--sealing=Manual`,
        `--no-grandpa`,
        //`--force-authoring`,
        //`-l${Shadows_LOG}`,
        `--port=${PORT}`,
        `--rpc-port=${RPC_PORT}`,
        `--ws-port=${WS_PORT}`,
        `--tmp`,
    ];
    const binary = spawn(cmd, args);

    binary.on("error", (err) => {
        if ((err as any).errno == "ENOENT") {
            console.error(
                `\x1b[31mMissing Shadows binary (${BINARY_PATH}).\nPlease compile the Shadows project:\ncargo build\x1b[0m`
            );
        } else {
            console.error(err);
        }
        process.exit(1);
    });

    // @ts-ignore
    const binaryLogs = [];
    await new Promise<void>((resolve) => {
        const timer = setTimeout(() => {
            console.error(`\x1b[31m Failed to start Shadows Template Node.\x1b[0m`);
            console.error(`Command: ${cmd} ${args.join(" ")}`);
            console.error(`Logs:`);
            // @ts-ignore
            console.error(binaryLogs.map((chunk) => chunk.toString()).join("\n"));
            process.exit(1);
        }, SPAWNING_TIME - 2000);

        // @ts-ignore
        const onData = async (chunk) => {
            if (DISPLAY_LOG) {
                console.log(chunk.toString());
            }
            binaryLogs.push(chunk);
            if (chunk.toString().match(/Shadows severce Ready/)) {
                if (!provider || provider == "http") {
                    // This is needed as the EVM runtime needs to warmup with a first call
                    // @ts-ignore
                    await web3.eth.getChainId();
                }

                clearTimeout(timer);
                if (!DISPLAY_LOG) {
                    binary.stderr.off("data", onData);
                    binary.stdout.off("data", onData);
                }
                // console.log(`\x1b[31m Starting RPC\x1b[0m`);
                resolve();
            }
        };
        binary.stderr.on("data", onData);
        binary.stdout.on("data", onData);
    });

    if (provider == 'ws') {
        web3 = new Web3(`ws://localhost:${WS_PORT}`);
    }
    const wsProvider = new WsProvider(`ws://localhost:${WS_PORT}`);
    const polkadotApi = await ApiPromise.create({
        provider: wsProvider
    });

    // @ts-ignore
    return {context: {web3, polkadotApi}, binary};
}

export function describeWithShadows(title: string, cb: (context: Context) => void, provider?: string) {
    // @ts-ignore
    describe(title, () => {
        // @ts-ignore
        let context: Context = {web3: null, wsProvider: null, polkadotApi: null};
        let binary: ChildProcess;
        // Making sure the Shadows node has started
        // @ts-i``gnore
        before("Starting Shadows Test Node", async function () {
            // @ts-ignore
            this.timeout(SPAWNING_TIME);
            const init = await startShadowsNode(provider);
            context.web3 = init.context.web3;
            context.polkadotApi = init.context.polkadotApi;
            binary = init.binary;
        });

        // @ts-ignore
        after(async function () {
            //console.log(`\x1b[31m Killing RPC\x1b[0m`);
            binary.kill();
        });

        cb(context);
    });
}
