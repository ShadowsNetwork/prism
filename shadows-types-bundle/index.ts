import type {
  OverrideBundleDefinition,
  OverrideBundleType,
} from "@polkadot/api/node_modules/@polkadot/types/types";
export const shadowsDefinitions = {
  types: [
    {
      minmax: [0, 4],
      types: {
        AccountId: "EthereumAccountId",
        Address: "AccountId",
        Balance: "u128",
        RefCount: "u8",
        LookupSource: "AccountId",
        Account: {
          nonce: "U256",
          balance: "u128",
        },
      },
    },
    {
      minmax: [5, 5],
      types: {
        AccountId: "EthereumAccountId",
        Address: "AccountId",
        Balance: "u128",
        LookupSource: "AccountId",
        Account: {
          nonce: "U256",
          balance: "u128",
        },
      },
    },
    {
      minmax: [6, undefined],
      types: {
        AccountId: "EthereumAccountId",
        Address: "AccountId",
        Balance: "u128",
        LookupSource: "AccountId",
        Account: {
          nonce: "U256",
          balance: "u128",
        },
        ExtrinsicSignature: "EthereumSignature",
        RoundIndex: "u32",
        Candidate: {
          validator: "AccountId",
          fee: "Perbill",
          nominators: "Vec<Bond>",
          total: "Balance",
          state: "ValidatorStatus",
        },
        Bond: {
          owner: "AccountId",
          amount: "Balance",
        },
        ValidatorStatus: {
          _enum: ["Active", "Idle", "Leaving(RoundIndex)"],
        },
      },
    },
  ],
} as OverrideBundleDefinition;

export const typesBundle = {
  spec: {
    "ombre-alphanet": shadowsDefinitions,
    shadowsDefinitions,
    "shadows-standalone": shadowsDefinitions,
    "node-shadows": shadowsDefinitions,
  },
} as OverrideBundleType;
