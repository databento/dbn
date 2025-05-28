# ruff: noqa: UP007 PYI021 PYI011
from __future__ import annotations

import datetime as dt
from collections.abc import Iterable
from collections.abc import Sequence
from enum import Enum
from typing import BinaryIO
from typing import ClassVar
from typing import Optional
from typing import SupportsBytes
from typing import TextIO
from typing import Union

from databento_dbn import MappingIntervalDict
from databento_dbn import SymbolMapping

DBN_VERSION: int
FIXED_PRICE_SCALE: int
UNDEF_PRICE: int
UNDEF_ORDER_SIZE: int
UNDEF_STAT_QUANTITY: int
UNDEF_TIMESTAMP: int

_DBNRecord = Union[
    Metadata,
    MBOMsg,
    MBP1Msg,
    BBOMsg,
    CMBP1Msg,
    CBBOMsg,
    MBP10Msg,
    OHLCVMsg,
    TradeMsg,
    InstrumentDefMsgV1,
    InstrumentDefMsgV2,
    InstrumentDefMsg,
    ImbalanceMsg,
    ErrorMsg,
    ErrorMsgV1,
    SymbolMappingMsg,
    SymbolMappingMsgV1,
    SystemMsg,
    SystemMsgV1,
    StatMsgV1,
    StatMsg,
    StatusMsg,
]

class DBNError(Exception):
    """
    An exception from databento_dbn Rust code.
    """

class Side(Enum):
    """
    A side of the market. The side of the market for resting orders, or the side
    of the aggressor for trades.

    ASK
        A sell order or sell aggressor in a trade.
    BID
        A buy order or a buy aggressor in a trade.
    NONE
        No side specified by the original source.

    """

    ASK: str
    BID: str
    NONE: str

    @classmethod
    def from_str(cls, value: str) -> Side: ...
    @classmethod
    def variants(cls) -> Iterable[Side]: ...

class Action(Enum):
    """
    A tick action.

    MODIFY
        An existing order was modified.
    TRADE
        A trade executed.
    FILL
        An existing order was filled.
    CANCEL
        An order was cancelled.
    ADD
        A new order was added.
    CLEAR
        Reset the book; clear all orders for an instrument.
    NONE
        Has no effect on the book, but may carry `flags` or other information.
    """

    MODIFY: str
    TRADE: str
    FILL: str
    CANCEL: str
    ADD: str
    CLEAR: str
    NONE: str

    @classmethod
    def from_str(cls, value: str) -> Action: ...
    @classmethod
    def variants(cls) -> Iterable[Action]: ...

class InstrumentClass(Enum):
    """
    The class of instrument.

    BOND
        A bond.
    CALL
        A call option.
    FUTURE
        A future.
    STOCK
        A stock.
    MIXED_SPREAD
        A spread composed of multiple instrument classes.
    PUT
        A put option.
    FUTURE_SPREAD
        A spread composed of futures.
    OPTION_SPREAD
        A spread composed of options.
    FX_SPOT
        A foreign exchange spot.
    COMMODITY_SPOT
        A commodity being traded for immediate delivery.

    """

    BOND: str
    CALL: str
    FUTURE: str
    STOCK: str
    MIXED_SPREAD: str
    PUT: str
    FUTURE_SPREAD: str
    OPTION_SPREAD: str
    FX_SPOT: str
    COMMODITY_SPOT: str

    @classmethod
    def from_str(cls, value: str) -> InstrumentClass: ...
    @classmethod
    def variants(cls) -> Iterable[InstrumentClass]: ...

class MatchAlgorithm(Enum):
    """
    The type of matching algorithm used for the instrument at the exchange.


    UNDEFINED
        No matching algorithm was specified.
    FIFO
        First-in-first-out matching.
    CONFIGURABLE
        A configurable match algorithm.
    PRO_RATA
        Trade quantity is allocated to resting orders based on a pro-rata percentage: resting order quantity divided by total quantity.
    FIFO_LMM
        Like `FIFO` but with LMM allocations prior to FIFO allocations.
    THRESHOLD_PRO_RATA
        Like `PRO_RATA` but includes a configurable allocation to the first order that improves the market.
    FIFO_TOP_LMM
        Like `FIFO_LMM` but includes a configurable allocation to the first order that improves the market.
    THRESHOLD_PRO_RATA_LMM
        Like `THRESHOLD_PRO_RATA` but includes a special priority to LMMs.
    EURODOLLAR_FUTURES
        Special variant used only for Eurodollar futures on CME.
    TIME_PRO_RATA
        Trade quantity is shared between all orders at the best price. Orders with the
        highest time priority receive a higher matched quantity.
    INSTITUTIONAL_PRIORITIZATION
        A two-pass FIFO algorithm. The first pass fills the Institutional Group the aggressing
        order is associated with. The second pass matches orders without an Institutional Group
        association.
    """

    UNDEFINED: str
    FIFO: str
    CONFIGURABLE: str
    PRO_RATA: str
    FIFO_LMM: str
    THRESHOLD_PRO_RATA: str
    FIFO_TOP_LMM: str
    THRESHOLD_PRO_RATA_LMM: str
    EURODOLLAR_FUTURES: str
    TIME_PRO_RATA: str
    INSTITUTIONAL_PRIORITIZATION: str

    @classmethod
    def from_str(cls, value: str) -> MatchAlgorithm: ...
    @classmethod
    def variants(cls) -> Iterable[MatchAlgorithm]: ...

class UserDefinedInstrument(Enum):
    """
    Whether the instrument is user-defined.

    NO
        The instrument is not user-defined.
    YES
        The instrument is user-defined.

    """

    NO: str
    YES: str

    @classmethod
    def from_str(cls, value: str) -> UserDefinedInstrument: ...
    @classmethod
    def variants(cls) -> Iterable[UserDefinedInstrument]: ...

class SType(Enum):
    """
    A DBN symbology type.

    INSTRUMENT_ID
        Symbology using a unique numeric ID.
    RAW_SYMBOL
        Symbology using the original symbols provided by the publisher.
    CONTINUOUS
        A Databento-specific symbology where one symbol may point to different
        instruments at different points of time, e.g. to always refer to the front month
        future.
    PARENT
        A Databento-specific symbology for referring to a group of symbols by one
        "parent" symbol, e.g. ES.FUT to refer to all ES futures.
    NASDAQ_SYMBOL
        Symbology for US equities using NASDAQ Integrated suffix conventions.
    CMS_SYMBOL
        Symbology for US equities using CMS suffix conventions.
    ISIN
        Symbology using International Security Identification Numbers (ISIN) - ISO 6166.
    US_CODE
        Symbology using US domestic Committee on Uniform Securities Identification Procedure (CUSIP) codes.
    BBG_COMP_ID
        Symbology using Bloomberg composite global IDs.
    BBG_COMP_TICKER
        Symbology using Bloomberg composite tickers.
    FIGI
        Symbology using Bloomberg FIGI exchange level IDs.
    FIGI_TICKER
        Symbology using Bloomberg exchange level tickers.

    """

    INSTRUMENT_ID: str
    RAW_SYMBOL: str
    CONTINUOUS: str
    PARENT: str
    NASDAQ_SYMBOL: str
    CMS_SYMBOL: str
    ISIN: str
    US_CODE: str
    BBG_COMP_ID: str
    BBG_COMP_TICKER: str
    FIGI: str
    FIGI_TICKER: str

    @classmethod
    def from_str(cls, value: str) -> SType: ...
    @classmethod
    def variants(cls) -> Iterable[SType]: ...

class RType(Enum):
    """
    A DBN record type.

    MBP_0
        Denotes a market-by-price record with a book depth of 0 (used for the `Trades` schema).
    MBP_1
        Denotes a market-by-price record with a book depth of 1 (also used for the
        `Tbbo` schema).
    MBP_10
        Denotes a market-by-price record with a book depth of 10.
    OHLCV_DEPRECATED
        Denotes an open, high, low, close, and volume record at an unspecified cadence.
    OHLCV_1S
        Denotes an open, high, low, close, and volume record at a 1-second cadence.
    OHLCV_1M
        Denotes an open, high, low, close, and volume record at a 1-minute cadence.
    OHLCV_1H
        Denotes an open, high, low, close, and volume record at an hourly cadence.
    OHLCV_1D
        Denotes an open, high, low, close, and volume record at a daily cadence
        based on the UTC date.
    OHLCV_EOD
        Denotes an open, high, low, close, and volume record at a daily cadence
        based on the end of the trading session.
    STATUS
        Denotes an exchange status record.
    INSTRUMENT_DEF
        Denotes an instrument definition record.
    IMBALANCE
        Denotes an order imbalance record.
    ERROR
        Denotes an error from gateway.
    SYMBOL_MAPPING
        Denotes a symbol mapping record.
    SYSTEM
        Denotes a non-error message from the gateway. Also used for heartbeats.
    STATISTICS
        Denotes a statistics record from the publisher (not calculated by Databento).
    MBO
        Denotes a market by order record.
    CMBP_1
        Denotes a consolidated best bid and offer record.
    CBBO_1S
        Denotes a consolidated best bid and offer record.
    CBBO_1M
        Denotes a consolidated best bid and offer record subsampled on a one-minute
        interval.
    TCBBO
        Denotes a consolidated best bid and offer trade record containing the
        consolidated BBO before the trade.
    BBO_1S
        Denotes a best bid and offer record subsampled on a one-second interval.
    BBO_1M
        Denotes a best bid and offer record subsampled on a one-minute interval.

    """  # noqa: D405, D411

    MBP_0: int
    MBP_1: int
    MBP_10: int
    OHLCV_DEPRECATED: int
    OHLCV_1S: int
    OHLCV_1M: int
    OHLCV_1H: int
    OHLCV_1D: int
    OHLCV_EOD: int
    STATUS: int
    INSTRUMENT_DEF: int
    IMBALANCE: int
    ERROR: int
    SYMBOL_MAPPING: int
    SYSTEM: int
    STATISTICS: int
    MBO: int
    CMBP_1: int
    CBBO_1S: int
    CBBO_1M: int
    TCBBO: int
    BBO_1S: int
    BBO_1M: int

    @classmethod
    def from_int(cls, value: int) -> RType: ...
    @classmethod
    def from_schema(cls, value: Schema) -> RType: ...
    @classmethod
    def from_str(cls, value: str) -> RType: ...
    @classmethod
    def variants(cls) -> Iterable[RType]: ...

class Schema(Enum):
    """
    A DBN record schema.

    MBO
        Market by order.
    MBP_1
        Market by price with a book depth of 1.
    MBP_10
        Market by price with a book depth of 10.
    TBBO
        All trade events with the best bid and offer (BBO) immediately before the effect of the trade.
    TRADES
        All trade events.
    OHLCV_1S
        Open, high, low, close, and volume at a one-second interval.
    OHLCV_1M
        Open, high, low, close, and volume at a one-minute interval.
    OHLCV_1H
        Open, high, low, close, and volume at an hourly interval.
    OHLCV_1D
        Open, high, low, close, and volume at a daily interval.
    OHLCV_EOD
        Open, high, low, close, and volume at a daily cadence based on the end of the trading session.
    DEFINITION
        Instrument definitions.
    STATISTICS
        Additional data disseminated by publishers.
    STATUS
        Exchange status.
    IMBALANCE
        Auction imbalance events.
    CMBP_1
        Consolidated best bid and offer.
    CBBO_1S
        Consolidated best bid and offer record.
    CBBO_1M
        Consolidated best bid and offer record subsampled on a one-second interval.
    TCBBO
        Consolidated best bid and offer record subsampled on a one-minute interval.
    BBO_1S
        Consolidated best bid and offer trade record containing the consolidated BBO before the trade.
    BBO_1M
        Best bid and offer record subsampled on a one-second interval.

    """

    MBO: str
    MBP_1: str
    MBP_10: str
    TBBO: str
    TRADES: str
    OHLCV_1S: str
    OHLCV_1M: str
    OHLCV_1H: str
    OHLCV_1D: str
    OHLCV_EOD: str
    DEFINITION: str
    STATISTICS: str
    STATUS: str
    IMBALANCE: str
    CMBP_1: str
    CBBO_1S: str
    CBBO_1M: str
    TCBBO: str
    BBO_1S: str
    BBO_1M: str

    @classmethod
    def from_str(cls, value: str) -> Schema: ...
    @classmethod
    def variants(cls) -> Iterable[Schema]: ...

class Encoding(Enum):
    """
    Data output encoding.

    DBN
        Databento Binary Encoding.
    CSV
        Comma-separated values.
    JSON
        JavaScript object notation.

    """

    DBN: str
    CSV: str
    JSON: str

    @classmethod
    def from_str(cls, value: str) -> Encoding: ...
    @classmethod
    def variants(cls) -> Iterable[Encoding]: ...

class Compression(Enum):
    """
    Data compression format.

    NONE
        Uncompressed.
    ZSTD
        Zstandard compressed.

    """

    NONE: str
    ZSTD: str

    @classmethod
    def from_str(cls, value: str) -> Compression: ...
    @classmethod
    def variants(cls) -> Iterable[Compression]: ...

class SecurityUpdateAction(Enum):
    """
    The type of definition update.

    ADD
        A new instrument definition.
    MODIFY
        A modified instrument definition of an existing one.
    DELETE
        Removal of an instrument definition.
    INVALID
        Deprecated

    """

    ADD: str
    MODIFY: str
    DELETE: str
    INVALID: str

    @classmethod
    def from_str(cls, value: str) -> SecurityUpdateAction: ...
    @classmethod
    def variants(cls) -> Iterable[SecurityUpdateAction]: ...

class StatType(Enum):
    """
    The type of statistic contained in a `StatMsg`.

    OPENING_PRICE
        The price of the first trade of an instrument. `price` will be set.
        `quantity` will be set when provided by the venue.
    INDICATIVE_OPENING_PRICE
        The probable price of the first trade of an instrument published during pre- open. Both
        `price` and `quantity` will be set.
    SETTLEMENT_PRICE
        The settlement price of an instrument. `price` will be set and `flags` indicate whether the
        price is final or preliminary and actual or theoretical. `ts_ref` will indicate the trading
        date of the settlement price.
    TRADING_SESSION_LOW_PRICE
        The lowest trade price of an instrument during the trading session. `price` will be set.
    TRADING_SESSION_HIGH_PRICE
        The highest trade price of an instrument during the trading session. `price` will be set.
    CLEARED_VOLUME
        The number of contracts cleared for an instrument on the previous trading date. `quantity`
        will be set. `ts_ref` will indicate the trading date of the volume.
    LOWEST_OFFER
        The lowest offer price for an instrument during the trading session. `price` will be set.
    HIGHEST_BID
        The highest bid price for an instrument during the trading session. `price` will be set.
    OPEN_INTEREST
        The current number of outstanding contracts of an instrument. `quantity` will be set.
        `ts_ref` will indicate the trading date for which the open interest was calculated.
    FIXING_PRICE
        The volume-weighted average price (VWAP) for a fixing period. `price` will be set.
    CLOSE_PRICE
        The last trade price during a trading session. `price` will be set.
        `quantity` will be set when provided by the venue.
    NET_CHANGE
        The change in price from the close price of the previous trading session to the most recent
        trading session. `price` will be set.
    VWAP
        The volume-weighted average price (VWAP) during the trading session. `price` will be set to
        the VWAP while `quantity` will be the traded volume.
    VOLATILITY
        The implied volatility associated with the settlement price.
    DELTA
        The option delta associated with the settlement price.
    UNCROSSING_PRICE
        The auction uncrossing price. This is used for auctions that are neither the
        official opening auction nor the official closing auction. `price` will be set.
        `quantity` will be set when provided by the venue.

    """

    OPENING_PRICE: int
    INDICATIVE_OPENING_PRICE: int
    SETTLEMENT_PRICE: int
    TRADING_SESSION_LOW_PRICE: int
    TRADING_SESSION_HIGH_PRICE: int
    CLEARED_VOLUME: int
    LOWEST_OFFER: int
    HIGHEST_BID: int
    OPEN_INTEREST: int
    FIXING_PRICE: int
    CLOSE_PRICE: int
    NET_CHANGE: int
    VWAP: int
    VOLATILITY: int
    DELTA: int
    UNCROSSING_PRICE: int

    @classmethod
    def variants(cls) -> Iterable[StatType]: ...

class StatUpdateAction(Enum):
    """
    The type of `StatMsg` update.

    NEW
        A new statistic.
    DELETE
        A removal of a statistic.

    """

    NEW: str
    DELETE: str

    @classmethod
    def from_str(cls, value: str) -> StatUpdateAction: ...
    @classmethod
    def variants(cls) -> Iterable[StatUpdateAction]: ...

class StatusAction(Enum):
    """
    The primary enum for the type of `StatusMsg` update.

    NONE
        No change.
    PRE_OPEN
        The instrument is in a pre-open period.
    PRE_CROSS
        The instrument is in a pre-cross period.
    QUOTING
        The instrument is quoting but not trading.
    CROSS
        The instrument is in a cross/auction.
    ROTATION
        The instrument is being opened through a trading rotation.
    NEW_PRICE_INDICATION
        A new price indication is available for the instrument.
    TRADING
        The instrument is trading.
    HALT
        Trading in the instrument has been halted.
    PAUSE
        Trading in the instrument has been paused.
    SUSPEND
        Trading in the instrument has been suspended.
    PRE_CLOSE
        The instrument is in a pre-close period.
    CLOSE
        Trading in the instrument has closed.
    POST_CLOSE
        The instrument is in a post-close period.
    SSR_CHANGE
        A change in short-selling restrictions.
    NOT_AVAILABLE_FOR_TRADING
        The instrument is not available for trading, either trading has closed or been halted.

    """

    NONE: int
    PRE_OPEN: int
    PRE_CROSS: int
    QUOTING: int
    CROSS: int
    ROTATION: int
    NEW_PRICE_INDICATION: int
    TRADING: int
    HALT: int
    PAUSE: int
    SUSPEND: int
    PRE_CLOSE: int
    CLOSE: int
    POST_CLOSE: int
    SSR_CHANGE: int
    NOT_AVAILABLE_FOR_TRADING: int

    @classmethod
    def variants(cls) -> Iterable[StatusAction]: ...

class StatusReason(Enum):
    """
    The secondary enum for a `StatusMsg` update, explains the cause of a halt or other change in
    `action`.

    NONE
        No reason is given.
    SCHEDULED
        The change in status occurred as scheduled.
    SURVEILLANCE_INTERVENTION
        The instrument stopped due to a market surveillance intervention.
    MARKET_EVENT
        The status changed due to activity in the market.
    INSTRUMENT_ACTIVATION
        The derivative instrument began trading.
    INSTRUMENT_EXPIRATION
        The derivative instrument expired.
    RECOVERY_IN_PROCESS
        Recovery in progress.
    REGULATORY
        The status change was caused by a regulatory action.
    ADMINISTRATIVE
        The status change was caused by an administrative action.
    NON_COMPLIANCE
        The status change was caused by the issuer not being compliance with regulatory
        requirements.
    FILINGS_NOT_CURRENT
        Trading halted because the issuer's filings are not current.
    SEC_TRADING_SUSPENSION
        Trading halted due to an SEC trading suspension.
    NEW_ISSUE
        The status changed because a new issue is available.
    ISSUE_AVAILABLE
        The status changed because an issue is available.
    ISSUES_REVIEWED
        The status changed because the issue(s) were reviewed.
    FILING_REQS_SATISFIED
        The status changed because the filing requirements were satisfied.
    NEWS_PENDING
        Relevant news is pending.
    NEWS_RELEASED
        Relevant news was released.
    NEWS_AND_RESUMPTION_TIMES
        The news has been fully disseminated and times are available for the resumption in quoting
        and trading.
    NEWS_NOT_FORTHCOMING
        The relevant news was not forthcoming.
    ORDER_IMBALANCE
        Halted for order imbalance.
    LULD_PAUSE
        The instrument hit limit up or limit down.
    OPERATIONAL
        An operational issue occurred with the venue.
    ADDITIONAL_INFORMATION_REQUESTED
        The status changed until the exchange receives additional information.
    MERGER_EFFECTIVE
        Trading halted due to merger becoming effective.
    ETF
        Trading is halted in an ETF due to conditions with the component securities.
    CORPORATE_ACTION
        Trading is halted for a corporate action.
    NEW_SECURITY_OFFERING
        Trading is halted because the instrument is a new offering.
    MARKET_WIDE_HALT_LEVEL1
        Halted due to the market-wide circuit breaker level 1.
    MARKET_WIDE_HALT_LEVEL2
        Halted due to the market-wide circuit breaker level 2.
    MARKET_WIDE_HALT_LEVEL3
        Halted due to the market-wide circuit breaker level 3.
    MARKET_WIDE_HALT_CARRYOVER
        Halted due to the carryover of a market-wide circuit breaker from the previous trading day.
    MARKET_WIDE_HALT_RESUMPTION
        Resumption due to the end of a market-wide circuit breaker halt.
    QUOTATION_NOT_AVAILABLE
        Halted because quotation is not available.

    """

    NONE: int
    SCHEDULED: int
    SURVEILLANCE_INTERVENTION: int
    MARKET_EVENT: int
    INSTRUMENT_ACTIVATION: int
    INSTRUMENT_EXPIRATION: int
    RECOVERY_IN_PROCESS: int
    REGULATORY: int
    ADMINISTRATIVE: int
    NON_COMPLIANCE: int
    FILINGS_NOT_CURRENT: int
    SEC_TRADING_SUSPENSION: int
    NEW_ISSUE: int
    ISSUE_AVAILABLE: int
    ISSUES_REVIEWED: int
    FILING_REQS_SATISFIED: int
    NEWS_PENDING: int
    NEWS_RELEASED: int
    NEWS_AND_RESUMPTION_TIMES: int
    NEWS_NOT_FORTHCOMING: int
    ORDER_IMBALANCE: int
    LULD_PAUSE: int
    OPERATIONAL: int
    ADDITIONAL_INFORMATION_REQUESTED: int
    MERGER_EFFECTIVE: int
    ETF: int
    CORPORATE_ACTION: int
    NEW_SECURITY_OFFERING: int
    MARKET_WIDE_HALT_CARRYOVER: int
    MARKET_WIDE_HALT_RESUMPTION: int
    QUOTATION_NOT_AVAILABLE: int

    @classmethod
    def variants(cls) -> Iterable[StatusReason]: ...

class TradingEvent(Enum):
    """
    Further information about a status update.


    NONE
        No additional information given.
    NO_CANCEL
        Order entry and modification are not allowed.
    CHANGE_TRADING_SESSION
        A change of trading session occurred. Daily statistics are reset.
    IMPLIED_MATCHING_ON
        Implied matching is available.
    IMPLIED_MATCHING_OFF
        Implied matching is not available.

    """

    NONE: int
    NO_CANCEL: int
    CHANGE_TRADING_SESSION: int
    IMPLIED_MATCHING_ON: int
    IMPLIED_MATCHING_OFF: int

    @classmethod
    def variants(cls) -> Iterable[TradingEvent]: ...

class TriState(Enum):
    """
    An enum for representing unknown, true, or false values. Equivalent to `Optional[bool]`.

    NOT_AVAILABLE
        The value is not applicable or not known.
    NO
        False
    YES
        True

    """

    NOT_AVAILABLE: str
    NO: str
    YES: str

    @classmethod
    def from_str(cls, value: str) -> TriState: ...
    @classmethod
    def variants(cls) -> Iterable[TriState]: ...
    def opt_bool(self) -> Optional[bool]: ...

class VersionUpgradePolicy(Enum):
    """
    How to handle decoding a DBN data from a prior version.

    AS_IS
        Decode data from previous versions as-is.
    UPGRADE_TO_V2
        Decode and convert data from DBN versions prior to version 2 to that version.
        Attempting to decode data from newer versions will fail.
    UPGRADE_TO_V3
        Decode and convert data from DBN versions prior to version 3 to that version.
        Attempting to decode data from newer versions (when they're introduced) will
        fail.

    """

    AS_IS: int
    UPGRADE_TO_V2: int
    UPGRADE_TO_V3: int

class ErrorCode(Enum):
    """
    An error code from the live subscription gateway.

    AUTH_FAILED
        The authentication step failed.
    API_KEY_DEACTIVATED
        The user account or API key were deactivated.
    CONNECTION_LIMIT_EXCEEDED
        The user has exceeded their open connection limit
    SYMBOL_RESOLUTION_FAILED
        One or more symbols failed to resolve.
    INVALID_SUBSCRIPTION
        There was an issue with a subscription request (other than symbol resolution).
     INTERNAL_ERROR
        An error occurred in the gateway.

    """

    AUTH_FAILED: int
    API_KEY_DEACTIVATED: int
    CONNECTION_LIMIT_EXCEEDED: int
    SYMBOL_RESOLUTION_FAILED: int
    INVALID_SUBSCRIPTION: int
    INTERNAL_ERROR: int

    @classmethod
    def variants(cls) -> Iterable[ErrorCode]: ...

class SystemCode(Enum):
    """
    A `SystemMsg` code indicating the type of message from the live subscription
    gateway.

    HEARTBEAT
        A message sent in the absence of other records to indicate the connection
        remains open.
    SUBSCRIPTION_ACK
        An acknowledgement of a subscription request.
    SLOW_READER_WARNING
        The gateway has detected this session is falling behind real-time.
    REPLAY_COMPLETED
        Indicates a replay subscription has caught up with real-time data.

    """

    HEARTBEAT: int
    SUBSCRIPTION_ACK: int
    SLOW_READER_WARNING: int
    REPLAY_COMPLETED: int

    @classmethod
    def variants(cls) -> Iterable[ErrorCode]: ...

class Metadata(SupportsBytes):
    """
    Information about the data contained in a DBN file or stream. DBN requires
    the Metadata to be included at the start of the encoded data.
    """

    def __init__(
        self,
        dataset: str,
        start: int,
        stype_in: SType | None,
        stype_out: SType,
        schema: Schema | None,
        symbols: list[str] | None = None,
        partial: list[str] | None = None,
        not_found: list[str] | None = None,
        mappings: Sequence[SymbolMapping] | None = None,
        end: int | None = None,
        limit: int | None = None,
        ts_out: bool | None = None,
        version: int | None = None,
    ) -> None: ...
    def __bytes__(self) -> bytes: ...
    def __eq__(self, other) -> bool: ...
    def __ne__(self, other) -> bool: ...
    @property
    def version(self) -> int:
        """
        The DBN schema version number.

        Returns
        -------
        int

        """

    @property
    def dataset(self) -> str:
        """
        The dataset code.

        Returns
        -------
        str

        """

    @property
    def schema(self) -> str | None:
        """
        The data record schema. Specifies which record type is stored in the
        Zstd-compressed DBN file.

        Returns
        -------
        str | None

        """

    @property
    def start(self) -> int:
        """
        The UNIX nanosecond timestamp of the query start, or the first record
        if the file was split.

        Returns
        -------
        int

        """

    @property
    def end(self) -> int:
        """
        The UNIX nanosecond timestamp of the query end, or the last record if
        the file was split.

        Returns
        -------
        int

        """

    @property
    def limit(self) -> int:
        """
        The optional maximum number of records for the query.

        Returns
        -------
        int

        """

    @property
    def stype_in(self) -> SType | None:
        """
        The input symbology type to map from.

        Returns
        -------
        SType | None

        """

    @property
    def stype_out(self) -> SType:
        """
        The output symbology type to map to.

        Returns
        -------
        SType

        """

    @property
    def ts_out(self) -> bool:
        """
        `true` if this store contains live data with send timestamps appended
        to each record.

        Returns
        -------
        bool

        """

    @property
    def symbols(self) -> list[str]:
        """
        The original query input symbols from the request.

        Returns
        -------
        list[str]

        """

    @property
    def partial(self) -> list[str]:
        """
        Symbols that did not resolve for at least one day in the query time
        range.

        Returns
        -------
        list[str]

        """

    @property
    def not_found(self) -> list[str]:
        """
        Symbols that did not resolve for any day in the query time range.

        Returns
        -------
        list[str]

        """

    @property
    def mappings(self) -> dict[str, list[MappingIntervalDict]]:
        """
        Symbol mappings containing a native symbol and its mapping intervals.

        Returns
        -------
        dict[str, list[dict[str, Any]]]:

        """

    @classmethod
    def decode(cls, data: bytes, upgrade_policy: VersionUpgradePolicy | None = None) -> Metadata:
        """
        Decode the given Python `bytes` to `Metadata`. Returns a `Metadata`
        object with all the DBN metadata attributes.

        Parameters
        ----------
        data : bytes
            The bytes to decode from.
        upgrade_policy : VersionUpgradePolicy, default UPGRADE
            How to decode data from prior DBN versions. Defaults to upgrade decoding.

        Returns
        -------
        Metadata

        Raises
        ------
        DBNError
            When a Metadata instance cannot be parsed from `data`.

        """

    def encode(self) -> bytes:
        """
        Encode the Metadata to bytes.

        Returns
        -------
        bytes

        Raises
        ------
        DBNError
            When the Metadata object cannot be encoded.

        """

class RecordHeader:
    """
    DBN Record Header.
    """

    @property
    def length(self) -> int:
        """
        The length of the record.

        Returns
        -------
        int

        """

    @property
    def rtype(self) -> int:
        """
        The record type.

        Returns
        -------
        int

        """

    @property
    def publisher_id(self) -> int:
        """
        The publisher ID assigned by Databento, which denotes the dataset and venue.

        See `Publishers` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        """

    @property
    def instrument_id(self) -> int:
        """
        The numeric instrument ID.

        See `Instrument identifiers` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#instrument-identifiers.

        Returns
        -------
        int

        """

    @property
    def ts_event(self) -> int:
        """
        The matching-engine-received timestamp expressed as the number of nanoseconds
        since the UNIX epoch.

        See `ts_event` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-event.

        Returns
        -------
        int

        """

class Record(SupportsBytes):
    """
    Base class for DBN records.
    """

    size_hint: ClassVar[int]
    _dtypes: ClassVar[list[tuple[str, str]]]
    _hidden_fields: ClassVar[list[str]]
    _price_fields: ClassVar[list[str]]
    _ordered_fields: ClassVar[list[str]]
    _timestamp_fields: ClassVar[list[str]]

    def __bytes__(self) -> bytes: ...
    def __eq__(self, other) -> bool: ...
    def __ne__(self, other) -> bool: ...
    @property
    def hd(self) -> RecordHeader:
        """
        The common header.

        Returns
        -------
        RecordHeader

        """

    @property
    def record_size(self) -> int:
        """
        Return the size of the record in bytes.

        Returns
        -------
        int

        See Also
        --------
        size_hint

        """

    @property
    def rtype(self) -> int:
        """
        The record type.

        Returns
        -------
        int

        """

    @property
    def publisher_id(self) -> int:
        """
        The publisher ID assigned by Databento, which denotes the dataset and venue.

        See `Publishers` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        """

    @property
    def instrument_id(self) -> int:
        """
        The numeric instrument ID.

        See `Instrument identifiers` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#instrument-identifiers.

        Returns
        -------
        int

        """

    @property
    def pretty_ts_event(self) -> dt.datetime | None:
        """
        The matching-engine-received timestamp expressed as a
        datetime or a `pandas.Timestamp`, if available.

        Returns
        -------
        datetime.datetime

        """

    @property
    def ts_event(self) -> int:
        """
        The matching-engine-received timestamp expressed as the number of nanoseconds
        since the UNIX epoch.

        See `ts_event` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-event.

        Returns
        -------
        int

        """

    @property
    def ts_out(self) -> int | None:
        """
        The live gateway send timestamp expressed as the number of nanoseconds since the
        UNIX epoch.

        See `ts_out` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-out.

        Returns
        -------
        int | None

        """

class _MBOBase:
    """
    Base for market-by-order messages.
    """

    @property
    def order_id(self) -> int:
        """
        The order ID assigned at the venue.

        Returns
        -------
        int

        """

    @property
    def pretty_price(self) -> float:
        """
        The order price as a float.

        Returns
        -------
        float

        See Also
        --------
        price

        """

    @property
    def price(self) -> int:
        """
        The order price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
        0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_price

        """

    @property
    def size(self) -> int:
        """
        The order quantity.

        Returns
        -------
        int

        """

    @property
    def flags(self) -> int:
        """
        A bit field indicating event end, message characteristics, and data quality.

        Returns
        -------
        int

        """

    @property
    def channel_id(self) -> int:
        """
        The channel ID assigned by Databento as an incrementing integer starting at
        zero.

        Returns
        -------
        int

        """

    @property
    def action(self) -> str:
        """
        The event action. Can be **A**dd, **C**ancel, **M**odify, clea**R** book, **T**rade,
        **F**ill, or **N**one.

        See `Action` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#action.

        Returns
        -------
        str

        """

    @property
    def side(self) -> str:
        """
        The side that initiates the event. Can be **A**sk for a sell order
        (or sell aggressor in a trade), **B**id for a buy order (or buy aggressor in a trade),
        or **N**one where no side is specified.

        See `Side` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#side.

        Returns
        -------
        str

        """

    @property
    def pretty_ts_recv(self) -> dt.datetime | None:
        """
        The capture-server-received timestamp as a datetime or
        `pandas.Timestamp`, if available.

        Returns
        -------
        datetime.datetime

        """

    @property
    def ts_recv(self) -> int:
        """
        The capture-server-received timestamp expressed as the number of nanoseconds
        since the UNIX epoch.

        See `ts_recv` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv.

        Returns
        -------
        int

        """

    @property
    def ts_in_delta(self) -> int:
        """
        The matching-engine-sending timestamp expressed as the number of nanoseconds before
        `ts_recv`.

        See `ts_in_delta` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-in-delta.

        Returns
        -------
        int

        """

    @property
    def sequence(self) -> int:
        """
        The message sequence number assigned at the venue.

        Returns
        -------
        int

        """

class MBOMsg(Record, _MBOBase):
    """
    A market-by-order (MBO) tick message.
    """

    def __init__(
        self,
        publisher_id: int,
        instrument_id: int,
        ts_event: int,
        order_id: int,
        price: int,
        size: int,
        action: str,
        side: str,
        ts_recv: int,
        flags: int | None = None,
        channel_id: int | None = None,
        ts_in_delta: int | None = None,
        sequence: int | None = None,
    ) -> None: ...

class BidAskPair:
    """
    A book level.
    """

    def __init__(
        self,
        bid_px: int = UNDEF_PRICE,
        ask_px: int = UNDEF_PRICE,
        bid_sz: int = 0,
        ask_sz: int = 0,
        bid_ct: int = 0,
        ask_ct: int = 0,
    ) -> None: ...
    @property
    def pretty_bid_px(self) -> float:
        """
        The bid price as a float.

        Returns
        -------
        float

        See Also
        --------
        bid_px

        """

    @property
    def bid_px(self) -> int:
        """
        The bid price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
        0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_bid_px

        """

    @property
    def pretty_ask_px(self) -> float:
        """
        The ask price as a float.

        Returns
        -------
        float

        See Also
        --------
        ask_px

        """

    @property
    def ask_px(self) -> int:
        """
        The ask price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
        0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_ask_px

        """

    @property
    def bid_sz(self) -> int:
        """
        The bid size.

        Returns
        -------
        int

        """

    @property
    def ask_sz(self) -> int:
        """
        The ask size.

        Returns
        -------
        int

        """

    @property
    def bid_ct(self) -> int:
        """
        The bid order count.

        Returns
        -------
        int

        """

    @property
    def ask_ct(self) -> int:
        """
        The ask order count.

        Returns
        -------
        int

        """

class ConsolidatedBidAskPair:
    """
    A consolidated book level.
    """

    def __init__(
        self,
        bid_px: int = UNDEF_PRICE,
        ask_px: int = UNDEF_PRICE,
        bid_sz: int = 0,
        ask_sz: int = 0,
        bid_pb: int = 0,
        ask_pb: int = 0,
    ) -> None: ...
    @property
    def pretty_bid_px(self) -> float:
        """
        The bid price as a float.

        Returns
        -------
        float

        See Also
        --------
        bid_px

        """

    @property
    def bid_px(self) -> int:
        """
        The bid price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
        0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_bid_px

        """

    @property
    def pretty_ask_px(self) -> float:
        """
        The ask price as a float.

        Returns
        -------
        float

        See Also
        --------
        ask_px

        """

    @property
    def ask_px(self) -> int:
        """
        The ask price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
        0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_ask_px

        """

    @property
    def bid_sz(self) -> int:
        """
        The bid size.

        Returns
        -------
        int

        """

    @property
    def ask_sz(self) -> int:
        """
        The ask size.

        Returns
        -------
        int

        """

    @property
    def bid_pb(self) -> int:
        """
        The publisher ID indicating the venue containing the best bid.

        See `Publishers` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        """

    @property
    def pretty_bid_pb(self) -> Optional[str]:
        """
        The human-readable bid publisher.

        Returns
        -------
        Optional[str]

        """

    @property
    def ask_pb(self) -> int:
        """
        The publisher ID indicating the venue containing the best ask.

        See `Publishers` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        """

    @property
    def pretty_ask_pb(self) -> Optional[str]:
        """
        The human-readable ask publisher.

        Returns
        -------
        Optional[str]

        """

class _MBPBase:
    """
    Base for market-by-price messages.
    """

    @property
    def pretty_price(self) -> float:
        """
        The order price as a float.

        Returns
        -------
        float

        See Also
        --------
        price

        """

    @property
    def price(self) -> int:
        """
        The order price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
        0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_price

        """

    @property
    def size(self) -> int:
        """
        The order quantity.

        Returns
        -------
        int

        """

    @property
    def action(self) -> str:
        """
        The event action. Can be **A**dd, **C**ancel, **M**odify, clea**R** book or **T**rade.

        See `Action` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#action.

        Returns
        -------
        str

        """

    @property
    def side(self) -> str:
        """
        The side that initiates the event. Can be **A**sk for a sell order
        (or sell aggressor in a trade), **B**id for a buy order (or buy aggressor in a trade),
        or **N**one where no side is specified.

        See `Side` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#side.

        Returns
        -------
        str

        """

    @property
    def flags(self) -> int:
        """
        A bit field indicating event end, message characteristics, and data quality.

        Returns
        -------
        int

        """

    @property
    def depth(self) -> int:
        """
        The depth of actual book change.

        Returns
        -------
        int

        """

    @property
    def pretty_ts_recv(self) -> dt.datetime | None:
        """
        The capture-server-received timestamp as a datetime or
        `pandas.Timestamp`, if available.

        Returns
        -------
        datetime.datetime

        """

    @property
    def ts_recv(self) -> int:
        """
        The capture-server-received timestamp expressed as the number of nanoseconds
        since the UNIX epoch.

        See `ts_recv` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv.

        Returns
        -------
        int

        """

    @property
    def ts_in_delta(self) -> int:
        """
        The matching-engine-sending timestamp expressed as the number of nanoseconds before
        `ts_recv`.

        See `ts_in_delta` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-in-delta.

        Returns
        -------
        int

        """

    @property
    def sequence(self) -> int:
        """
        The message sequence number assigned at the venue.

        Returns
        -------
        int

        """

class TradeMsg(Record, _MBPBase):
    """
    Market by price implementation with a book depth of 0.

    Equivalent to MBP-0. The record of the `Trades` schema.

    """

    def __init__(
        self,
        publisher_id: int,
        instrument_id: int,
        ts_event: int,
        price: int,
        size: int,
        action: str,
        side: str,
        depth: int,
        ts_recv: int,
        flags: int | None = None,
        ts_in_delta: int | None = None,
        sequence: int | None = None,
    ) -> None: ...

class MBP1Msg(Record, _MBPBase):
    """
    Market by price implementation with a known book depth of 1.
    """

    def __init__(
        self,
        publisher_id: int,
        instrument_id: int,
        ts_event: int,
        price: int,
        size: int,
        action: str,
        side: str,
        depth: int,
        ts_recv: int,
        flags: int | None = None,
        ts_in_delta: int | None = None,
        sequence: int | None = None,
        levels: BidAskPair | None = None,
    ) -> None: ...
    @property
    def levels(self) -> list[BidAskPair]:
        """
        The top of the order book.

        Returns
        -------
        list[BidAskPair]

        Notes
        -----
        MBP1Msg contains 1 level of BidAskPair.

        """

class BBOMsg(Record):
    """
    Subsampled market by price with a known book depth of 1.
    """

    def __init__(
        self,
        rtype: int,
        publisher_id: int,
        instrument_id: int,
        ts_event: int,
        price: int,
        size: int,
        action: str,
        side: str,
        ts_recv: int,
        flags: int | None = None,
        sequence: int | None = None,
        levels: BidAskPair | None = None,
    ) -> None: ...
    @property
    def pretty_price(self) -> float:
        """
        The price of the last trade as a float.

        Returns
        -------
        float

        See Also
        --------
        price

        """

    @property
    def price(self) -> int:
        """
        The order price of the last trade where every 1 unit corresponds to 1e-9, i.e.
        1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_price

        """

    @property
    def size(self) -> int:
        """
        The quantity of the last trade.

        Returns
        -------
        int

        """

    @property
    def side(self) -> str:
        """
        The side that initiated the last trade. Can be **A**sk for a sell aggressor in a trade,
        **B**id for a buy aggressor in a trade, or **N**one where no side is specified.

        See `Side` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#side.

        Returns
        -------
        str

        """

    @property
    def flags(self) -> int:
        """
        A bit field indicating event end, message characteristics, and data quality.

        Returns
        -------
        int

        """

    @property
    def pretty_ts_recv(self) -> dt.datetime | None:
        """
        The end timestamp of the interval, clamped to the second/minute boundary, expressed as
        a datetime or`pandas.Timestamp`, if available.

        Returns
        -------
        datetime.datetime

        """

    @property
    def ts_recv(self) -> int:
        """
        The end timestamp of the interval, clamped to the second/minute boundary,
        expressed as the number of nanoseconds since the UNIX epoch.

        Returns
        -------
        int

        """

    @property
    def sequence(self) -> int:
        """
        The message sequence number assigned at the venue of the last update.

        Returns
        -------
        int

        """

    @property
    def levels(self) -> list[BidAskPair]:
        """
        The top of the order book.

        Returns
        -------
        list[BidAskPair]

        Notes
        -----
        BBOMsg contains 1 level of BidAskPair.

        """

class CMBP1Msg(Record):
    """
    Consolidated best bid and offer implementation.
    """

    def __init__(
        self,
        rtype: int,
        publisher_id: int,
        instrument_id: int,
        ts_event: int,
        price: int,
        size: int,
        action: str,
        side: str,
        ts_recv: int,
        flags: int | None = None,
        ts_in_delta: int | None = None,
        levels: ConsolidatedBidAskPair | None = None,
    ) -> None: ...
    @property
    def pretty_price(self) -> float:
        """
        The order price as a float.

        Returns
        -------
        float

        See Also
        --------
        price

        """

    @property
    def price(self) -> int:
        """
        The order price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
        0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_price

        """

    @property
    def size(self) -> int:
        """
        The order quantity.

        Returns
        -------
        int

        """

    @property
    def action(self) -> str:
        """
        The event action. Can be **A**dd, **C**ancel, **M**odify, clea**R** book, or **T**rade.

        See `Action` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#action.

        Returns
        -------
        str

        """

    @property
    def side(self) -> str:
        """
        The side that initiates the event. Can be **A**sk for a sell order
        (or sell aggressor in a trade), **B**id for a buy order (or buy aggressor in a trade),
        or **N**one where no side is specified.

        See `Side` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#side.

        Returns
        -------
        str

        """

    @property
    def flags(self) -> int:
        """
        A bit field indicating event end, message characteristics, and data quality.

        Returns
        -------
        int

        """

    @property
    def pretty_ts_recv(self) -> dt.datetime | None:
        """
        The capture-server-received timestamp expressed as a datetime or `pandas.Timestamp`
        if available.

        Returns
        -------
        datetime.datetime

        """

    @property
    def ts_recv(self) -> int:
        """
        The capture-server-received timestamp expressed as the number of nanoseconds
        since the UNIX epoch.

        See `ts_recv` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv.

        Returns
        -------
        int

        """

    @property
    def ts_in_delta(self) -> int:
        """
        The matching-engine-sending timestamp expressed as the number of nanoseconds before
        `ts_recv`.

        See `ts_in_delta` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-in-delta.

        Returns
        -------
        int

        """

    @property
    def levels(self) -> list[ConsolidatedBidAskPair]:
        """
        The top of the consolidated order book.

        Returns
        -------
        list[ConsolidatedBidAskPair]

        Notes
        -----
        CMBP1Msg contains 1 level of ConsolidatedBidAskPair.

        """

class CBBOMsg(Record):
    """
    Subsampled consolidated best bid and offer.
    """

    def __init__(
        self,
        rtype: int,
        publisher_id: int,
        instrument_id: int,
        ts_event: int,
        price: int,
        size: int,
        side: str,
        ts_recv: int,
        flags: int | None = None,
        levels: ConsolidatedBidAskPair | None = None,
    ) -> None: ...
    @property
    def pretty_price(self) -> float:
        """
        The order price of the last trade as a float.

        Returns
        -------
        float

        See Also
        --------
        price

        """

    @property
    def price(self) -> int:
        """
        The order price of the last trade where every 1 unit corresponds to 1e-9, i.e.
        1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_price

        """

    @property
    def size(self) -> int:
        """
        The quantity of the last trade.

        Returns
        -------
        int

        """

    @property
    def side(self) -> str:
        """
        The side that initiated the last trade. Can be **A**sk for a sell order (or sell
        aggressor in a trade), **B**id for a buy order (or buy aggressor in a trade), or
        **N**one where no side is specified by the original source.

        Returns
        -------
        str

        """

    @property
    def flags(self) -> int:
        """
        A bit field indicating event end, message characteristics, and data quality.

        Returns
        -------
        int

        """

    @property
    def pretty_ts_recv(self) -> dt.datetime | None:
        """
        The end timestamp of the interval, clamped to the second/minute boundary,
        expressed as a datetime or `pandas.Timestamp`, if available.

        Returns
        -------
        datetime.datetime

        """

    @property
    def ts_recv(self) -> int:
        """
        The end timestamp of the interval, clamped to the second/minute boundary,
        expressed as the number of nanoseconds since the UNIX epoch.

        Returns
        -------
        int

        """

    @property
    def levels(self) -> list[ConsolidatedBidAskPair]:
        """
        The top of the order book.

        Returns
        -------
        list[BidAskPair]

        Notes
        -----
        BBOMsg contains 1 level of ConsolidatedBidAskPair.

        """

class MBP10Msg(Record, _MBPBase):
    """
    Market by price implementation with a known book depth of 10.
    """

    def __init__(
        self,
        publisher_id: int,
        instrument_id: int,
        ts_event: int,
        price: int,
        size: int,
        action: str,
        side: str,
        depth: int,
        ts_recv: int,
        flags: int | None = None,
        ts_in_delta: int | None = None,
        sequence: int | None = None,
        levels: list[BidAskPair] | None = None,
    ) -> None: ...
    @property
    def levels(self) -> list[BidAskPair]:
        """
        The top 10 levels.

        Returns
        -------
        list[BidAskPair]

        Notes
        -----
        MBP10Msg contains 10 levels of BidAskPairs.

        """

class OHLCVMsg(Record):
    """
    Open, high, low, close, and volume message.
    """

    def __init__(
        self,
        rtype: int,
        publisher_id: int,
        instrument_id: int,
        ts_event: int,
        open: int,
        high: int,
        low: int,
        close: int,
        volume: int,
    ) -> None: ...
    @property
    def pretty_open(self) -> float:
        """
        The open price for the bar as a float.

        Returns
        -------
        float

        See Also
        --------
        open

        """

    @property
    def open(self) -> int:
        """
        The open price for the bar where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000
        or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_open

        """

    @property
    def pretty_high(self) -> float:
        """
        The high price for the bar as a float.

        Returns
        -------
        float

        See Also
        --------
        high

        """

    @property
    def high(self) -> int:
        """
        The high price for the bar where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000
        or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_high

        """

    @property
    def pretty_low(self) -> float:
        """
        The low price for the bar as a float.

        Returns
        -------
        float

        See Also
        --------
        low

        """

    @property
    def low(self) -> int:
        """
        The low price for the bar where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000
        or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_low

        """

    @property
    def pretty_close(self) -> float:
        """
        The close price for the bar as a float.

        Returns
        -------
        float

        See Also
        --------
        close

        """

    @property
    def close(self) -> int:
        """
        The close price for the bar where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000
        or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_close

        """

    @property
    def volume(self) -> int:
        """
        The total volume traded during the aggregation period.

        Returns
        -------
        int

        """

class InstrumentDefMsg(Record):
    """
    Definition of an instrument in DBN version 3.
    """

    def __init__(
        self,
        publisher_id: int,
        instrument_id: int,
        ts_event: int,
        ts_recv: int,
        min_price_increment: int,
        display_factor: int,
        min_lot_size_round_lot: int,
        raw_symbol: str,
        group: str,
        exchange: str,
        instrument_class: str,
        match_algorithm: str,
        security_update_action: str,
        expiration: int = UNDEF_TIMESTAMP,
        activation: int = UNDEF_TIMESTAMP,
        high_limit_price: int = UNDEF_PRICE,
        low_limit_price: int = UNDEF_PRICE,
        max_price_variation: int = UNDEF_PRICE,
        unit_of_measure_qty: int = UNDEF_PRICE,
        min_price_increment_amount: int = UNDEF_PRICE,
        price_ratio: int = UNDEF_PRICE,
        inst_attrib_value: int | None = None,
        underlying_id: int | None = None,
        raw_instrument_id: int | None = None,
        market_depth_implied: int | None = None,
        market_depth: int | None = None,
        market_segment_id: int | None = None,
        max_trade_vol: int | None = None,
        min_lot_size: int | None = None,
        min_lot_size_block: int | None = None,
        min_trade_vol: int | None = None,
        contract_multiplier: int | None = None,
        decay_quantity: int | None = None,
        original_contract_size: int | None = None,
        appl_id: int | None = None,
        maturity_year: int | None = None,
        decay_start_date: int | None = None,
        channel_id: int | None = None,
        currency: str = "",
        settl_currency: str = "",
        secsubtype: str = "",
        asset: str = "",
        cfi: str = "",
        security_type: str = "",
        unit_of_measure: str = "",
        underlying: str = "",
        strike_price_currency: str = "",
        strike_price: int = UNDEF_PRICE,
        main_fraction: int | None = None,
        price_display_format: int | None = None,
        sub_fraction: int | None = None,
        underlying_product: int | None = None,
        maturity_month: int | None = None,
        maturity_day: int | None = None,
        maturity_week: int | None = None,
        user_defined_instrument: str | None = None,
        contract_multiplier_unit: int | None = None,
        flow_schedule_type: int | None = None,
        tick_rule: int | None = None,
        leg_count: int = 0,
        leg_index: int = 0,
        leg_price: int = UNDEF_PRICE,
        leg_delta: int = UNDEF_PRICE,
        leg_instrument_id: int = 0,
        leg_ratio_price_numerator: int = 0,
        leg_ratio_price_denominator: int = 0,
        leg_ratio_qty_numerator: int = 0,
        leg_ratio_qty_denominator: int = 0,
        leg_underlying_id: int = 0,
        leg_raw_symbol: str = "",
        leg_instrument_class: str | None = None,
        leg_side: str | None = None,
    ) -> None: ...
    @property
    def pretty_ts_recv(self) -> dt.datetime | None:
        """
        The capture-server-received timestamp as a datetime or
        `pandas.Timestamp`, if available.

        Returns
        -------
        datetime.datetime

        """

    @property
    def ts_recv(self) -> int:
        """
        The capture-server-received timestamp expressed as the number of nanoseconds
        since the UNIX epoch.

        See `ts_recv` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv.

        Returns
        -------
        int

        """

    @property
    def pretty_min_price_increment(self) -> float:
        """
        The minimum constant tick for the instrument as a float.

        Returns
        -------
        float

        See Also
        --------
        min_price_increment

        """

    @property
    def min_price_increment(self) -> int:
        """
        The minimum constant tick for the instrument where every 1 unit corresponds to 1e-9, i.e.
        1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_min_price_increment

        """

    @property
    def display_factor(self) -> int:
        """
        The multiplier to convert the venue’s display price to the conventional price where every
        1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        """

    @property
    def pretty_expiration(self) -> dt.datetime | None:
        """
        The last eligible trade time expressed as a datetime or
        `pandas.Timestamp`, if available.

        Some publishers only provide date-level granularity.

        Returns
        -------
        datetime.datetime

        """

    @property
    def expiration(self) -> int:
        """
        The last eligible trade time expressed as a number of nanoseconds since
        the UNIX epoch.

        Will be `UNDEF_TIMESTAMP` when null, such as for equities. Some publishers
        only provide date-level granularity.

        Returns
        -------
        int

        """

    @property
    def pretty_activation(self) -> dt.datetime | None:
        """
        The time of instrument activation expressed as a datetime or
        `pandas.Timestamp`, if available.

        Some publishers only provide date-level granularity.

        Returns
        -------
        datetime.datetime

        """

    @property
    def activation(self) -> int:
        """
        The time of instrument activation expressed as a number of nanoseconds
        since the UNIX epoch.

        Will be `UNDEF_TIMESTAMP` when null, such as for equities. Some publishers
        only provide date-level granularity.

        Returns
        -------
        int

        """

    @property
    def pretty_high_limit_price(self) -> float:
        """
        The allowable high limit price for the trading day as a float.

        Returns
        -------
        float

        See Also
        --------
        high_limit_price

        """

    @property
    def high_limit_price(self) -> int:
        """
        The allowable high limit price for the trading day where every 1 unit corresponds to
        1e-9, i.e. 1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_high_limit_price

        """

    @property
    def pretty_low_limit_price(self) -> float:
        """
        The allowable low limit price for the trading day as a float.

        Returns
        -------
        float

        See Also
        --------
        low_limit_price

        """

    @property
    def low_limit_price(self) -> int:
        """
        The allowable low limit price for the trading day where every 1 unit corresponds to
        1e-9, i.e. 1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_low_limit_price

        """

    @property
    def pretty_max_price_variation(self) -> float:
        """
        The differential value for price banding in units as a float.

        Returns
        -------
        float

        See Also
        --------
        max_price_variation

        """

    @property
    def max_price_variation(self) -> int:
        """
        The differential value for price banding where every 1 unit corresponds to 1e-9,
        i.e. 1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_max_price_variation

        """

    @property
    def unit_of_measure_qty(self) -> int:
        """
        The contract size for each instrument, in combination with `unit_of_measure`, where every
        1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        """

    @property
    def pretty_min_price_increment_amount(self) -> float:
        """
        The value currently under development by the venue as a float.

        Returns
        -------
        float

        See Also
        --------
        min_price_increment_amount

        """

    @property
    def min_price_increment_amount(self) -> int:
        """
        The value currently under development by the venue where every 1 unit corresponds to 1e-9,
        i.e. 1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_min_price_increment_amount

        """

    @property
    def pretty_price_ratio(self) -> float:
        """
        The value used for price calculation in spread and leg pricing as a
        float.

        Returns
        -------
        float

        See Also
        --------
        price_ratio

        """

    @property
    def price_ratio(self) -> int:
        """
        The value used for price calculation in spread and leg pricing where every 1 unit
        corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_price_ratio

        """

    @property
    def inst_attrib_value(self) -> int:
        """
        A bitmap of instrument eligibility attributes.

        Returns
        -------
        int

        """

    @property
    def underlying_id(self) -> int:
        """
        The `instrument_id` of the first underlying instrument.

        See `Instrument identifiers` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#instrument-identifiers.

        Returns
        -------
        int

        """

    @property
    def raw_instrument_id(self) -> int:
        """
        The instrument ID assigned by the publisher. May be the same as `instrument_id`.

        See `Instrument identifiers` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#instrument-identifiers.

        Returns
        -------
        int

        """

    @property
    def market_depth_implied(self) -> int:
        """
        The implied book depth on the price level data feed.

        Returns
        -------
        int

        """

    @property
    def market_depth(self) -> int:
        """
        The (outright) book depth on the price level data feed.

        Returns
        -------
        int

        """

    @property
    def market_segment_id(self) -> int:
        """
        The market segment of the instrument.

        Returns
        -------
        int

        """

    @property
    def max_trade_vol(self) -> int:
        """
        The maximum trading volume for the instrument.

        Returns
        -------
        int

        """

    @property
    def min_lot_size(self) -> int:
        """
        The minimum order entry quantity for the instrument.

        Returns
        -------
        int

        """

    @property
    def min_lot_size_block(self) -> int:
        """
        The minimum quantity required for a block trade of the instrument.

        Returns
        -------
        int

        """

    @property
    def min_lot_size_round_lot(self) -> int:
        """
        The minimum quantity required for a round lot of the instrument.
        Multiples of this quantity are also round lots.

        Returns
        -------
        int

        """

    @property
    def min_trade_vol(self) -> int:
        """
        The minimum trading volume for the instrument.

        Returns
        -------
        int

        """

    @property
    def contract_multiplier(self) -> int:
        """
        The number of deliverables per instrument, i.e. peak days.

        Returns
        -------
        int

        """

    @property
    def decay_quantity(self) -> int:
        """
        The quantity that a contract will decay daily, after `decay_start_date`
        has been reached.

        Returns
        -------
        int

        """

    @property
    def original_contract_size(self) -> int:
        """
        The fixed contract value assigned to each instrument.

        Returns
        -------
        int

        """

    @property
    def appl_id(self) -> int:
        """
        The channel ID assigned at the venue.

        Returns
        -------
        int

        """

    @property
    def maturity_year(self) -> int:
        """
        The calendar year reflected in the instrument symbol.

        Returns
        -------
        int

        """

    @property
    def decay_start_date(self) -> int:
        """
        The date at which a contract will begin to decay.

        Returns
        -------
        int

        """

    @property
    def channel_id(self) -> int:
        """
        The channel ID assigned by Databento as an incrementing integer
        starting at zero.

        Returns
        -------
        int

        """

    @property
    def currency(self) -> str:
        """
        The currency used for price fields.

        Returns
        -------
        str

        """

    @property
    def settl_currency(self) -> str:
        """
        The currency used for settlement, if different from `currency`.

        Returns
        -------
        str

        """

    @property
    def secsubtype(self) -> str:
        """
        The strategy type of the spread.

        Returns
        -------
        str

        """

    @property
    def raw_symbol(self) -> str:
        """
        The instrument raw symbol assigned by the publisher.

        Returns
        -------
        str

        """

    @property
    def group(self) -> str:
        """
        The security group code of the instrument.

        Returns
        -------
        str

        """

    @property
    def exchange(self) -> str:
        """
        The exchange used to identify the instrument.

        Returns
        -------
        str

        """

    @property
    def asset(self) -> str:
        """
        The underlying asset code (product code) of the instrument.

        Returns
        -------
        str

        """

    @property
    def cfi(self) -> str:
        """
        The ISO standard instrument categorization code.

        Returns
        -------
        str

        """

    @property
    def security_type(self) -> str:
        """
        The security type of the instrument, e.g. FUT for future or future spread.

        See `Security Type` https://databento.com/docs/schemas-and-data-formats/instrument-definitions#security-type.

        Returns
        -------
        str

        """

    @property
    def unit_of_measure(self) -> str:
        """
        The unit of measure for the instrument’s original contract size, e.g.
        USD or LBS.

        Returns
        -------
        str

        """

    @property
    def underlying(self) -> str:
        """
        The symbol of the first underlying instrument.

        Returns
        -------
        str

        """

    @property
    def strike_price_currency(self) -> str:
        """
        The currency of `strike_price`.

        Returns
        -------
        str

        """

    @property
    def instrument_class(self) -> str:
        """
        The classification of the instrument.

        See `Instrument class` https://databento.com/docs/schemas-and-data-formats/instrument-definitions#instrument-class

        Returns
        -------
        str

        """

    @property
    def pretty_strike_price(self) -> float:
        """
        The strike price of the option as a float.

        Returns
        -------
        float

        See Also
        --------
        strike_price

        """

    @property
    def strike_price(self) -> int:
        """
        The strike price of the option where every 1 unit corresponds to 1e-9, i.e.
        1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_strike_price

        """

    @property
    def match_algorithm(self) -> str:
        """
        The matching algorithm used for the instrument, typically **F**IFO.

        See `Matching algorithm` https://databento.com/docs/schemas-and-data-formats/instrument-definitions#matching-algorithm.

        Returns
        -------
        str

        """

    @property
    def main_fraction(self) -> int:
        """
        The price denominator of the main fraction.

        Returns
        -------
        int

        """

    @property
    def price_display_format(self) -> int:
        """
        The number of digits to the right of the tick mark, to display
        fractional prices.

        Returns
        -------
        int

        """

    @property
    def sub_fraction(self) -> int:
        """
        The price denominator of the sub fraction.

        Returns
        -------
        int

        """

    @property
    def underlying_product(self) -> int:
        """
        The product complex of the instrument.

        Returns
        -------
        int

        """

    @property
    def security_update_action(self) -> str:
        """
        Indicates if the instrument definition has been added, modified, or
        deleted.

        Returns
        -------
        str

        """

    @property
    def maturity_month(self) -> int:
        """
        The calendar month reflected in the instrument symbol.

        Returns
        -------
        int

        """

    @property
    def maturity_day(self) -> int:
        """
        The calendar day reflected in the instrument symbol, or 0.

        Returns
        -------
        int

        """

    @property
    def maturity_week(self) -> int:
        """
        The calendar week reflected in the instrument symbol, or 0.

        Returns
        -------
        int

        """

    @property
    def user_defined_instrument(self) -> str:
        """
        Indicates if the instrument is user defined: **Y**es or **N**o.

        Returns
        -------
        str

        """

    @property
    def contract_multiplier_unit(self) -> int:
        """
        The type of `contract_multiplier`. Either `1` for hours, or `2` for
        days.

        Returns
        -------
        int

        """

    @property
    def flow_schedule_type(self) -> int:
        """
        The schedule for delivering electricity.

        Returns
        -------
        int

        """

    @property
    def tick_rule(self) -> int:
        """
        The tick rule of the spread.

        Returns
        -------
        int

        """

    @property
    def pretty_leg_price(self) -> float:
        """
        The leg price as a float.

        Returns
        -------
        float

        See Also
        --------
        leg_price

        """

    @property
    def leg_price(self) -> int:
        """
        The leg price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
        0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_leg_price

        """

    @property
    def pretty_leg_delta(self) -> float:
        """
        The leg delta as a float.

        Returns
        -------
        float

        See Also
        --------
        leg_delta

        """

    @property
    def leg_delta(self) -> int:
        """
        The leg delta where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
        0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        leg_delta

        """

class InstrumentDefMsgV2(Record):
    """
    Definition of an instrument in DBN version 2.
    """

    def __init__(
        self,
        publisher_id: int,
        instrument_id: int,
        ts_event: int,
        ts_recv: int,
        min_price_increment: int,
        display_factor: int,
        min_lot_size_round_lot: int,
        raw_symbol: str,
        group: str,
        exchange: str,
        instrument_class: str,
        match_algorithm: str,
        md_security_trading_status: str,
        security_update_action: str,
        expiration: int = UNDEF_TIMESTAMP,
        activation: int = UNDEF_TIMESTAMP,
        high_limit_price: int = UNDEF_PRICE,
        low_limit_price: int = UNDEF_PRICE,
        max_price_variation: int = UNDEF_PRICE,
        trading_reference_price: int = UNDEF_PRICE,
        unit_of_measure_qty: int = UNDEF_PRICE,
        min_price_increment_amount: int = UNDEF_PRICE,
        price_ratio: int = UNDEF_PRICE,
        inst_attrib_value: int | None = None,
        underlying_id: int | None = None,
        raw_instrument_id: int | None = None,
        market_depth_implied: int | None = None,
        market_depth: int | None = None,
        market_segment_id: int | None = None,
        max_trade_vol: int | None = None,
        min_lot_size: int | None = None,
        min_lot_size_block: int | None = None,
        min_trade_vol: int | None = None,
        contract_multiplier: int | None = None,
        decay_quantity: int | None = None,
        original_contract_size: int | None = None,
        trading_reference_date: int | None = None,
        appl_id: int | None = None,
        maturity_year: int | None = None,
        decay_start_date: int | None = None,
        channel_id: int | None = None,
        currency: str = "",
        settl_currency: str = "",
        secsubtype: str = "",
        asset: str = "",
        cfi: str = "",
        security_type: str = "",
        unit_of_measure: str = "",
        underlying: str = "",
        strike_price_currency: str = "",
        strike_price: int = UNDEF_PRICE,
        main_fraction: int | None = None,
        price_display_format: int | None = None,
        settl_price_type: int | None = None,
        sub_fraction: int | None = None,
        underlying_product: int | None = None,
        maturity_month: int | None = None,
        maturity_day: int | None = None,
        maturity_week: int | None = None,
        user_defined_instrument: str | None = None,
        contract_multiplier_unit: int | None = None,
        flow_schedule_type: int | None = None,
        tick_rule: int | None = None,
    ) -> None: ...
    @property
    def pretty_ts_recv(self) -> dt.datetime | None:
        """
        The capture-server-received timestamp as a datetime or
        `pandas.Timestamp`, if available.

        Returns
        -------
        datetime.datetime

        """

    @property
    def ts_recv(self) -> int:
        """
        The capture-server-received timestamp expressed as the number of nanoseconds
        since the UNIX epoch.

        See `ts_recv` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv.

        Returns
        -------
        int

        """

    @property
    def pretty_min_price_increment(self) -> float:
        """
        The minimum constant tick for the instrument as a float.

        Returns
        -------
        float

        See Also
        --------
        min_price_increment

        """

    @property
    def min_price_increment(self) -> int:
        """
        The minimum constant tick for the instrument where every 1 unit corresponds to 1e-9, i.e.
        1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_min_price_increment

        """

    @property
    def display_factor(self) -> int:
        """
        The multiplier to convert the venue’s display price to the conventional price where every
        1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        """

    @property
    def pretty_expiration(self) -> dt.datetime | None:
        """
        The last eligible trade time expressed as a datetime or
        `pandas.Timestamp`, if available.

        Some publishers only provide date-level granularity.

        Returns
        -------
        datetime.datetime

        """

    @property
    def expiration(self) -> int:
        """
        The last eligible trade time expressed as a number of nanoseconds since
        the UNIX epoch.

        Will be `UNDEF_TIMESTAMP` when null, such as for equities. Some publishers
        only provide date-level granularity.

        Returns
        -------
        int

        """

    @property
    def pretty_activation(self) -> dt.datetime | None:
        """
        The time of instrument activation expressed as a datetime or
        `pandas.Timestamp`, if available.

        Some publishers only provide date-level granularity.

        Returns
        -------
        datetime.datetime

        """

    @property
    def activation(self) -> int:
        """
        The time of instrument activation expressed as a number of nanoseconds
        since the UNIX epoch.

        Will be `UNDEF_TIMESTAMP` when null, such as for equities. Some publishers
        only provide date-level granularity.

        Returns
        -------
        int

        """

    @property
    def pretty_high_limit_price(self) -> float:
        """
        The allowable high limit price for the trading day as a float.

        Returns
        -------
        float

        See Also
        --------
        high_limit_price

        """

    @property
    def high_limit_price(self) -> int:
        """
        The allowable high limit price for the trading day where every 1 unit corresponds to
        1e-9, i.e. 1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_high_limit_price

        """

    @property
    def pretty_low_limit_price(self) -> float:
        """
        The allowable low limit price for the trading day as a float.

        Returns
        -------
        float

        See Also
        --------
        low_limit_price

        """

    @property
    def low_limit_price(self) -> int:
        """
        The allowable low limit price for the trading day where every 1 unit corresponds to
        1e-9, i.e. 1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_low_limit_price

        """

    @property
    def pretty_max_price_variation(self) -> float:
        """
        The differential value for price banding in units as a float.

        Returns
        -------
        float

        See Also
        --------
        max_price_variation

        """

    @property
    def max_price_variation(self) -> int:
        """
        The differential value for price banding where every 1 unit corresponds to 1e-9,
        i.e. 1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_max_price_variation

        """

    @property
    def pretty_trading_reference_price(self) -> float:
        """
        The trading session settlement price on `trading_reference_date` as a float.

        Returns
        -------
        float

        See Also
        --------
        trading_reference_price

        """

    @property
    def trading_reference_price(self) -> int:
        """
        The trading session settlement price on `trading_reference_date` where every 1 unit
        corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_trading_reference_price

        """

    @property
    def unit_of_measure_qty(self) -> int:
        """
        The contract size for each instrument, in combination with `unit_of_measure`, where every
        1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        """

    @property
    def pretty_min_price_increment_amount(self) -> float:
        """
        The value currently under development by the venue as a float.

        Returns
        -------
        float

        See Also
        --------
        min_price_increment_amount

        """

    @property
    def min_price_increment_amount(self) -> int:
        """
        The value currently under development by the venue where every 1 unit corresponds to 1e-9,
        i.e. 1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_min_price_increment_amount

        """

    @property
    def pretty_price_ratio(self) -> float:
        """
        The value used for price calculation in spread and leg pricing as a
        float.

        Returns
        -------
        float

        See Also
        --------
        price_ratio

        """

    @property
    def price_ratio(self) -> int:
        """
        The value used for price calculation in spread and leg pricing where every 1 unit
        corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_price_ratio

        """

    @property
    def inst_attrib_value(self) -> int:
        """
        A bitmap of instrument eligibility attributes.

        Returns
        -------
        int

        """

    @property
    def underlying_id(self) -> int:
        """
        The `instrument_id` of the first underlying instrument.

        See `Instrument identifiers` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#instrument-identifiers.

        Returns
        -------
        int

        """

    @property
    def raw_instrument_id(self) -> int:
        """
        The instrument ID assigned by the publisher. May be the same as `instrument_id`.

        See `Instrument identifiers` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#instrument-identifiers.

        Returns
        -------
        int

        """

    @property
    def market_depth_implied(self) -> int:
        """
        The implied book depth on the price level data feed.

        Returns
        -------
        int

        """

    @property
    def market_depth(self) -> int:
        """
        The (outright) book depth on the price level data feed.

        Returns
        -------
        int

        """

    @property
    def market_segment_id(self) -> int:
        """
        The market segment of the instrument.

        Returns
        -------
        int

        """

    @property
    def max_trade_vol(self) -> int:
        """
        The maximum trading volume for the instrument.

        Returns
        -------
        int

        """

    @property
    def min_lot_size(self) -> int:
        """
        The minimum order entry quantity for the instrument.

        Returns
        -------
        int

        """

    @property
    def min_lot_size_block(self) -> int:
        """
        The minimum quantity required for a block trade of the instrument.

        Returns
        -------
        int

        """

    @property
    def min_lot_size_round_lot(self) -> int:
        """
        The minimum quantity required for a round lot of the instrument.
        Multiples of this quantity are also round lots.

        Returns
        -------
        int

        """

    @property
    def min_trade_vol(self) -> int:
        """
        The minimum trading volume for the instrument.

        Returns
        -------
        int

        """

    @property
    def contract_multiplier(self) -> int:
        """
        The number of deliverables per instrument, i.e. peak days.

        Returns
        -------
        int

        """

    @property
    def decay_quantity(self) -> int:
        """
        The quantity that a contract will decay daily, after `decay_start_date`
        has been reached.

        Returns
        -------
        int

        """

    @property
    def original_contract_size(self) -> int:
        """
        The fixed contract value assigned to each instrument.

        Returns
        -------
        int

        """

    @property
    def trading_reference_date(self) -> int:
        """
        The trading session date corresponding to the settlement price in
        `trading_reference_price`, in number of days since the UNIX epoch.

        Returns
        -------
        int

        """

    @property
    def appl_id(self) -> int:
        """
        The channel ID assigned at the venue.

        Returns
        -------
        int

        """

    @property
    def maturity_year(self) -> int:
        """
        The calendar year reflected in the instrument symbol.

        Returns
        -------
        int

        """

    @property
    def decay_start_date(self) -> int:
        """
        The date at which a contract will begin to decay.

        Returns
        -------
        int

        """

    @property
    def channel_id(self) -> int:
        """
        The channel ID assigned by Databento as an incrementing integer
        starting at zero.

        Returns
        -------
        int

        """

    @property
    def currency(self) -> str:
        """
        The currency used for price fields.

        Returns
        -------
        str

        """

    @property
    def settl_currency(self) -> str:
        """
        The currency used for settlement, if different from `currency`.

        Returns
        -------
        str

        """

    @property
    def secsubtype(self) -> str:
        """
        The strategy type of the spread.

        Returns
        -------
        str

        """

    @property
    def raw_symbol(self) -> str:
        """
        The instrument name (symbol).

        Returns
        -------
        str

        """

    @property
    def group(self) -> str:
        """
        The security group code of the instrument.

        Returns
        -------
        str

        """

    @property
    def exchange(self) -> str:
        """
        The exchange used to identify the instrument.

        Returns
        -------
        str

        """

    @property
    def asset(self) -> str:
        """
        The underlying asset code (product code) of the instrument.

        Returns
        -------
        str

        """

    @property
    def cfi(self) -> str:
        """
        The ISO standard instrument categorization code.

        Returns
        -------
        str

        """

    @property
    def security_type(self) -> str:
        """
        The security type of the instrument, e.g. FUT for future or future spread.

        See `Security Type` https://databento.com/docs/schemas-and-data-formats/instrument-definitions#security-type.

        Returns
        -------
        str

        """

    @property
    def unit_of_measure(self) -> str:
        """
        The unit of measure for the instrument’s original contract size, e.g.
        USD or LBS.

        Returns
        -------
        str

        """

    @property
    def underlying(self) -> str:
        """
        The symbol of the first underlying instrument.

        Returns
        -------
        str

        """

    @property
    def strike_price_currency(self) -> str:
        """
        The currency of `strike_price`.

        Returns
        -------
        str

        """

    @property
    def instrument_class(self) -> str:
        """
        The classification of the instrument.

        See `Instrument class` https://databento.com/docs/schemas-and-data-formats/instrument-definitions#instrument-class

        Returns
        -------
        str

        """

    @property
    def pretty_strike_price(self) -> float:
        """
        The strike price of the option as a float.

        Returns
        -------
        float

        See Also
        --------
        strike_price

        """

    @property
    def strike_price(self) -> int:
        """
        The strike price of the option where every 1 unit corresponds to 1e-9, i.e.
        1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_strike_price

        """

    @property
    def match_algorithm(self) -> str:
        """
        The matching algorithm used for the instrument, typically **F**IFO.

        See `Matching algorithm` https://databento.com/docs/schemas-and-data-formats/instrument-definitions#matching-algorithm.

        Returns
        -------
        str

        """

    @property
    def md_security_trading_status(self) -> int:
        """
        The current trading state of the instrument.

        Returns
        -------
        int

        """

    @property
    def main_fraction(self) -> int:
        """
        The price denominator of the main fraction.

        Returns
        -------
        int

        """

    @property
    def price_display_format(self) -> int:
        """
        The number of digits to the right of the tick mark, to display
        fractional prices.

        Returns
        -------
        int

        """

    @property
    def settl_price_type(self) -> int:
        """
        The type indicators for the settlement price, as a bitmap.

        Returns
        -------
        int

        """

    @property
    def sub_fraction(self) -> int:
        """
        The price denominator of the sub fraction.

        Returns
        -------
        int

        """

    @property
    def underlying_product(self) -> int:
        """
        The product complex of the instrument.

        Returns
        -------
        int

        """

    @property
    def security_update_action(self) -> str:
        """
        Indicates if the instrument definition has been added, modified, or
        deleted.

        Returns
        -------
        str

        """

    @property
    def maturity_month(self) -> int:
        """
        The calendar month reflected in the instrument symbol.

        Returns
        -------
        int

        """

    @property
    def maturity_day(self) -> int:
        """
        The calendar day reflected in the instrument symbol, or 0.

        Returns
        -------
        int

        """

    @property
    def maturity_week(self) -> int:
        """
        The calendar week reflected in the instrument symbol, or 0.

        Returns
        -------
        int

        """

    @property
    def user_defined_instrument(self) -> str:
        """
        Indicates if the instrument is user defined: **Y**es or **N**o.

        Returns
        -------
        str

        """

    @property
    def contract_multiplier_unit(self) -> int:
        """
        The type of `contract_multiplier`. Either `1` for hours, or `2` for
        days.

        Returns
        -------
        int

        """

    @property
    def flow_schedule_type(self) -> int:
        """
        The schedule for delivering electricity.

        Returns
        -------
        int

        """

    @property
    def tick_rule(self) -> int:
        """
        The tick rule of the spread.

        Returns
        -------
        int

        """

class InstrumentDefMsgV1(Record):
    """
    Definition of an instrument in DBN version 1.
    """

    def __init__(
        self,
        publisher_id: int,
        instrument_id: int,
        ts_event: int,
        ts_recv: int,
        min_price_increment: int,
        display_factor: int,
        min_lot_size_round_lot: int,
        raw_symbol: str,
        group: str,
        exchange: str,
        instrument_class: str,
        match_algorithm: str,
        md_security_trading_status: str,
        security_update_action: str,
        expiration: int = UNDEF_TIMESTAMP,
        activation: int = UNDEF_TIMESTAMP,
        high_limit_price: int = UNDEF_PRICE,
        low_limit_price: int = UNDEF_PRICE,
        max_price_variation: int = UNDEF_PRICE,
        trading_reference_price: int = UNDEF_PRICE,
        unit_of_measure_qty: int = UNDEF_PRICE,
        min_price_increment_amount: int = UNDEF_PRICE,
        price_ratio: int = UNDEF_PRICE,
        inst_attrib_value: int | None = None,
        underlying_id: int | None = None,
        raw_instrument_id: int | None = None,
        market_depth_implied: int | None = None,
        market_depth: int | None = None,
        market_segment_id: int | None = None,
        max_trade_vol: int | None = None,
        min_lot_size: int | None = None,
        min_lot_size_block: int | None = None,
        min_trade_vol: int | None = None,
        contract_multiplier: int | None = None,
        decay_quantity: int | None = None,
        original_contract_size: int | None = None,
        trading_reference_date: int | None = None,
        appl_id: int | None = None,
        maturity_year: int | None = None,
        decay_start_date: int | None = None,
        channel_id: int | None = None,
        currency: str = "",
        settl_currency: str = "",
        secsubtype: str = "",
        asset: str = "",
        cfi: str = "",
        security_type: str = "",
        unit_of_measure: str = "",
        underlying: str = "",
        strike_price_currency: str = "",
        strike_price: int = UNDEF_PRICE,
        main_fraction: int | None = None,
        price_display_format: int | None = None,
        settl_price_type: int | None = None,
        sub_fraction: int | None = None,
        underlying_product: int | None = None,
        maturity_month: int | None = None,
        maturity_day: int | None = None,
        maturity_week: int | None = None,
        user_defined_instrument: str | None = None,
        contract_multiplier_unit: int | None = None,
        flow_schedule_type: int | None = None,
        tick_rule: int | None = None,
    ) -> None: ...
    @property
    def pretty_ts_recv(self) -> dt.datetime | None:
        """
        The capture-server-received timestamp as a datetime or
        `pandas.Timestamp`, if available.

        Returns
        -------
        datetime.datetime

        """

    @property
    def ts_recv(self) -> int:
        """
        The capture-server-received timestamp expressed as the number of nanoseconds
        since the UNIX epoch.

        See `ts_recv` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv.

        Returns
        -------
        int

        """

    @property
    def pretty_min_price_increment(self) -> float:
        """
        The minimum constant tick for the instrument as a float.

        Returns
        -------
        float

        See Also
        --------
        min_price_increment

        """

    @property
    def min_price_increment(self) -> int:
        """
        The minimum constant tick for the instrument where every 1 unit corresponds to 1e-9, i.e.
        1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_min_price_increment

        """

    @property
    def display_factor(self) -> int:
        """
        The multiplier to convert the venue’s display price to the conventional price where every
        1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        """

    @property
    def pretty_expiration(self) -> dt.datetime | None:
        """
        The last eligible trade time expressed as a datetime or
        `pandas.Timestamp`, if available.

        Some publishers only provide date-level granularity.

        Returns
        -------
        datetime.datetime

        """

    @property
    def expiration(self) -> int:
        """
        The last eligible trade time expressed as a number of nanoseconds since
        the UNIX epoch.

        Will be `UNDEF_TIMESTAMP` when null, such as for equities. Some publishers
        only provide date-level granularity.

        Returns
        -------
        int

        """

    @property
    def pretty_activation(self) -> dt.datetime | None:
        """
        The time of instrument activation expressed as a datetime or
        `pandas.Timestamp`, if available.

        Some publishers only provide date-level granularity.

        Returns
        -------
        datetime.datetime

        """

    @property
    def activation(self) -> int:
        """
        The time of instrument activation expressed as a number of nanoseconds
        since the UNIX epoch.

        Will be `UNDEF_TIMESTAMP` when null, such as for equities. Some publishers
        only provide date-level granularity.

        Returns
        -------
        int

        """

    @property
    def pretty_high_limit_price(self) -> float:
        """
        The allowable high limit price for the trading day as a float.

        Returns
        -------
        float

        See Also
        --------
        high_limit_price

        """

    @property
    def high_limit_price(self) -> int:
        """
        The allowable high limit price for the trading day where every 1 unit corresponds to
        1e-9, i.e. 1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_high_limit_price

        """

    @property
    def pretty_low_limit_price(self) -> float:
        """
        The allowable low limit price for the trading day as a float.

        Returns
        -------
        float

        See Also
        --------
        low_limit_price

        """

    @property
    def low_limit_price(self) -> int:
        """
        The allowable low limit price for the trading day where every 1 unit corresponds to
        1e-9, i.e. 1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_low_limit_price

        """

    @property
    def pretty_max_price_variation(self) -> float:
        """
        The differential value for price banding in units as a float.

        Returns
        -------
        float

        See Also
        --------
        max_price_variation

        """

    @property
    def max_price_variation(self) -> int:
        """
        The differential value for price banding where every 1 unit corresponds to 1e-9,
        i.e. 1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_max_price_variation

        """

    @property
    def pretty_trading_reference_price(self) -> float:
        """
        The trading session settlement price on `trading_reference_date` as a float.

        Returns
        -------
        float

        See Also
        --------
        trading_reference_price

        """

    @property
    def trading_reference_price(self) -> int:
        """
        The trading session settlement price on `trading_reference_date` where every 1 unit
        corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_trading_reference_price

        """

    @property
    def unit_of_measure_qty(self) -> int:
        """
        The contract size for each instrument, in combination with `unit_of_measure`, where every
        1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        """

    @property
    def pretty_min_price_increment_amount(self) -> float:
        """
        The value currently under development by the venue as a float.

        Returns
        -------
        float

        See Also
        --------
        min_price_increment_amount

        """

    @property
    def min_price_increment_amount(self) -> int:
        """
        The value currently under development by the venue where every 1 unit corresponds to 1e-9,
        i.e. 1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_min_price_increment_amount

        """

    @property
    def pretty_price_ratio(self) -> float:
        """
        The value used for price calculation in spread and leg pricing as a
        float.

        Returns
        -------
        float

        See Also
        --------
        price_ratio

        """

    @property
    def price_ratio(self) -> int:
        """
        The value used for price calculation in spread and leg pricing where every 1 unit
        corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_price_ratio

        """

    @property
    def inst_attrib_value(self) -> int:
        """
        A bitmap of instrument eligibility attributes.

        Returns
        -------
        int

        """

    @property
    def underlying_id(self) -> int:
        """
        The `instrument_id` of the first underlying instrument.

        See `Instrument identifiers` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#instrument-identifiers.

        Returns
        -------
        int

        """

    @property
    def raw_instrument_id(self) -> int:
        """
        The instrument ID assigned by the publisher. May be the same as `instrument_id`.

        See `Instrument identifiers` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#instrument-identifiers.

        Returns
        -------
        int

        """

    @property
    def market_depth_implied(self) -> int:
        """
        The implied book depth on the price level data feed.

        Returns
        -------
        int

        """

    @property
    def market_depth(self) -> int:
        """
        The (outright) book depth on the price level data feed.

        Returns
        -------
        int

        """

    @property
    def market_segment_id(self) -> int:
        """
        The market segment of the instrument.

        Returns
        -------
        int

        """

    @property
    def max_trade_vol(self) -> int:
        """
        The maximum trading volume for the instrument.

        Returns
        -------
        int

        """

    @property
    def min_lot_size(self) -> int:
        """
        The minimum order entry quantity for the instrument.

        Returns
        -------
        int

        """

    @property
    def min_lot_size_block(self) -> int:
        """
        The minimum quantity required for a block trade of the instrument.

        Returns
        -------
        int

        """

    @property
    def min_lot_size_round_lot(self) -> int:
        """
        The minimum quantity required for a round lot of the instrument.
        Multiples of this quantity are also round lots.

        Returns
        -------
        int

        """

    @property
    def min_trade_vol(self) -> int:
        """
        The minimum trading volume for the instrument.

        Returns
        -------
        int

        """

    @property
    def contract_multiplier(self) -> int:
        """
        The number of deliverables per instrument, i.e. peak days.

        Returns
        -------
        int

        """

    @property
    def decay_quantity(self) -> int:
        """
        The quantity that a contract will decay daily, after `decay_start_date`
        has been reached.

        Returns
        -------
        int

        """

    @property
    def original_contract_size(self) -> int:
        """
        The fixed contract value assigned to each instrument.

        Returns
        -------
        int

        """

    @property
    def trading_reference_date(self) -> int:
        """
        The trading session date corresponding to the settlement price in
        `trading_reference_price`, in number of days since the UNIX epoch.

        Returns
        -------
        int

        """

    @property
    def appl_id(self) -> int:
        """
        The channel ID assigned at the venue.

        Returns
        -------
        int

        """

    @property
    def maturity_year(self) -> int:
        """
        The calendar year reflected in the instrument symbol.

        Returns
        -------
        int

        """

    @property
    def decay_start_date(self) -> int:
        """
        The date at which a contract will begin to decay.

        Returns
        -------
        int

        """

    @property
    def channel_id(self) -> int:
        """
        The channel ID assigned by Databento as an incrementing integer
        starting at zero.

        Returns
        -------
        int

        """

    @property
    def currency(self) -> str:
        """
        The currency used for price fields.

        Returns
        -------
        str

        """

    @property
    def settl_currency(self) -> str:
        """
        The currency used for settlement, if different from `currency`.

        Returns
        -------
        str

        """

    @property
    def secsubtype(self) -> str:
        """
        The strategy type of the spread.

        Returns
        -------
        str

        """

    @property
    def raw_symbol(self) -> str:
        """
        The instrument name (symbol).

        Returns
        -------
        str

        """

    @property
    def group(self) -> str:
        """
        The security group code of the instrument.

        Returns
        -------
        str

        """

    @property
    def exchange(self) -> str:
        """
        The exchange used to identify the instrument.

        Returns
        -------
        str

        """

    @property
    def asset(self) -> str:
        """
        The underlying asset code (product code) of the instrument.

        Returns
        -------
        str

        """

    @property
    def cfi(self) -> str:
        """
        The ISO standard instrument categorization code.

        Returns
        -------
        str

        """

    @property
    def security_type(self) -> str:
        """
        The security type of the instrument, e.g. FUT for future or future spread.

        See `Security Type` https://databento.com/docs/schemas-and-data-formats/instrument-definitions#security-type.

        Returns
        -------
        str

        """

    @property
    def unit_of_measure(self) -> str:
        """
        The unit of measure for the instrument’s original contract size, e.g.
        USD or LBS.

        Returns
        -------
        str

        """

    @property
    def underlying(self) -> str:
        """
        The symbol of the first underlying instrument.

        Returns
        -------
        str

        """

    @property
    def strike_price_currency(self) -> str:
        """
        The currency of `strike_price`.

        Returns
        -------
        str

        """

    @property
    def instrument_class(self) -> str:
        """
        The classification of the instrument.

        See `Instrument class` https://databento.com/docs/schemas-and-data-formats/instrument-definitions#instrument-class

        Returns
        -------
        str

        """

    @property
    def pretty_strike_price(self) -> float:
        """
        The strike price of the option as a float.

        Returns
        -------
        float

        See Also
        --------
        strike_price

        """

    @property
    def strike_price(self) -> int:
        """
        The strike price of the option where every 1 unit corresponds to 1e-9, i.e.
        1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_strike_price

        """

    @property
    def match_algorithm(self) -> str:
        """
        The matching algorithm used for the instrument, typically **F**IFO.

        See `Matching algorithm` https://databento.com/docs/schemas-and-data-formats/instrument-definitions#matching-algorithm.

        Returns
        -------
        str

        """

    @property
    def md_security_trading_status(self) -> int:
        """
        The current trading state of the instrument.

        Returns
        -------
        int

        """

    @property
    def main_fraction(self) -> int:
        """
        The price denominator of the main fraction.

        Returns
        -------
        int

        """

    @property
    def price_display_format(self) -> int:
        """
        The number of digits to the right of the tick mark, to display
        fractional prices.

        Returns
        -------
        int

        """

    @property
    def settl_price_type(self) -> int:
        """
        The type indicators for the settlement price, as a bitmap.

        Returns
        -------
        int

        """

    @property
    def sub_fraction(self) -> int:
        """
        The price denominator of the sub fraction.

        Returns
        -------
        int

        """

    @property
    def underlying_product(self) -> int:
        """
        The product complex of the instrument.

        Returns
        -------
        int

        """

    @property
    def security_update_action(self) -> str:
        """
        Indicates if the instrument definition has been added, modified, or
        deleted.

        Returns
        -------
        str

        """

    @property
    def maturity_month(self) -> int:
        """
        The calendar month reflected in the instrument symbol.

        Returns
        -------
        int

        """

    @property
    def maturity_day(self) -> int:
        """
        The calendar day reflected in the instrument symbol, or 0.

        Returns
        -------
        int

        """

    @property
    def maturity_week(self) -> int:
        """
        The calendar week reflected in the instrument symbol, or 0.

        Returns
        -------
        int

        """

    @property
    def user_defined_instrument(self) -> str:
        """
        Indicates if the instrument is user defined: **Y**es or **N**o.

        Returns
        -------
        str

        """

    @property
    def contract_multiplier_unit(self) -> int:
        """
        The type of `contract_multiplier`. Either `1` for hours, or `2` for
        days.

        Returns
        -------
        int

        """

    @property
    def flow_schedule_type(self) -> int:
        """
        The schedule for delivering electricity.

        Returns
        -------
        int

        """

    @property
    def tick_rule(self) -> int:
        """
        The tick rule of the spread.

        Returns
        -------
        int

        """

class ImbalanceMsg(Record):
    """
    An auction imbalance message.
    """

    def __init__(
        self,
        publisher_id: int,
        instrument_id: int,
        ts_event: int,
        ts_recv: int,
        ref_price: int,
        cont_book_clr_price: int,
        auct_interest_clr_price: int,
        paired_qty: int,
        total_imbalance_qty: int,
        auction_type: str,
        side: str,
        significant_imbalance: str,
    ) -> None: ...
    @property
    def pretty_ts_recv(self) -> dt.datetime | None:
        """
        The capture-server-received timestamp as a datetime or
        `pandas.Timestamp`, if available.

        Returns
        -------
        datetime.datetime

        """

    @property
    def ts_recv(self) -> int:
        """
        The capture-server-received timestamp expressed as the number of nanoseconds
        since the UNIX epoch.

        See `ts_recv` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv.

        Returns
        -------
        int

        """

    @property
    def pretty_ref_price(self) -> float:
        """
        The price at which the imbalance shares are calculated as a float.

        Returns
        -------
        float

        See Also
        --------
        ref_price

        """

    @property
    def ref_price(self) -> int:
        """
        The price at which the imbalance shares are calculated, where every 1
        unit corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_ref_price

        """

    @property
    def auction_time(self) -> int:
        """
        Reserved for future use.

        Returns
        -------
        int

        """

    @property
    def pretty_cont_book_clr_price(self) -> float:
        """
        The hypothetical auction-clearing price for both cross and continuous
        orders as a float.

        Returns
        -------
        float

        See Also
        --------
        cont_book_clr_price

        """

    @property
    def cont_book_clr_price(self) -> int:
        """
        The hypothetical auction-clearing price for both cross and continuous
        orders where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
        0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_cont_book_clr_price

        """

    @property
    def pretty_auct_interest_clr_price(self) -> float:
        """
        The hypothetical auction-clearing price for cross orders only as a
        float.

        Returns
        -------
        float

        See Also
        --------
        auct_interest_clr_price

        """

    @property
    def auct_interest_clr_price(self) -> int:
        """
        The hypothetical auction-clearing price for cross orders only where
        every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_auct_interest_clr_price

        """

    @property
    def ssr_filling_price(self) -> int:
        """
        Reserved for future use.

        Returns
        -------
        int

        """

    @property
    def ind_match_price(self) -> int:
        """
        Reserved for future use.

        Returns
        -------
        int

        """

    @property
    def upper_collar(self) -> int:
        """
        Reserved for future use.

        Returns
        -------
        int

        """

    @property
    def lower_collar(self) -> int:
        """
        Reserved for future use.

        Returns
        -------
        int

        """

    @property
    def paired_qty(self) -> int:
        """
        The quantity of shares that are eligible to be matched at `ref_price`.

        Returns
        -------
        int

        """

    @property
    def total_imbalance_qty(self) -> int:
        """
        The quantity of shares that are not paired at `ref_price`.

        Returns
        -------
        int

        """

    @property
    def market_imbalance_qty(self) -> int:
        """
        Reserved for future use.

        Returns
        -------
        int

        """

    @property
    def unpaired_qty(self) -> int:
        """
        Reserved for future use.

        Returns
        -------
        int

        """

    @property
    def auction_type(self) -> str:
        """
        Venue-specific character code indicating the auction type.

        Returns
        -------
        str

        """

    @property
    def side(self) -> str:
        """
        The market side of the `total_imbalance_qty`. Can be **A**sk, **B**id, or
        **N**one.

        Returns
        -------
        str

        """

    @property
    def auction_status(self) -> int:
        """
        Reserved for future use.

        Returns
        -------
        int

        """

    @property
    def freeze_status(self) -> int:
        """
        Reserved for future use.

        Returns
        -------
        int

        """

    @property
    def num_extensions(self) -> int:
        """
        Reserved for future use.

        Returns
        -------
        int

        """

    @property
    def unpaired_side(self) -> str:
        """
        Reserved for future use.

        Returns
        -------
        str

        """

    @property
    def significant_imbalance(self) -> str:
        """
        Venue-specific character code. For Nasdaq, contains the raw Price
        Variation Indicator.

        Returns
        -------
        str

        """

class StatMsgV1(Record):
    """
    A statistics message.

    A catchall for various data disseminated by publishers. The
    `stat_type` field indicates the statistic contained in the message.

    """

    def __init__(
        self,
        publisher_id: int,
        instrument_id: int,
        ts_event: int,
        ts_recv: int,
        ts_ref: int,
        price: int,
        quantity: int,
        stat_type: int,
        sequence: int | None = None,
        ts_in_delta: int | None = None,
        channel_id: int | None = None,
        update_action: int | None = None,
        stat_flags: int = 0,
    ) -> None: ...
    @property
    def pretty_ts_recv(self) -> dt.datetime | None:
        """
        The capture-server-received timestamp as a datetime or
        `pandas.Timestamp`, if available.

        Returns
        -------
        datetime.datetime

        """

    @property
    def ts_recv(self) -> int:
        """
        The capture-server-received timestamp expressed as the number of nanoseconds
        since the UNIX epoch.

        See `ts_recv` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv.

        Returns
        -------
        int

        """

    @property
    def pretty_ts_ref(self) -> dt.datetime | None:
        """
        Reference timestamp expressed as the number of nanoseconds since the
        UNIX epoch as a datetime or `pandas.Timestamp`, if available.

        Returns
        -------
        datetime.datetime

        """

    @property
    def ts_ref(self) -> int:
        """
        The reference timestamp of the statistic value expressed as the number of
        nanoseconds since the UNIX epoch. Will be `UNDEF_TIMESTAMP` when unused.

        Returns
        -------
        int

        """

    @property
    def pretty_price(self) -> float:
        """
        The value for price statistics as a float. Will be nan when unused.

        Returns
        -------
        float

        See Also
        --------
        price

        """

    @property
    def price(self) -> int:
        """
        The order price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
        0.000000001.

        See `Prices` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices.

        Returns
        -------
        int

        See Also
        --------
        pretty_price

        """

    @property
    def quantity(self) -> int:
        """
        The value for non-price statistics. Will be undefined when unused.

        Returns
        -------
        int

        """

    @property
    def sequence(self) -> int:
        """
        The message sequence number assigned at the venue.

        Returns
        -------
        int

        """

    @property
    def ts_in_delta(self) -> int:
        """
        The matching-engine-sending timestamp expressed as the number of nanoseconds before
        `ts_recv`.

        See `ts_in_delta` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-in-delta.

        Returns
        -------
        int

        """

    @property
    def stat_type(self) -> int:
        """
        The type of statistic value contained in the message.

        Returns
        -------
        int

        """

    @property
    def channel_id(self) -> int:
        """
        A channel ID within the venue.

        Returns
        -------
        int

        """

    @property
    def update_action(self) -> int:
        """
        Indicates if the statistic is newly added (1) or deleted (2). (Deleted
        is only used with some stat types)

        Returns
        -------
        int

        """

    @property
    def stat_flags(self) -> int:
        """
        Additional flags associate with certain stat types.

        Returns
        -------
        int

        """

class StatMsg(StatMsgV1):
    """
    A statistics message.

    A catchall for various data disseminated by publishers. The
    `stat_type` field indicates the statistic contained in the message.

    """

class StatusMsg(Record):
    """
    A trading status update message.
    """

    @property
    def pretty_ts_recv(self) -> dt.datetime | None:
        """
        The capture-server-received timestamp as a datetime or
        `pandas.Timestamp`, if available.

        Returns
        -------
        datetime.datetime

        """

    @property
    def ts_recv(self) -> int:
        """
        The capture-server-received timestamp expressed as the number of nanoseconds
        since the UNIX epoch.

        See `ts_recv` https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv.

        Returns
        -------
        int

        """

    @property
    def action(self) -> int:
        """
        The type of status change.

        Returns
        -------
        int

        """

    @property
    def reason(self) -> int:
        """
        Additional details about the cause of the status change.

        Returns
        -------
        int

        """

    @property
    def trading_event(self) -> int:
        """
        Further information about the status change and its effect on trading.

        Returns
        -------
        int

        """

    @property
    def is_trading(self) -> bool | None:
        """
        The best-efforts state of trading in the instrument, either `Y`, `N`, or `~`.

        Returns
        -------
        bool | None

        """

    @property
    def is_quoting(self) -> bool | None:
        """
        The best-efforts state of quoting in the instrument, either `Y`, `N`, or `~`.

        Returns
        -------
        bool | None

        """

    @property
    def is_short_sell_restricted(self) -> bool | None:
        """
        The best-efforts state of short sell restrictions for the instrument (if applicable),
        either `Y`, `N`, or `~`.

        Returns
        -------
        bool | None

        """

class ErrorMsg(ErrorMsgV1):
    """
    An error message from the Databento Live Subscription Gateway (LSG).
    """

    def __init__(
        self, ts_event: int, err: str, is_last: bool = True, code: ErrorCode | None = None
    ) -> None: ...
    @property
    def code(self) -> ErrorCode | None:
        """
        The error code, if any.

        Returns
        -------
        ErrorCode | None
        """

    @property
    def is_last(self) -> int:
        """
        Sometimes multiple errors are sent together. This field will be non-zero for the
        last error.

        Returns
        -------
        int

        """

class ErrorMsgV1(Record):
    """
    A DBN version 1 error message from the Databento Live Subscription Gateway (LSG).
    """

    def __init__(self, ts_event: int, err: str, is_last: bool = True) -> None: ...
    @property
    def err(self) -> str:
        """
        The error message.

        Returns
        -------
        str

        """

class SymbolMappingMsg(Record):
    """A symbol mapping message which maps a symbol of one `SType` to
    another.
    """

    def __init__(
        self,
        publisher_id: int,
        instrument_id: int,
        ts_event: int,
        stype_in: SType,
        stype_in_symbol: str,
        stype_out: SType,
        stype_out_symbol: str,
        start_ts: int,
        end_ts: int,
    ) -> None: ...
    @property
    def stype_in(self) -> SType:
        """
        The input symbology type.

        Returns
        -------
        SType

        """

    @property
    def stype_in_symbol(self) -> str:
        """
        The input symbol.

        Returns
        -------
        str

        """

    @property
    def stype_out(self) -> SType:
        """
        The output symbology type.

        Returns
        -------
        SType

        """

    @property
    def stype_out_symbol(self) -> str:
        """
        The output symbol.

        Returns
        -------
        str

        """

    @property
    def pretty_start_ts(self) -> dt.datetime | None:
        """
        The start of the mapping interval expressed as a datetime
        or `pandas.Timestamp`, if available.

        Returns
        -------
        datetime.datetime

        """

    @property
    def start_ts(self) -> int:
        """
        The start of the mapping interval expressed as the number of
        nanoseconds since the UNIX epoch.

        Returns
        -------
        int

        """

    @property
    def pretty_end_ts(self) -> dt.datetime | None:
        """
        The end of the mapping interval expressed as a datetime
        or `pandas.Timestamp`, if available.

        Returns
        -------
        datetime.datetime

        """

    @property
    def end_ts(self) -> int:
        """
        The end of the mapping interval expressed as the number of nanoseconds
        since the UNIX epoch.

        Returns
        -------
        int

        """

class SymbolMappingMsgV1(SymbolMappingMsg):
    """A DBN version 1 symbol mapping message which maps a symbol of one `SType`
    to another.
    """

class SystemMsg(SystemMsgV1):
    """
    A non-error message from the Databento Live Subscription Gateway (LSG).

    Also used for heartbeating.

    """

    def __init__(self, ts_event: int, msg: str, code: SystemCode | None = None) -> None: ...
    @property
    def code(self) -> SystemCode | None:
        """
        Type of system message, if any.

        Returns
        -------
        SystemCode | None
        """

class SystemMsgV1(Record):
    """
    A DBN version 1 non-error message from the Databento Live Subscription Gateway
    (LSG).

    Also used for heartbeating.

    """

    def __init__(self, ts_event: int, msg: str) -> None: ...
    @property
    def msg(self) -> str:
        """
        The message from the Databento Live Subscription Gateway (LSG).

        Returns
        -------
        str

        """

    def is_heartbeat(self) -> bool:
        """
        Return `true` if this message is a heartbeat, used to indicate the connection
        with the gateway is still open.

        Returns
        -------
        bool

        """

class DBNDecoder:
    """
    A class for decoding DBN data to Python objects.

    Parameters
    ----------
    has_metadata : bool, default True
        Whether the input bytes begin with DBN metadata. Pass False to decode
        individual records or a fragment of a DBN stream.
    ts_out : bool, default False
        Whether the records include the server send timestamp ts_out. Only needs to be
        specified if `has_metadata` is False.
    input_version : int, default None
        Specify the DBN version of the input. Only used when decoding data without
        metadata.
    upgrade_policy : VersionUpgradePolicy, default UPGRADE
        How to decode data from prior DBN versions. Defaults to upgrade decoding.
    """

    def __init__(
        self,
        has_metadata: bool = True,
        ts_out: bool = False,
        input_version: int | None = None,
        upgrade_policy: VersionUpgradePolicy | None = None,
    ): ...
    def buffer(self) -> bytes:
        """
        Return the internal buffer of the decoder.

        Returns
        -------
        bytes

        """

    def decode(
        self,
    ) -> list[_DBNRecord]:
        """
        Decode the buffered data into DBN records.

        Returns
        -------
        list[DBNRecord]

        Raises
        ------
        DBNError
            When the decoding fails.

        See Also
        --------
        write

        """

    def write(
        self,
        bytes: bytes,
    ) -> None:
        """
        Write a sequence of bytes to the internal buffer of the DBNDecoder.

        Raises
        ------
        DBNError
            When the write to the internal buffer fails.

        See Also
        --------
        decode

        """

class Transcoder:
    """
    A class for transcoding DBN i.e. converting it from one compression and encoding to
    another.

    Parameters
    ----------
    file : BinaryIO | TextIO
        The file-like object to write the transcoded output to.
    encoding : Encoding
        The encoding for the output.
    compression : Compression
        The compression for the output.
    pretty_px : bool, default True
        Whether to serialize fixed-precision prices as decimal strings. Only applicable
        to CSV and JSON.
    pretty_ts : bool, default Tru | Nonee
        Whether to serialize nanosecond UNIX timestamps as ISO8601 datetime strings.
        Only applicable to CSV and JSON.
    map_symbols : bool, default None
        If symbology mappings from the metadata should be used to create
        a 'symbol' field, mapping the instrument ID to its requested symbol for
        every record. Defaults to True for text encodings and False for DBN.
    has_metadata : bool, default True
        Whether the input bytes begin with DBN metadata. Pass False to transcode
        individual records or a fragment of a DBN stream.
    ts_out : bool, default False
        Whether the records include the server send timestamp ts_out. Only needs to be
        specified if `has_metadata` is False.
    symbol_interval_map : dict[int, list[tuple[datetime.date, datetime.date, str]]], default None
        Specify the initial symbol mappings to use with map_symbols. If not specified,
        only the mappings in the metadata header will be used.
    schema : Schema | None, default None
        The data record schema to encode. This is required for transcoding Live CSV data,
        as the tabular format is incompatible with mixed schemas.
    input_version : int, default None
        Specify the DBN version of the input. Only used when transcoding data without
        metadata.
    upgrade_policy : VersionUpgradePolicy, default UPGRADE
        How to decode data from prior DBN versions. Defaults to upgrade decoding.
    """

    def __init__(
        self,
        file: BinaryIO | TextIO,
        encoding: Encoding,
        compression: Compression,
        pretty_px: bool = True,
        pretty_ts: bool = True,
        map_symbols: bool | None = None,
        has_metadata: bool = True,
        ts_out: bool = False,
        symbol_interval_map: dict[int, list[tuple[dt.date, dt.date, str]]] | None = None,
        schema: Schema | None = None,
        input_version: int | None = None,
        upgrade_policy: VersionUpgradePolicy | None = None,
    ): ...
    def buffer(self) -> bytes:
        """
        Return the internal buffer of the decoder.

        Returns
        -------
        bytes
        """

    def write(
        self,
        bytes: bytes,
    ) -> None:
        """
        Write a sequence of bytes to the internal buffer for transcoding.

        Raises
        ------
        DBNError
            When the write to the internal buffer or the output fails.
        """

    def flush(
        self,
    ) -> None:
        """
        Flushes remaining bytes from buffer through to the output file.

        Raises
        ------
        DBNError
            When the write to the output fails.
        """

def update_encoded_metadata(
    file: BinaryIO,
    start: int,
    end: int | None = None,
    limit: int | None = None,
) -> None:
    """
    Update existing fields that have already been written to the given file.

    Parameters
    ----------
    file : BinaryIO
        The file handle to update.
    start : int
        The UNIX nanosecond timestamp of the query start, or the
        first record if the file was split.
    end : int | None
        The UNIX nanosecond timestamp of the query end, or the
        last record if the file was split.
    limit : int | None
        The optional maximum number of records for the query.

    Raises
    ------
    DBNError
        When the file update fails.

    """

# Aliases
TBBOMsg = MBOMsg
BBO1SMsg = BBOMsg
BBO1MMsg = BBOMsg
TCBBOMsg = CMBP1Msg
CBBO1SMsg = CBBOMsg
CBBO1MMsg = CBBOMsg
