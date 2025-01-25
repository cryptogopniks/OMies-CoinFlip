import { NetworkName } from "../../common/config/index";

export type NetworkItem = {
  [K in NetworkName]?: string;
};

export const LEGACY_CHAIN_ID_LIST = ["stargaze-0", "stargaze-2"];
