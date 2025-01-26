import { PlatformMsgComposer } from "../codegen/Platform.message-composer";
import { PlatformQueryClient } from "../codegen/Platform.client";
import { Range } from "../codegen/Platform.types";

import CONFIG_JSON from "../config/config.json";
import { getLast, l, logAndReturn } from "../utils";
import { toBase64, fromUtf8, toUtf8 } from "@cosmjs/encoding";
import {
  MsgMigrateContract,
  MsgUpdateAdmin,
} from "cosmjs-types/cosmwasm/wasm/v1/tx";
import { getChainOptionById, getContractByLabel } from "../config/config-utils";
import {
  getCwClient,
  signAndBroadcastWrapper,
  getExecuteContractMsg,
} from "./clients";
import {
  SigningCosmWasmClient,
  CosmWasmClient,
  MsgExecuteContractEncodeObject,
  MsgUpdateAdminEncodeObject,
  MsgMigrateContractEncodeObject,
} from "@cosmjs/cosmwasm-stargate";
import {
  DirectSecp256k1HdWallet,
  OfflineSigner,
  OfflineDirectSigner,
  coin,
} from "@cosmjs/proto-signing";
import {
  Cw20SendMsg,
  TokenUnverified,
  ChainConfig,
  ContractInfo,
  QueryAllOperatorsResponse,
  QueryAllOperatorsMsg,
  ApproveAllMsg,
  RevokeAllMsg,
  QueryApprovalsMsg,
  ApprovalsResponse,
  QueryTokens,
  TokensResponse,
  QueryOwnerOf,
  OwnerOfResponse,
} from "../interfaces";
import { Side } from "../codegen/Platform.types";

function addSingleTokenToComposerObj(
  obj: MsgExecuteContractEncodeObject,
  amount: number,
  token: TokenUnverified
): MsgExecuteContractEncodeObject {
  const {
    value: { contract, sender, msg },
  } = obj;

  if (!(contract && sender && msg)) {
    throw new Error(`${msg} parameters error!`);
  }

  return getSingleTokenExecMsg(
    contract,
    sender,
    JSON.parse(fromUtf8(msg)),
    amount,
    token
  );
}

function getSingleTokenExecMsg(
  contractAddress: string,
  senderAddress: string,
  msg: any,
  amount?: number,
  token?: TokenUnverified
) {
  // get msg without funds
  if (!(token && amount)) {
    return getExecuteContractMsg(contractAddress, senderAddress, msg, []);
  }

  // get msg with native token
  if ("native" in token) {
    return getExecuteContractMsg(contractAddress, senderAddress, msg, [
      coin(amount, token.native.denom),
    ]);
  }

  // get msg with CW20 token
  const cw20SendMsg: Cw20SendMsg = {
    send: {
      contract: contractAddress,
      amount: `${amount}`,
      msg: toBase64(msg),
    },
  };

  return getExecuteContractMsg(
    token.cw20.address,
    senderAddress,
    cw20SendMsg,
    []
  );
}

async function isCollectionApproved(
  signingClient: SigningCosmWasmClient,
  owner: string,
  target: string,
  collection: string
) {
  const queryAllOperatorsMsg: QueryAllOperatorsMsg = {
    all_operators: {
      owner,
    },
  };

  const { operators }: QueryAllOperatorsResponse =
    await signingClient.queryContractSmart(collection, queryAllOperatorsMsg);

  return operators.find((x) => x.spender === target);
}

function getApproveCollectionMsg(
  collectionAddress: string,
  senderAddress: string,
  operator: string
): MsgExecuteContractEncodeObject {
  const approveAllMsg: ApproveAllMsg = {
    approve_all: {
      operator,
    },
  };

  return getSingleTokenExecMsg(collectionAddress, senderAddress, approveAllMsg);
}

function getRevokeCollectionMsg(
  collectionAddress: string,
  senderAddress: string,
  operator: string
): MsgExecuteContractEncodeObject {
  const revokeAllMsg: RevokeAllMsg = {
    revoke_all: {
      operator,
    },
  };

  return getSingleTokenExecMsg(collectionAddress, senderAddress, revokeAllMsg);
}

function getContracts(contracts: ContractInfo[]) {
  let PLATFORM_CONTRACT: ContractInfo | undefined;

  try {
    PLATFORM_CONTRACT = getContractByLabel(contracts, "platform");
  } catch (error) {
    l(error);
  }

  return {
    PLATFORM_CONTRACT,
  };
}

async function getCwExecHelpers(
  chainId: string,
  rpc: string,
  owner: string,
  signer: (OfflineSigner & OfflineDirectSigner) | DirectSecp256k1HdWallet
) {
  const CHAIN_CONFIG = CONFIG_JSON as ChainConfig;
  const {
    OPTION: { CONTRACTS },
  } = getChainOptionById(CHAIN_CONFIG, chainId);

  const { PLATFORM_CONTRACT } = getContracts(CONTRACTS);

  const cwClient = await getCwClient(rpc, owner, signer);
  if (!cwClient) throw new Error("cwClient is not found!");

  const signingClient = cwClient.client as SigningCosmWasmClient;
  const _signAndBroadcast = signAndBroadcastWrapper(signingClient, owner);

  const platformMsgComposer = new PlatformMsgComposer(
    owner,
    PLATFORM_CONTRACT?.ADDRESS || ""
  );

  async function _msgWrapperWithGasPrice(
    msgs: MsgExecuteContractEncodeObject[],
    gasPrice: string,
    gasAdjustment: number = 1,
    memo?: string
  ) {
    const tx = await _signAndBroadcast(msgs, gasPrice, gasAdjustment, memo);
    l("\n", tx, "\n");
    return tx;
  }

  // utils

  async function cwTransferAdmin(
    contract: string,
    newAdmin: string,
    gasPrice: string,
    gasAdjustment: number = 1
  ) {
    const msg: MsgUpdateAdminEncodeObject = {
      typeUrl: "/cosmwasm.wasm.v1.MsgUpdateAdmin",
      value: MsgUpdateAdmin.fromPartial({
        sender: owner,
        contract,
        newAdmin,
      }),
    };

    const tx = await _signAndBroadcast([msg], gasPrice, gasAdjustment);
    l("\n", tx, "\n");
    return tx;
  }

  async function cwMigrateMultipleContracts(
    contractList: string[],
    codeId: number,
    migrateMsg: any,
    gasPrice: string,
    gasAdjustment: number = 1
  ) {
    const msgList: MsgMigrateContractEncodeObject[] = contractList.map(
      (contract) => ({
        typeUrl: "/cosmwasm.wasm.v1.MsgMigrateContract",
        value: MsgMigrateContract.fromPartial({
          sender: owner,
          contract,
          codeId: BigInt(codeId),
          msg: toUtf8(JSON.stringify(migrateMsg)),
        }),
      })
    );

    const tx = await _signAndBroadcast(msgList, gasPrice, gasAdjustment);
    l("\n", tx, "\n");
    return tx;
  }

  async function cwRevoke(
    collectionAddress: string,
    senderAddress: string,
    operator: string,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [getRevokeCollectionMsg(collectionAddress, senderAddress, operator)],
      gasPrice
    );
  }

  async function cwMintNft(
    collectionAddress: string,
    recipient: string,
    tokenIdList: number[],
    gasPrice: string
  ) {
    const msgList = tokenIdList.map((tokenId) => {
      const mintMsg = {
        mint: {
          owner: recipient,
          token_id: tokenId.toString(),
        },
      };

      return getSingleTokenExecMsg(collectionAddress, owner, mintMsg);
    });

    return await _msgWrapperWithGasPrice(msgList, gasPrice);
  }

  // platform

  async function cwFlip(
    side: Side,
    amount: number,
    token: TokenUnverified,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        addSingleTokenToComposerObj(
          platformMsgComposer.flip({ side }),
          amount,
          token
        ),
      ],
      gasPrice
    );
  }

  async function cwClaim(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [platformMsgComposer.claim()],
      gasPrice
    );
  }

  async function cwAcceptAdminRole(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [platformMsgComposer.acceptAdminRole()],
      gasPrice
    );
  }

  async function cwDeposit(
    amount: number,
    token: TokenUnverified,
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        addSingleTokenToComposerObj(
          platformMsgComposer.deposit(),
          amount,
          token
        ),
      ],
      gasPrice
    );
  }

  async function cwWithdraw(
    amount: number,
    { recipient }: { recipient?: string },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [platformMsgComposer.withdraw({ amount: amount.toString(), recipient })],
      gasPrice
    );
  }

  async function cwUpdateConfig(
    {
      admin,
      worker,
      bet,
      platformFee,
    }: {
      admin?: string;
      worker?: string;
      bet?: Range;
      platformFee?: number;
    },
    gasPrice: string
  ) {
    return await _msgWrapperWithGasPrice(
      [
        platformMsgComposer.updateConfig({
          admin,
          worker,
          bet,
          platformFee: platformFee?.toString(),
        }),
      ],
      gasPrice
    );
  }

  async function cwPause(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [platformMsgComposer.pause()],
      gasPrice
    );
  }

  async function cwUnpause(gasPrice: string) {
    return await _msgWrapperWithGasPrice(
      [platformMsgComposer.unpause()],
      gasPrice
    );
  }

  return {
    utils: { cwTransferAdmin, cwMigrateMultipleContracts, cwRevoke, cwMintNft },
    platform: {
      cwFlip,
      cwClaim,
      cwAcceptAdminRole,
      cwDeposit,
      cwWithdraw,
      cwUpdateConfig,
      cwPause,
      cwUnpause,
    },
  };
}

async function getCwQueryHelpers(chainId: string, rpc: string) {
  const CHAIN_CONFIG = CONFIG_JSON as ChainConfig;
  const {
    OPTION: { CONTRACTS },
  } = getChainOptionById(CHAIN_CONFIG, chainId);

  const { PLATFORM_CONTRACT } = getContracts(CONTRACTS);

  const cwClient = await getCwClient(rpc);
  if (!cwClient) throw new Error("cwClient is not found!");

  const cosmwasmQueryClient: CosmWasmClient = cwClient.client;

  const platformQueryClient = new PlatformQueryClient(
    cosmwasmQueryClient,
    PLATFORM_CONTRACT?.ADDRESS || ""
  );

  // utils

  async function cwQueryOperators(
    ownerAddress: string,
    collectionAddress: string,
    isDisplayed: boolean = false
  ) {
    const queryAllOperatorsMsg: QueryAllOperatorsMsg = {
      all_operators: {
        owner: ownerAddress,
      },
    };
    const res: QueryAllOperatorsResponse =
      await cosmwasmQueryClient.queryContractSmart(
        collectionAddress,
        queryAllOperatorsMsg
      );
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryApprovals(
    collectionAddress: string,
    tokenId: string,
    isDisplayed: boolean = false
  ) {
    const queryApprovalsMsg: QueryApprovalsMsg = {
      approvals: {
        token_id: tokenId,
      },
    };
    const res: ApprovalsResponse = await cosmwasmQueryClient.queryContractSmart(
      collectionAddress,
      queryApprovalsMsg
    );
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryBalanceInNft(
    owner: string,
    collectionAddress: string,
    isDisplayed: boolean = false
  ) {
    const MAX_LIMIT = 100;
    const ITER_LIMIT = 50;

    let tokenList: string[] = [];
    let tokenAmountSum: number = 0;
    let i: number = 0;
    let lastToken: string | undefined = undefined;

    while ((!i || tokenAmountSum === MAX_LIMIT) && i < ITER_LIMIT) {
      i++;

      try {
        const queryTokensMsg: QueryTokens = {
          tokens: {
            owner,
            start_after: lastToken,
            limit: MAX_LIMIT,
          },
        };

        const { tokens }: TokensResponse =
          await cosmwasmQueryClient.queryContractSmart(
            collectionAddress,
            queryTokensMsg
          );

        tokenList = [...tokenList, ...tokens];
        tokenAmountSum = tokens.length;
        lastToken = getLast(tokens);
      } catch (error) {
        l(error);
      }
    }

    const res: TokensResponse = { tokens: tokenList };
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryNftOwner(
    collectionAddress: string,
    tokenId: string,
    isDisplayed: boolean = false
  ) {
    const queryOwnerOfMsg: QueryOwnerOf = {
      owner_of: { token_id: tokenId },
    };
    const res: OwnerOfResponse = await cosmwasmQueryClient.queryContractSmart(
      collectionAddress,
      queryOwnerOfMsg
    );
    return logAndReturn(res, isDisplayed);
  }

  // platform

  async function cwQueryConfig(isDisplayed: boolean = false) {
    const res = await platformQueryClient.config();
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryAppInfo(isDisplayed: boolean = false) {
    const res = await platformQueryClient.appInfo();
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryRequiredToDeposit(isDisplayed: boolean = false) {
    const res = await platformQueryClient.requiredToDeposit();
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryAvailableToWithdraw(isDisplayed: boolean = false) {
    const res = await platformQueryClient.availableToWithdraw();
    return logAndReturn(res, isDisplayed);
  }

  async function cwQueryUser(address: string, isDisplayed: boolean = false) {
    const res = await platformQueryClient.user({ address });
    return logAndReturn(res, isDisplayed);
  }

  // TODO: pQuery, snapshot
  async function cwQueryUserList(
    amount: number = 100,
    startAfter: string | undefined = undefined,
    isDisplayed: boolean = false
  ) {
    const res = await platformQueryClient.userList({
      amount,
      startAfter,
    });
    return logAndReturn(res, isDisplayed);
  }

  return {
    utils: {
      cwQueryOperators,
      cwQueryApprovals,
      cwQueryBalanceInNft,
      cwQueryNftOwner,
    },
    platform: {
      cwQueryConfig,
      cwQueryAppInfo,
      cwQueryRequiredToDeposit,
      cwQueryAvailableToWithdraw,
      cwQueryUser,
      cwQueryUserList,
    },
  };
}

export { getCwExecHelpers, getCwQueryHelpers };
