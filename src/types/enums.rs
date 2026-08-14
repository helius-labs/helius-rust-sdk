use serde::{Deserialize, Serialize};
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::{Transaction, VersionedTransaction};

use super::*;

/// The asset interface type, representing the on-chain program standard used to create the asset.
///
/// Used in DAS API responses to classify assets by their underlying program standard.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub enum Interface {
    #[serde(rename = "V1_NFT")]
    V1NFT,
    #[default]
    #[serde(rename = "Custom")]
    Custom,
    #[serde(rename = "V1_PRINT")]
    V1Print,
    #[serde(rename = "Legacy_NFT")]
    LegacyNFT,
    #[serde(rename = "V2_NFT")]
    V2NFT,
    #[serde(rename = "FungibleAsset")]
    FungibleAsset,
    #[serde(rename = "Identity")]
    Identity,
    #[serde(rename = "Executable")]
    Executable,
    #[serde(rename = "ProgrammableNFT")]
    ProgrammableNFT,
    #[serde(rename = "FungibleToken")]
    FungibleToken,
    #[serde(rename = "MplCoreAsset")]
    MplCoreAsset,
    #[serde(rename = "MplCoreCollection")]
    MplCoreCollection,
}

/// The ownership model for an asset.
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub enum OwnershipModel {
    #[default]
    #[serde(rename = "single")]
    Single,
    #[serde(rename = "token")]
    Token,
}

/// The royalty distribution model for an asset.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RoyaltyModel {
    #[serde(rename = "creators")]
    Creators,
    #[serde(rename = "fanout")]
    Fanout,
    #[serde(rename = "single")]
    Single,
}

/// The use method for a Metaplex "uses" enabled asset.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum UseMethod {
    Burn,
    Single,
    Multiple,
}

/// The scope of a delegate's authority over an asset.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Scope {
    #[serde(rename = "full")]
    Full,
    #[serde(rename = "royalty")]
    Royalty,
    #[serde(rename = "metadata")]
    Metadata,
    #[serde(rename = "extension")]
    Extension,
}

/// The display context in which an asset's content should be rendered.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Context {
    #[serde(rename = "wallet-default")]
    WalletDefault,
    #[serde(rename = "web-desktop")]
    WebDesktop,
    #[serde(rename = "web-mobile")]
    WebMobile,
    #[serde(rename = "app-mobile")]
    AppMobile,
    #[serde(rename = "app-desktop")]
    AppDesktop,
    #[serde(rename = "app")]
    App,
    #[serde(rename = "vr")]
    Vr,
}

/// The field to sort assets by in DAS API queries.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum AssetSortBy {
    #[serde(rename = "id")]
    Id,
    #[serde(rename = "created")]
    Created,
    #[serde(rename = "updated")]
    Updated,
    #[serde(rename = "recent_action")]
    RecentAction,
    #[serde(rename = "none")]
    None,
}

/// The sort direction for DAS API queries.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum AssetSortDirection {
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}

/// Specifies whether all or any of the search conditions must be met.
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum SearchConditionType {
    #[serde(rename = "all")]
    All,
    #[serde(rename = "any")]
    Any,
}

/// The type of token to filter by in DAS API queries.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum TokenType {
    Fungible,
    NonFungible,
    CompressedNft,
    RegularNft,
    All,
}

/// The Helius mint API authority public key, varying by cluster.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MintApiAuthority {
    Mainnet(Pubkey),
    Devnet(Pubkey),
}

impl MintApiAuthority {
    pub fn from_cluster(cluster: &Cluster) -> Self {
        match cluster {
            Cluster::MainnetBeta | Cluster::StakedMainnetBeta => {
                MintApiAuthority::Mainnet(pubkey!("HnT5KVAywGgQDhmh6Usk4bxRg4RwKxCK4jmECyaDth5R"))
            }
            Cluster::Devnet => MintApiAuthority::Devnet(pubkey!("2LbAtCJSaHqTnP9M5QSjvAMXk79RNLusFspFN5Ew67TC")),
        }
    }
}

impl From<MintApiAuthority> for Pubkey {
    fn from(value: MintApiAuthority) -> Self {
        match value {
            MintApiAuthority::Mainnet(s) => s,
            MintApiAuthority::Devnet(s) => s,
        }
    }
}

/// The priority fee level for the `getPriorityFeeEstimate` RPC method.
///
/// Higher levels correspond to higher percentile estimates, resulting in faster inclusion
/// at a greater cost.
#[derive(Serialize, Deserialize, Debug)]
pub enum PriorityLevel {
    Min,
    Low,
    Medium,
    High,
    VeryHigh,
    UnsafeMax,
    Default,
}

/// The encoding format for transaction data in RPC responses.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum UiTransactionEncoding {
    Binary,
    Base64,
    Base58,
    Json,
    JsonParsed,
}

/// The classified type of an enhanced transaction.
///
/// Helius categorizes every Solana transaction into a labeled type for filtering and display.
/// These types span NFT marketplace operations, DeFi swaps, staking, governance, and more.
/// Use with [`ParsedTransactionHistoryRequest`](crate::types::ParsedTransactionHistoryRequest)
/// to filter transaction history, or inspect the `transaction_type` field on
/// [`EnhancedTransaction`](crate::types::EnhancedTransaction).
///
/// Serialized as `SCREAMING_SNAKE_CASE` (e.g., `NFT_SALE`, `COMPRESSED_NFT_MINT`).
/// Unrecognized values are captured in the `Other(String)` variant.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize_enum_str, Serialize_enum_str)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionType {
    AcceptEscrowArtist,
    AcceptEscrowUser,
    AcceptRequestArtist,
    ActivateTransaction,
    ActivateVault,
    AddInstruction,
    AddItem,
    AddRaritiesToBank,
    AddTokenToVault,
    AddToPool,
    AddToWhitelist,
    Any,
    ApproveTransaction,
    AttachMetadata,
    AuctionHouseCreate,
    AuctionManagerClaimBid,
    AuthorizeFunder,
    BorrowFox,
    BorrowSolForNft,
    Burn,
    BurnNft,
    BuyItem,
    BuySubscription,
    BuyTickets,
    CancelEscrow,
    CancelLoanRequest,
    CancelOffer,
    CancelOrder,
    CancelReward,
    CancelSwap,
    CancelTransaction,
    CandyMachineRoute,
    CandyMachineUnwrap,
    CandyMachineUpdate,
    CandyMachineWrap,
    ChangeComicState,
    ClaimNft,
    ClaimRewards,
    CloseAccount,
    CloseEscrowAccount,
    CloseItem,
    CloseOrder,
    ClosePosition,
    CompressedNftBurn,
    CompressedNftCancelRedeem,
    CompressedNftDelegate,
    CompressedNftMint,
    CompressedNftRedeem,
    CompressedNftSetVerifyCollection,
    CompressedNftTransfer,
    CompressedNftUnverifyCollection,
    CompressedNftUnverifyCreator,
    CompressedNftVerifyCollection,
    CompressedNftVerifyCreator,
    CompressNft,
    CreateAppraisal,
    CreateBet,
    CreateEscrow,
    CreateMasterEdition,
    CreateMerkleTree,
    CreateOrder,
    CreatePool,
    CreateRaffle,
    CreateStore,
    CreateTransaction,
    DeauthorizeFunder,
    DecompressNft,
    DelegateMerkleTree,
    DelistItem,
    Deposit,
    DepositFractionalPool,
    DepositGem,
    DistributeCompressionRewards,
    EmptyPaymentAccount,
    ExecuteTransaction,
    FillOrder,
    FinalizeProgramInstruction,
    ForecloseLoan,
    Fractionalize,
    FundReward,
    Fuse,
    InitAuctionManagerV2,
    InitBank,
    InitFarm,
    InitFarmer,
    InitializeAccount,
    InitRent,
    InitStake,
    InitSwap,
    InitVault,
    KickItem,
    LendForNft,
    ListItem,
    Loan,
    LoanFox,
    LockReward,
    MergeStake,
    MigrateToPnft,
    NftAuctionCancelled,
    NftAuctionCreated,
    NftAuctionUpdated,
    NftBid,
    NftBidCancelled,
    NftCancelListing,
    NftGlobalBid,
    NftGlobalBidCancelled,
    NftListing,
    NftMint,
    NftMintRejected,
    NftParticipationReward,
    NftRentActivate,
    NftRentCancelListing,
    NftRentEnd,
    NftRentListing,
    NftRentUpdateListing,
    NftSale,
    OfferLoan,
    Payout,
    PlaceBet,
    PlaceSolBet,
    PlatformFee,
    ReborrowSolForNft,
    RecordRarityPoints,
    RefreshFarmer,
    RejectSwap,
    RejectTransaction,
    RemoveFromPool,
    RemoveFromWhitelist,
    RepayLoan,
    RequestLoan,
    RequestPnftMigration,
    RescindLoan,
    SetAuthority,
    SetBankFlags,
    SetVaultLock,
    SplitStake,
    StakeSol,
    StakeToken,
    StartPnftMigration,
    Swap,
    SwitchFox,
    SwitchFoxRequest,
    TakeLoan,
    TokenMint,
    Transfer,
    Unknown,
    Unlabeled,
    UnstakeSol,
    UnstakeToken,
    UpdateBankManager,
    UpdateExternalPriceAccount,
    UpdateFarm,
    UpdateItem,
    UpdateOffer,
    UpdateOrder,
    UpdatePrimarySaleMetadata,
    UpdateRaffle,
    UpdateRecordAuthorityData,
    UpdateVaultOwner,
    UpgradeFox,
    UpgradeFoxRequest,
    UpgradeProgramInstruction,
    ValidateSafetyDepositBoxV2,
    WhitelistCreator,
    Withdraw,
    WithdrawGem,
    #[serde(other)]
    Other(String),
}

impl TransactionType {
    pub fn all() -> Vec<Self> {
        vec![
            Self::AcceptEscrowArtist,
            Self::AcceptEscrowUser,
            Self::AcceptRequestArtist,
            Self::ActivateTransaction,
            Self::ActivateVault,
            Self::AddInstruction,
            Self::AddItem,
            Self::AddRaritiesToBank,
            Self::AddTokenToVault,
            Self::AddToPool,
            Self::AddToWhitelist,
            Self::Any,
            Self::ApproveTransaction,
            Self::AttachMetadata,
            Self::AuctionHouseCreate,
            Self::AuctionManagerClaimBid,
            Self::AuthorizeFunder,
            Self::BorrowFox,
            Self::BorrowSolForNft,
            Self::Burn,
            Self::BurnNft,
            Self::BuyItem,
            Self::BuySubscription,
            Self::BuyTickets,
            Self::CancelEscrow,
            Self::CancelLoanRequest,
            Self::CancelOffer,
            Self::CancelOrder,
            Self::CancelReward,
            Self::CancelSwap,
            Self::CancelTransaction,
            Self::CandyMachineRoute,
            Self::CandyMachineUnwrap,
            Self::CandyMachineUpdate,
            Self::CandyMachineWrap,
            Self::ChangeComicState,
            Self::ClaimNft,
            Self::ClaimRewards,
            Self::CloseAccount,
            Self::CloseEscrowAccount,
            Self::CloseItem,
            Self::CloseOrder,
            Self::ClosePosition,
            Self::CompressedNftBurn,
            Self::CompressedNftCancelRedeem,
            Self::CompressedNftDelegate,
            Self::CompressedNftMint,
            Self::CompressedNftRedeem,
            Self::CompressedNftSetVerifyCollection,
            Self::CompressedNftTransfer,
            Self::CompressedNftUnverifyCollection,
            Self::CompressedNftUnverifyCreator,
            Self::CompressedNftVerifyCollection,
            Self::CompressedNftVerifyCreator,
            Self::CompressNft,
            Self::CreateAppraisal,
            Self::CreateBet,
            Self::CreateEscrow,
            Self::CreateMasterEdition,
            Self::CreateMerkleTree,
            Self::CreateOrder,
            Self::CreatePool,
            Self::CreateRaffle,
            Self::CreateStore,
            Self::CreateTransaction,
            Self::DeauthorizeFunder,
            Self::DecompressNft,
            Self::DelegateMerkleTree,
            Self::DelistItem,
            Self::Deposit,
            Self::DepositFractionalPool,
            Self::DepositGem,
            Self::DistributeCompressionRewards,
            Self::EmptyPaymentAccount,
            Self::ExecuteTransaction,
            Self::FillOrder,
            Self::FinalizeProgramInstruction,
            Self::ForecloseLoan,
            Self::Fractionalize,
            Self::FundReward,
            Self::Fuse,
            Self::InitAuctionManagerV2,
            Self::InitBank,
            Self::InitFarm,
            Self::InitFarmer,
            Self::InitializeAccount,
            Self::InitRent,
            Self::InitStake,
            Self::InitSwap,
            Self::InitVault,
            Self::KickItem,
            Self::LendForNft,
            Self::ListItem,
            Self::Loan,
            Self::LoanFox,
            Self::LockReward,
            Self::MergeStake,
            Self::MigrateToPnft,
            Self::NftAuctionCancelled,
            Self::NftAuctionCreated,
            Self::NftAuctionUpdated,
            Self::NftBid,
            Self::NftBidCancelled,
            Self::NftCancelListing,
            Self::NftGlobalBid,
            Self::NftGlobalBidCancelled,
            Self::NftListing,
            Self::NftMint,
            Self::NftMintRejected,
            Self::NftParticipationReward,
            Self::NftRentActivate,
            Self::NftRentCancelListing,
            Self::NftRentEnd,
            Self::NftRentListing,
            Self::NftRentUpdateListing,
            Self::NftSale,
            Self::OfferLoan,
            Self::Payout,
            Self::PlaceBet,
            Self::PlaceSolBet,
            Self::PlatformFee,
            Self::ReborrowSolForNft,
            Self::RecordRarityPoints,
            Self::RefreshFarmer,
            Self::RejectSwap,
            Self::RejectTransaction,
            Self::RemoveFromPool,
            Self::RemoveFromWhitelist,
            Self::RepayLoan,
            Self::RequestLoan,
            Self::RequestPnftMigration,
            Self::RescindLoan,
            Self::SetAuthority,
            Self::SetBankFlags,
            Self::SetVaultLock,
            Self::SplitStake,
            Self::StakeSol,
            Self::StakeToken,
            Self::StartPnftMigration,
            Self::Swap,
            Self::SwitchFox,
            Self::SwitchFoxRequest,
            Self::TakeLoan,
            Self::TokenMint,
            Self::Transfer,
            Self::Unknown,
            Self::Unlabeled,
            Self::UnstakeSol,
            Self::UnstakeToken,
            Self::UpdateBankManager,
            Self::UpdateExternalPriceAccount,
            Self::UpdateFarm,
            Self::UpdateItem,
            Self::UpdateOffer,
            Self::UpdateOrder,
            Self::UpdatePrimarySaleMetadata,
            Self::UpdateRaffle,
            Self::UpdateRecordAuthorityData,
            Self::UpdateVaultOwner,
            Self::UpgradeFox,
            Self::UpgradeFoxRequest,
            Self::UpgradeProgramInstruction,
            Self::ValidateSafetyDepositBoxV2,
            Self::WhitelistCreator,
            Self::Withdraw,
            Self::WithdrawGem,
        ]
    }
}

/// The source protocol or marketplace that originated an enhanced transaction.
///
/// Identifies the on-chain program or platform responsible for a transaction, such as
/// `MagicEden`, `Jupiter`, `Raydium`, or `SystemProgram`. Used for filtering transaction
/// history and labeling swap program sources.
///
/// Serialized as `SCREAMING_SNAKE_CASE`. Unrecognized values are captured in `Other(String)`.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize_enum_str, Serialize_enum_str)]
#[allow(non_camel_case_types)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Source {
    FormFunction,
    ExchangeArt,
    CandyMachineV3,
    CandyMachineV2,
    CandyMachineV1,
    Unknown,
    Solanart,
    Solsea,
    MagicEden,
    Holaplex,
    Metaplex,
    Opensea,
    SolanaProgramLibrary,
    Anchor,
    Phantom,
    SystemProgram,
    StakeProgram,
    Coinbase,
    CoralCube,
    Hedge,
    LaunchMyNft,
    GemBank,
    GemFarm,
    Degods,
    Bsl,
    Yawww,
    Atadia,
    DigitalEyes,
    Hyperspace,
    Tensor,
    Bifrost,
    Jupiter,
    Mecurial,
    Saber,
    Serum,
    StepFinance,
    Cropper,
    Raydium,
    Aldrin,
    Crema,
    Lifinity,
    Cykura,
    Orca,
    Marinade,
    Stepn,
    Sencha,
    Saros,
    EnglishAuction,
    Foxy,
    Hadeswap,
    FoxyStaking,
    FoxyRaffle,
    FoxyTokenMarket,
    FoxyMissions,
    FoxyMarmalade,
    FoxyCoinflip,
    FoxyAuction,
    Citrus,
    Zeta,
    Elixir,
    ElixirLaunchpad,
    CardinalRent,
    CardinalStaking,
    BpfLoader,
    BpfUpgradeableLoader,
    Squads,
    SharkyFi,
    OpenCreatorProtocol,
    Bubblegum,
    W_SOL,
    DUST,
    SOLI,
    USDC,
    FLWR,
    HDG,
    MEAN,
    UXD,
    SHDW,
    POLIS,
    ATLAS,
    USH,
    TRTLS,
    RUNNER,
    INVICTUS,
    #[serde(other)]
    Other(String),
}

/// The Metaplex token standard for an asset.
///
/// Classifies tokens by their fungibility and edition semantics. Unrecognized values
/// are captured in `Other(String)`.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize_enum_str, Serialize_enum_str)]
pub enum TokenStandard {
    Fungible,
    FungibleAsset,
    NonFungible,
    NonFungibleEdition,
    ProgrammableNonFungible,
    UnknownStandard,
    #[serde(other)]
    Other(String),
}

/// The context of an NFT sale event, indicating how the sale was initiated.
#[derive(Clone, Debug, Eq, PartialEq, Serialize_enum_str, Deserialize_enum_str)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionContext {
    Auction,
    InstantSale,
    Offer,
    GlobalOffer,
    Mint,
    Unknown,
    #[serde(other)]
    Other(String),
}

/// The name of a known DEX or swap program, used to label inner swap hops.
///
/// Unrecognized programs are captured in `Other(String)`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize_enum_str, Deserialize_enum_str)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProgramName {
    Unkown,
    JupiterV1,
    JupiterV2,
    JupiterV3,
    JupiterV4,
    MercurialStableSwap,
    SaberStableSwap,
    SaberExchange,
    SerumDexV1,
    SerumDexV2,
    SerumDexV3,
    SerumSwap,
    StepFinance,
    Cropper,
    RaydiumLiquidityPoolV2,
    RaydiumLiquidityPoolV3,
    RaydiumLiquidityPoolV4,
    AldrinAmmV1,
    AldrinAmmV2,
    Crema,
    Lifinity,
    LifinityV2,
    Cykura,
    OrcaTokenSwapV1,
    OrcaTokenSwapV2,
    OrcaWhirlpools,
    Marinade,
    Stepn,
    SenchaExchange,
    SarosAmm,
    FoxyStake,
    FoxySwap,
    FoxyRaffle,
    FoxyTokenMarket,
    FoxyMissions,
    FoxyMarmalade,
    FoxyCoinflip,
    FoxyAuction,
    Citrus,
    HadeSwap,
    Zeta,
    CardinalRent,
    CardinalStaking,
    SharkyFi,
    OpenCreatorProtocol,
    Bubblegum,
    CoralCube,
    #[serde(other)]
    Other(String),
}

/// Filter for transaction success/failure status.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    #[default]
    All,
    Success,
    Failed,
}

/// The type of webhook, determining the data format and target cluster.
#[derive(Deserialize, Serialize, Eq, PartialEq, Clone, Debug, Default)]
pub enum WebhookType {
    #[serde(rename = "enhanced")]
    #[default]
    Enhanced,
    #[serde(rename = "enhancedDevnet")]
    EnhancedDevnet,
    #[serde(rename = "raw")]
    Raw,
    #[serde(rename = "rawDevnet")]
    RawDevnet,
    #[serde(rename = "discord")]
    Discord,
    #[serde(rename = "discordDevnet")]
    DiscordDevnet,
}

/// The encoding format for account data in webhook payloads.
#[derive(Clone, Debug, Deserialize_enum_str, Serialize_enum_str, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AccountWebhookEncoding {
    #[default]
    JsonParsed,
    #[serde(other)]
    Other(String),
}

/// An identifier for an NFT collection, used to query collections by creator or verified address.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CollectionIdentifier {
    #[serde(rename = "firstVerifiedCreators")]
    FirstVerifiedCreators(Vec<String>),
    #[serde(rename = "verifiedCollectionAddress")]
    VerifiedCollectionAddress(Vec<String>),
}

/// A Solana transaction that can be either a legacy or versioned format.
///
/// Used as input to the smart transaction sending methods.
#[derive(Serialize, Deserialize, Debug)]
pub enum SmartTransaction {
    Legacy(Transaction),
    Versioned(VersionedTransaction),
}

/// The encoding format for account data in RPC V2 responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Encoding {
    #[serde(rename = "jsonParsed")]
    JsonParsed,
    #[serde(rename = "base58")]
    Base58,
    #[serde(rename = "base64")]
    Base64,
    #[serde(rename = "base64+zstd")]
    Base64Zstd,
}

/// A filter for the `getProgramAccounts` RPC method.
///
/// Allows filtering accounts by data size or by matching bytes at a specific offset.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GpaFilter {
    DataSize {
        #[serde(rename = "dataSize")]
        data_size: u64,
    },
    Memcmp {
        memcmp: GpaMemcmp,
    },
}

/// A filter for the `getTokenAccountsByOwner` RPC method.
///
/// Restricts results to token accounts for a specific mint or owned by a specific program.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TokenAccountsOwnerFilter {
    Mint {
        mint: String,
    },
    Program {
        #[serde(rename = "programId")]
        program_id: String,
    },
}
