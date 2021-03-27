require("@nomiclabs/hardhat-waffle");
require('@nomiclabs/hardhat-ethers');
import {HardhatUserConfig} from 'hardhat/types';

const ROPSTEN_PRIVATE_KEY = "0a052d705e5f027bb519f816b3622afc87bd0f833e739a1e3f7719adab6acd20";

/**
 * @type import('hardhat/config').HardhatUserConfig
 */
const config: HardhatUserConfig = {
    solidity: "0.7.3",
    networks: {
        devlocalhost: {
            url: `http://127.0.0.1:9933`,
            chainId: 9909,
            accounts: [`0x${ROPSTEN_PRIVATE_KEY}`]
        },
        dev: {
            url: `http://119.45.201.48:9933`,
            chainId: 888,
            accounts: [`0x${ROPSTEN_PRIVATE_KEY}`]
        },
        moonbase: {
            url: `https://rpc.testnet.moonbeam.network`,
            chainId: 1287,
            accounts: [`0x${ROPSTEN_PRIVATE_KEY}`]
        }
    }
};

export default config;
