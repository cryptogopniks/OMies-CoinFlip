import { floor, l } from "../../../common/utils";
import { readFile, writeFile } from "fs/promises";
import { ChainConfig } from "../../../common/interfaces";
import { getChainOptionById } from "../../../common/config/config-utils";
import { getCwQueryHelpers } from "../../../common/account/cw-helpers";
import {
  ENCODING,
  PATH_TO_CONFIG_JSON,
  parseStoreArgs,
  getSnapshotPath,
  epochToDateStringUTC,
} from "../utils";

const PAGINATION_QUERY_AMOUNT = 200;

const toNumber = (x: string) => floor(Number(x) / 1e6, 1);

async function main() {
  const configJsonStr = await readFile(PATH_TO_CONFIG_JSON, {
    encoding: ENCODING,
  });
  const CHAIN_CONFIG: ChainConfig = JSON.parse(configJsonStr);
  const { chainId } = parseStoreArgs();
  const {
    NAME,
    OPTION: {
      RPC_LIST: [RPC],
      TYPE,
    },
  } = getChainOptionById(CHAIN_CONFIG, chainId);

  const { platform } = await getCwQueryHelpers(chainId, RPC);

  const writeUsers = async () => {
    try {
      // sort by ROI descending
      const users = (await platform.pQueryUserList(PAGINATION_QUERY_AMOUNT))
        .sort((a, b) => Number(b.info.roi) - Number(a.info.roi))
        .map(
          ({
            address,
            info: {
              stats: {
                bets: { count: betsCount, value: betsValue },
                wins: { count: winsCount, value: winsValue },
              },
              roi,
              unclaimed,
              last_flip_date,
            },
          }) => {
            const lastFlipDate = epochToDateStringUTC(last_flip_date);

            return {
              address,
              bets: {
                count: betsCount,
                value: toNumber(betsValue),
              },
              wins: {
                count: winsCount,
                value: toNumber(winsValue),
              },
              roi: Number(roi),
              unclaimed: toNumber(unclaimed),
              lastFlipDate,
            };
          }
        );

      // write files
      await writeFile(
        getSnapshotPath(NAME, TYPE, "users.json"),
        JSON.stringify(users, null, 2),
        {
          encoding: ENCODING,
        }
      );
    } catch (error) {
      l(error);
    }
  };

  await writeUsers();
}

main();
