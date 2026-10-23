#![allow(deprecated)] // TODO: remove with SType::Smart

use std::ffi::c_char;

use super::{
    fields::{ENUM_16_ID, ENUM_8_ID, ENUM_CHAR_ID},
    types::EnumField,
};

impl EnumField for crate::enums::RType {
    type Repr = u8;
    type Storage = u8;
    const TYPE_ID: u8 = ENUM_8_ID;
    const NAME: &'static str = "RType";
    const LABELS: &'static [(u16, &'static str)] = &[
        (Self::Mbp0 as u16, "MBP_0"),
        (Self::Mbp1 as u16, "MBP_1"),
        (Self::Mbp10 as u16, "MBP_10"),
        (Self::OhlcvDeprecated as u16, "OHLCV_DEPRECATED"),
        (Self::Ohlcv1S as u16, "OHLCV_1S"),
        (Self::Ohlcv1M as u16, "OHLCV_1M"),
        (Self::Ohlcv1H as u16, "OHLCV_1H"),
        (Self::Ohlcv1D as u16, "OHLCV_1D"),
        (Self::OhlcvEod as u16, "OHLCV_EOD"),
        (Self::Status as u16, "STATUS"),
        (Self::InstrumentDef as u16, "INSTRUMENT_DEF"),
        (Self::Imbalance as u16, "IMBALANCE"),
        (Self::Error as u16, "ERROR"),
        (Self::SymbolMapping as u16, "SYMBOL_MAPPING"),
        (Self::System as u16, "SYSTEM"),
        (Self::Statistics as u16, "STATISTICS"),
        (Self::Mbo as u16, "MBO"),
        (Self::Cmbp1 as u16, "CMBP_1"),
        (Self::Cbbo1S as u16, "CBBO_1S"),
        (Self::Cbbo1M as u16, "CBBO_1M"),
        (Self::Tcbbo as u16, "TCBBO"),
        (Self::Bbo1S as u16, "BBO_1S"),
        (Self::Bbo1M as u16, "BBO_1M"),
        (Self::Tbbo as u16, "TBBO"),
    ];
}

impl EnumField for crate::enums::Side {
    type Repr = u8;
    type Storage = c_char;
    const TYPE_ID: u8 = ENUM_CHAR_ID;
    const NAME: &'static str = "Side";
    const LABELS: &'static [(u16, &'static str)] = &[
        (Self::Ask as u16, "ASK"),
        (Self::Bid as u16, "BID"),
        (Self::None as u16, "NONE"),
    ];
}

impl EnumField for crate::enums::Action {
    type Repr = u8;
    type Storage = c_char;
    const TYPE_ID: u8 = ENUM_CHAR_ID;
    const NAME: &'static str = "Action";
    const LABELS: &'static [(u16, &'static str)] = &[
        (Self::Modify as u16, "MODIFY"),
        (Self::Trade as u16, "TRADE"),
        (Self::Fill as u16, "FILL"),
        (Self::Cancel as u16, "CANCEL"),
        (Self::Add as u16, "ADD"),
        (Self::Clear as u16, "CLEAR"),
        (Self::None as u16, "NONE"),
    ];
}

impl EnumField for crate::enums::InstrumentClass {
    type Repr = u8;
    type Storage = c_char;
    const TYPE_ID: u8 = ENUM_CHAR_ID;
    const NAME: &'static str = "InstrumentClass";
    const LABELS: &'static [(u16, &'static str)] = &[
        (Self::Bond as u16, "BOND"),
        (Self::Call as u16, "CALL"),
        (Self::Future as u16, "FUTURE"),
        (Self::Index as u16, "INDEX"),
        (Self::Stock as u16, "STOCK"),
        (Self::MixedSpread as u16, "MIXED_SPREAD"),
        (Self::Put as u16, "PUT"),
        (Self::FutureSpread as u16, "FUTURE_SPREAD"),
        (Self::OptionSpread as u16, "OPTION_SPREAD"),
        (Self::FxSpot as u16, "FX_SPOT"),
        (Self::CommoditySpot as u16, "COMMODITY_SPOT"),
    ];
}

impl EnumField for crate::enums::MatchAlgorithm {
    type Repr = u8;
    type Storage = c_char;
    const TYPE_ID: u8 = ENUM_CHAR_ID;
    const NAME: &'static str = "MatchAlgorithm";
    const LABELS: &'static [(u16, &'static str)] = &[
        (Self::Undefined as u16, "UNDEFINED"),
        (Self::Fifo as u16, "FIFO"),
        (Self::Configurable as u16, "CONFIGURABLE"),
        (Self::ProRata as u16, "PRO_RATA"),
        (Self::FifoLmm as u16, "FIFO_LMM"),
        (Self::ThresholdProRata as u16, "THRESHOLD_PRO_RATA"),
        (Self::FifoTopLmm as u16, "FIFO_TOP_LMM"),
        (Self::ThresholdProRataLmm as u16, "THRESHOLD_PRO_RATA_LMM"),
        (Self::EurodollarFutures as u16, "EURODOLLAR_FUTURES"),
        (Self::TimeProRata as u16, "TIME_PRO_RATA"),
        (
            Self::InstitutionalPrioritization as u16,
            "INSTITUTIONAL_PRIORITIZATION",
        ),
        (Self::Allocation as u16, "ALLOCATION"),
    ];
}

impl EnumField for crate::enums::UserDefinedInstrument {
    type Repr = u8;
    type Storage = c_char;
    const TYPE_ID: u8 = ENUM_CHAR_ID;
    const NAME: &'static str = "UserDefinedInstrument";
    const LABELS: &'static [(u16, &'static str)] =
        &[(Self::No as u16, "NO"), (Self::Yes as u16, "YES")];
}

impl EnumField for crate::enums::SecurityUpdateAction {
    type Repr = u8;
    type Storage = c_char;
    const TYPE_ID: u8 = ENUM_CHAR_ID;
    const NAME: &'static str = "SecurityUpdateAction";
    const LABELS: &'static [(u16, &'static str)] = &[
        (Self::Add as u16, "ADD"),
        (Self::Modify as u16, "MODIFY"),
        (Self::Delete as u16, "DELETE"),
        (Self::Invalid as u16, "INVALID"),
    ];
}

impl EnumField for crate::enums::SType {
    type Repr = u8;
    type Storage = u8;
    const TYPE_ID: u8 = ENUM_8_ID;
    const NAME: &'static str = "SType";
    const LABELS: &'static [(u16, &'static str)] = &[
        (Self::InstrumentId as u16, "INSTRUMENT_ID"),
        (Self::RawSymbol as u16, "RAW_SYMBOL"),
        (Self::Smart as u16, "SMART"),
        (Self::Continuous as u16, "CONTINUOUS"),
        (Self::Parent as u16, "PARENT"),
        (Self::NasdaqSymbol as u16, "NASDAQ_SYMBOL"),
        (Self::CmsSymbol as u16, "CMS_SYMBOL"),
        (Self::Isin as u16, "ISIN"),
        (Self::UsCode as u16, "US_CODE"),
        (Self::BbgCompId as u16, "BBG_COMP_ID"),
        (Self::BbgCompTicker as u16, "BBG_COMP_TICKER"),
        (Self::Figi as u16, "FIGI"),
        (Self::FigiTicker as u16, "FIGI_TICKER"),
        (Self::ListingId as u16, "LISTING_ID"),
        (Self::IssuerId as u16, "ISSUER_ID"),
        (Self::SecurityId as u16, "SECURITY_ID"),
    ];
}

impl EnumField for crate::enums::Schema {
    type Repr = u16;
    type Storage = u16;
    const TYPE_ID: u8 = ENUM_16_ID;
    const NAME: &'static str = "Schema";
    const LABELS: &'static [(u16, &'static str)] = &[
        (Self::Mbo as u16, "MBO"),
        (Self::Mbp1 as u16, "MBP_1"),
        (Self::Mbp10 as u16, "MBP_10"),
        (Self::Tbbo as u16, "TBBO"),
        (Self::Trades as u16, "TRADES"),
        (Self::Ohlcv1S as u16, "OHLCV_1S"),
        (Self::Ohlcv1M as u16, "OHLCV_1M"),
        (Self::Ohlcv1H as u16, "OHLCV_1H"),
        (Self::Ohlcv1D as u16, "OHLCV_1D"),
        (Self::Definition as u16, "DEFINITION"),
        (Self::Statistics as u16, "STATISTICS"),
        (Self::Status as u16, "STATUS"),
        (Self::Imbalance as u16, "IMBALANCE"),
        (Self::OhlcvEod as u16, "OHLCV_EOD"),
        (Self::Cmbp1 as u16, "CMBP_1"),
        (Self::Cbbo1S as u16, "CBBO_1S"),
        (Self::Cbbo1M as u16, "CBBO_1M"),
        (Self::Tcbbo as u16, "TCBBO"),
        (Self::Bbo1S as u16, "BBO_1S"),
        (Self::Bbo1M as u16, "BBO_1M"),
    ];
}

impl EnumField for crate::enums::Encoding {
    type Repr = u8;
    type Storage = u8;
    const TYPE_ID: u8 = ENUM_8_ID;
    const NAME: &'static str = "Encoding";
    const LABELS: &'static [(u16, &'static str)] = &[
        (Self::Dbn as u16, "DBN"),
        (Self::Csv as u16, "CSV"),
        (Self::Json as u16, "JSON"),
    ];
}

impl EnumField for crate::enums::Compression {
    type Repr = u8;
    type Storage = u8;
    const TYPE_ID: u8 = ENUM_8_ID;
    const NAME: &'static str = "Compression";
    const LABELS: &'static [(u16, &'static str)] =
        &[(Self::None as u16, "NONE"), (Self::Zstd as u16, "ZSTD")];
}

impl EnumField for crate::enums::StatType {
    type Repr = u16;
    type Storage = u16;
    const TYPE_ID: u8 = ENUM_16_ID;
    const NAME: &'static str = "StatType";
    const LABELS: &'static [(u16, &'static str)] = &[
        (Self::OpeningPrice as u16, "OPENING_PRICE"),
        (
            Self::IndicativeOpeningPrice as u16,
            "INDICATIVE_OPENING_PRICE",
        ),
        (Self::SettlementPrice as u16, "SETTLEMENT_PRICE"),
        (
            Self::TradingSessionLowPrice as u16,
            "TRADING_SESSION_LOW_PRICE",
        ),
        (
            Self::TradingSessionHighPrice as u16,
            "TRADING_SESSION_HIGH_PRICE",
        ),
        (Self::ClearedVolume as u16, "CLEARED_VOLUME"),
        (Self::LowestOffer as u16, "LOWEST_OFFER"),
        (Self::HighestBid as u16, "HIGHEST_BID"),
        (Self::OpenInterest as u16, "OPEN_INTEREST"),
        (Self::FixingPrice as u16, "FIXING_PRICE"),
        (Self::ClosePrice as u16, "CLOSE_PRICE"),
        (Self::NetChange as u16, "NET_CHANGE"),
        (Self::Vwap as u16, "VWAP"),
        (Self::Volatility as u16, "VOLATILITY"),
        (Self::Delta as u16, "DELTA"),
        (Self::UncrossingPrice as u16, "UNCROSSING_PRICE"),
        (Self::UpperPriceLimit as u16, "UPPER_PRICE_LIMIT"),
        (Self::LowerPriceLimit as u16, "LOWER_PRICE_LIMIT"),
        (Self::BlockVolume as u16, "BLOCK_VOLUME"),
        (Self::IndicativeClosePrice as u16, "INDICATIVE_CLOSE_PRICE"),
        (Self::MwcbLevel1 as u16, "MWCB_LEVEL_1"),
        (Self::MwcbLevel2 as u16, "MWCB_LEVEL_2"),
        (Self::MwcbLevel3 as u16, "MWCB_LEVEL_3"),
        (
            Self::AuctionCollarReferencePrice as u16,
            "AUCTION_COLLAR_REFERENCE_PRICE",
        ),
        (
            Self::AuctionCollarUpperPrice as u16,
            "AUCTION_COLLAR_UPPER_PRICE",
        ),
        (
            Self::AuctionCollarLowerPrice as u16,
            "AUCTION_COLLAR_LOWER_PRICE",
        ),
        (Self::VenueSpecificVolume1 as u16, "VENUE_SPECIFIC_VOLUME_1"),
        (Self::VenueSpecificPrice1 as u16, "VENUE_SPECIFIC_PRICE_1"),
    ];
}

impl EnumField for crate::enums::StatUpdateAction {
    type Repr = u8;
    type Storage = u8;
    const TYPE_ID: u8 = ENUM_8_ID;
    const NAME: &'static str = "StatUpdateAction";
    const LABELS: &'static [(u16, &'static str)] =
        &[(Self::New as u16, "NEW"), (Self::Delete as u16, "DELETE")];
}

impl EnumField for crate::enums::StatusAction {
    type Repr = u16;
    type Storage = u16;
    const TYPE_ID: u8 = ENUM_16_ID;
    const NAME: &'static str = "StatusAction";
    const LABELS: &'static [(u16, &'static str)] = &[
        (Self::None as u16, "NONE"),
        (Self::PreOpen as u16, "PRE_OPEN"),
        (Self::PreCross as u16, "PRE_CROSS"),
        (Self::Quoting as u16, "QUOTING"),
        (Self::Cross as u16, "CROSS"),
        (Self::Rotation as u16, "ROTATION"),
        (Self::NewPriceIndication as u16, "NEW_PRICE_INDICATION"),
        (Self::Trading as u16, "TRADING"),
        (Self::Halt as u16, "HALT"),
        (Self::Pause as u16, "PAUSE"),
        (Self::Suspend as u16, "SUSPEND"),
        (Self::PreClose as u16, "PRE_CLOSE"),
        (Self::Close as u16, "CLOSE"),
        (Self::PostClose as u16, "POST_CLOSE"),
        (Self::SsrChange as u16, "SSR_CHANGE"),
        (
            Self::NotAvailableForTrading as u16,
            "NOT_AVAILABLE_FOR_TRADING",
        ),
    ];
}

impl EnumField for crate::enums::StatusReason {
    type Repr = u16;
    type Storage = u16;
    const TYPE_ID: u8 = ENUM_16_ID;
    const NAME: &'static str = "StatusReason";
    const LABELS: &'static [(u16, &'static str)] = &[
        (Self::None as u16, "NONE"),
        (Self::Scheduled as u16, "SCHEDULED"),
        (
            Self::SurveillanceIntervention as u16,
            "SURVEILLANCE_INTERVENTION",
        ),
        (Self::MarketEvent as u16, "MARKET_EVENT"),
        (Self::InstrumentActivation as u16, "INSTRUMENT_ACTIVATION"),
        (Self::InstrumentExpiration as u16, "INSTRUMENT_EXPIRATION"),
        (Self::RecoveryInProcess as u16, "RECOVERY_IN_PROCESS"),
        (Self::Regulatory as u16, "REGULATORY"),
        (Self::Administrative as u16, "ADMINISTRATIVE"),
        (Self::NonCompliance as u16, "NON_COMPLIANCE"),
        (Self::FilingsNotCurrent as u16, "FILINGS_NOT_CURRENT"),
        (Self::SecTradingSuspension as u16, "SEC_TRADING_SUSPENSION"),
        (Self::NewIssue as u16, "NEW_ISSUE"),
        (Self::IssueAvailable as u16, "ISSUE_AVAILABLE"),
        (Self::IssuesReviewed as u16, "ISSUES_REVIEWED"),
        (Self::FilingReqsSatisfied as u16, "FILING_REQS_SATISFIED"),
        (Self::NewsPending as u16, "NEWS_PENDING"),
        (Self::NewsReleased as u16, "NEWS_RELEASED"),
        (
            Self::NewsAndResumptionTimes as u16,
            "NEWS_AND_RESUMPTION_TIMES",
        ),
        (Self::NewsNotForthcoming as u16, "NEWS_NOT_FORTHCOMING"),
        (Self::OrderImbalance as u16, "ORDER_IMBALANCE"),
        (Self::LuldPause as u16, "LULD_PAUSE"),
        (Self::Operational as u16, "OPERATIONAL"),
        (
            Self::AdditionalInformationRequested as u16,
            "ADDITIONAL_INFORMATION_REQUESTED",
        ),
        (Self::MergerEffective as u16, "MERGER_EFFECTIVE"),
        (Self::Etf as u16, "ETF"),
        (Self::CorporateAction as u16, "CORPORATE_ACTION"),
        (Self::NewSecurityOffering as u16, "NEW_SECURITY_OFFERING"),
        (Self::MarketWideHaltLevel1 as u16, "MARKET_WIDE_HALT_LEVEL1"),
        (Self::MarketWideHaltLevel2 as u16, "MARKET_WIDE_HALT_LEVEL2"),
        (Self::MarketWideHaltLevel3 as u16, "MARKET_WIDE_HALT_LEVEL3"),
        (
            Self::MarketWideHaltCarryover as u16,
            "MARKET_WIDE_HALT_CARRYOVER",
        ),
        (
            Self::MarketWideHaltResumption as u16,
            "MARKET_WIDE_HALT_RESUMPTION",
        ),
        (
            Self::QuotationNotAvailable as u16,
            "QUOTATION_NOT_AVAILABLE",
        ),
    ];
}

impl EnumField for crate::enums::TradingEvent {
    type Repr = u16;
    type Storage = u16;
    const TYPE_ID: u8 = ENUM_16_ID;
    const NAME: &'static str = "TradingEvent";
    const LABELS: &'static [(u16, &'static str)] = &[
        (Self::None as u16, "NONE"),
        (Self::NoCancel as u16, "NO_CANCEL"),
        (Self::ChangeTradingSession as u16, "CHANGE_TRADING_SESSION"),
        (Self::ImpliedMatchingOn as u16, "IMPLIED_MATCHING_ON"),
        (Self::ImpliedMatchingOff as u16, "IMPLIED_MATCHING_OFF"),
    ];
}

impl EnumField for crate::enums::TriState {
    type Repr = u8;
    type Storage = c_char;
    const TYPE_ID: u8 = ENUM_CHAR_ID;
    const NAME: &'static str = "TriState";
    const LABELS: &'static [(u16, &'static str)] = &[
        (Self::NotAvailable as u16, "NOT_AVAILABLE"),
        (Self::No as u16, "NO"),
        (Self::Yes as u16, "YES"),
    ];
}

impl EnumField for crate::enums::VersionUpgradePolicy {
    type Repr = u8;
    type Storage = u8;
    const TYPE_ID: u8 = ENUM_8_ID;
    const NAME: &'static str = "VersionUpgradePolicy";
    const LABELS: &'static [(u16, &'static str)] = &[
        (Self::AsIs as u16, "AS_IS"),
        (Self::UpgradeToV2 as u16, "UPGRADE_TO_V2"),
        (Self::UpgradeToV3 as u16, "UPGRADE_TO_V3"),
        (Self::UpgradeToV4 as u16, "UPGRADE_TO_V4"),
        (Self::ExpectV4 as u16, "EXPECT_V4"),
    ];
}

impl EnumField for crate::enums::ErrorCode {
    type Repr = u8;
    type Storage = u8;
    const TYPE_ID: u8 = ENUM_8_ID;
    const NAME: &'static str = "ErrorCode";
    const LABELS: &'static [(u16, &'static str)] = &[
        (Self::AuthFailed as u16, "AUTH_FAILED"),
        (Self::ApiKeyDeactivated as u16, "API_KEY_DEACTIVATED"),
        (
            Self::ConnectionLimitExceeded as u16,
            "CONNECTION_LIMIT_EXCEEDED",
        ),
        (
            Self::SymbolResolutionFailed as u16,
            "SYMBOL_RESOLUTION_FAILED",
        ),
        (Self::InvalidSubscription as u16, "INVALID_SUBSCRIPTION"),
        (Self::InternalError as u16, "INTERNAL_ERROR"),
        (
            Self::SkippedRecordsAfterSlowReading as u16,
            "SKIPPED_RECORDS_AFTER_SLOW_READING",
        ),
        (Self::ReplayDataAgedOut as u16, "REPLAY_DATA_AGED_OUT"),
        (Self::Unset as u16, "UNSET"),
    ];
}

impl EnumField for crate::enums::SystemCode {
    type Repr = u8;
    type Storage = u8;
    const TYPE_ID: u8 = ENUM_8_ID;
    const NAME: &'static str = "SystemCode";
    const LABELS: &'static [(u16, &'static str)] = &[
        (Self::Heartbeat as u16, "HEARTBEAT"),
        (Self::SubscriptionAck as u16, "SUBSCRIPTION_ACK"),
        (Self::SlowReaderWarning as u16, "SLOW_READER_WARNING"),
        (Self::ReplayCompleted as u16, "REPLAY_COMPLETED"),
        (Self::EndOfInterval as u16, "END_OF_INTERVAL"),
        (Self::UnsubscribeAck as u16, "UNSUBSCRIBE_ACK"),
        (Self::Unset as u16, "UNSET"),
    ];
}
