use cosmwasm_std::StdError;
use thiserror::Error;

impl From<StdError> for ContractError {
    fn from(std_error: StdError) -> Self {
        Self::CustomError {
            val: std_error.to_string(),
        }
    }
}

impl From<ContractError> for StdError {
    fn from(contract_error: ContractError) -> Self {
        Self::generic_err(contract_error.to_string())
    }
}

pub fn parse_err(err: anyhow::Error) -> StdError {
    let context = format!("{:#?}", err);
    let source = err.source().map(|x| x.to_string()).unwrap_or_default();

    StdError::generic_err(format!("{}\n{}", context, source))
}

/// Never is a placeholder to ensure we don't return any errors
#[derive(Error, Debug)]
pub enum Never {}

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("Sender does not have access permissions!")]
    Unauthorized,

    #[error("Parameters are not provided!")]
    NoParameters,

    #[error("It's too late to accept admin role!")]
    TransferAdminDeadline,

    #[error("Not enough liquidity to withdraw funds!")]
    NotEnoughLiquidity,

    #[error("Chain ID is not found!")]
    ChainIdIsNotFound,

    #[error("Zero amount to send!")]
    ZeroAmount,

    #[error("Undefined Reply ID!")]
    UndefinedReplyId,

    #[error("Asset is not found!")]
    AssetIsNotFound,

    #[error("Wrong asset type!")]
    WrongAssetType,

    #[error("Wrong message type!")]
    WrongMessageType,

    #[error("Wrong action type!")]
    WrongActionType,

    #[error("Wrong funds combination!")]
    WrongFundsCombination,

    #[error("{value:?} config is not found!")]
    ParameterIsNotFound { value: String },

    #[error("The contract is paused")]
    ContractIsPaused,

    #[error("Denom already exists!")]
    DenomExists,

    #[error("Exceeded tokens per owner limit!")]
    TokenLimit,

    #[error("Bet is out of range!")]
    BetIsOutOfRange,

    #[error("Parsing previous version error!")]
    ParsingPrevVersion,

    #[error("Parsing new version error!")]
    ParsingNewVersion,

    #[error("Msg version is not equal contract new version!")]
    ImproperMsgVersion,
}
