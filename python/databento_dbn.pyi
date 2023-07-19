# ruff: noqa: UP007 PYI021 PYI053
from __future__ import annotations

from collections.abc import Iterable
from collections.abc import Sequence
from datetime import datetime
from enum import Enum
from typing import (
    Any,
    BinaryIO,
    SupportsBytes,
    Union,
)


_DBNRecord = Union[
    Metadata,
    MBOMsg,
    MBP1Msg,
    MBP10Msg,
    OHLCVMsg,
    TradeMsg,
    InstrumentDefMsg,
    ImbalanceMsg,
    ErrorMsg,
    SymbolMappingMsg,
    SystemMsg,
    StatMsg,
]

class Compression(Enum):
    """
    Data compression format.

    NONE
        Uncompressed
    ZSTD
        Zstandard compressed.

    """

    NONE: str
    ZSTD: str

    @classmethod
    def from_str(cls, str) -> Compression: ...
    @classmethod
    def variants(cls) -> Iterable[Compression]: ...

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
    def from_str(cls, str) -> Encoding: ...
    @classmethod
    def variants(cls) -> Iterable[Compression]: ...

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
    DEFINITION
        Instrument definitions.
    STATISTICS
        Additional data disseminated by publishers.
    STATUS
        Exchange status.
    IMBALANCE
        Auction imbalance events.

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
    DEFINITION: str
    STATISTICS: str
    STATUS: str
    IMBALANCE: str

    @classmethod
    def from_str(cls, str) -> Schema: ...
    @classmethod
    def variants(cls) -> Iterable[Schema]: ...

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

    """

    INSTRUMENT_ID: str
    RAW_SYMBOL: str
    CONTINUOUS: str
    PARENT: str

    @classmethod
    def from_str(cls, str) -> SType: ...
    @classmethod
    def variants(cls) -> Iterable[SType]: ...

class Metadata(SupportsBytes):
    """
    Information about the data contained in a DBN file or stream. DBN requires
    the Metadata to be included at the start of the encoded data.

    See Also
    --------
    decode_metadata
    encode_metadata

    """

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
    def stype_in(self) -> str | None:
        """
        The input symbology type to map from.

        Returns
        -------
        str | None

        """
    @property
    def stype_out(self) -> str:
        """
        The output symbology type to map to.

        Returns
        -------
        str

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
    def mappings(self) -> dict[str, list[dict[str, Any]]]:
        """
        Symbol mappings containing a native symbol and its mapping intervals.

        Returns
        -------
        dict[str, list[dict[str, Any]]]:

        """
    @classmethod
    def decode(cls, data: bytes) -> Metadata:
        """
        Decode the given Python `bytes` to `Metadata`. Returns a `Metadata`
        object with all the DBN metadata attributes.

        Parameters
        ----------
        data : bytes
            The bytes to decode from.

        Returns
        -------
        Metadata

        Raises
        ------
        ValueError
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
        ValueError
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
        The publisher ID assigned by Databento.

        Returns
        -------
        int

        """
    @property
    def instrument_id(self) -> int:
        """
        The numeric ID assigned to the instrument.

        Returns
        -------
        int

        """
    @property
    def ts_event(self) -> int:
        """
        The matching-engine-received timestamp expressed as number of
        nanoseconds since the UNIX epoch.

        Returns
        -------
        int

        """

class Record(SupportsBytes):
    """
    Base class for DBN records.
    """

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
    @classmethod
    def size_hint(cls) -> int:
        """
        Return an estimated size of the record in bytes.

        Returns
        -------
        int

        See Also
        --------
        record_size

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
        The publisher ID assigned by Databento.

        Returns
        -------
        int

        """
    @property
    def instrument_id(self) -> int:
        """
        The numeric ID assigned to the instrument.

        Returns
        -------
        int

        """
    @property
    def pretty_ts_event(self) -> datetime:
        """
        The matching-engine-received timestamp expressed as a
        datetime or a `pandas.Timestamp`, if available.

        Returns
        -------
        datetime

        """
    @property
    def ts_event(self) -> int:
        """
        The matching-engine-received timestamp expressed as number of
        nanoseconds since the UNIX epoch.

        Returns
        -------
        int

        """
    @property
    def ts_out(self) -> int | None:
        """
        The live gateway send timestamp expressed as number of nanoseconds
        since the UNIX epoch.

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
        The order price expressed as a signed integer where every 1 unit
        corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

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
        A combination of packet end with matching engine status.

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
    def action(self) -> str:
        """
        The event action. Can be `A`dd, `C`ancel, `M`odify, clea`R`, `T`rade,
        or `F`ill.

        Returns
        -------
        str

        """
    @property
    def side(self) -> str:
        """
        The order side. Can be `A`sk, `B`id or `N`one.

        Returns
        -------
        str

        """
    @property
    def pretty_ts_recv(self) -> datetime:
        """
        The capture-server-received timestamp as a datetime or
        `pandas.Timestamp`, if available.

        Returns
        -------
        datetime

        """
    @property
    def ts_recv(self) -> int:
        """
        The capture-server-received timestamp expressed as number of
        nanoseconds since the UNIX epoch.

        Returns
        -------
        int

        """
    @property
    def ts_in_delta(self) -> int:
        """
        The delta of `ts_recv - ts_exchange_send`, max 2 seconds.

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

class BidAskPair:
    """
    A book level.
    """

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
        The bid price expressed as a signed integer where every 1 unit
        corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

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
        The ask price as a signed integer where every 1 unit
        corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

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
    def bid_ask_ct(self) -> int:
        """
        The ask order count.

        Returns
        -------
        int

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
        The order price expressed as a signed integer where every 1 unit
        corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

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
        The event action. Can be `A`dd, `C`ancel, `M`odify, clea`R`, or
        `T`rade.

        Returns
        -------
        str

        """
    @property
    def side(self) -> str:
        """
        The order side. Can be `A`sk, `B`id or `N`one.

        Returns
        -------
        str

        """
    @property
    def flags(self) -> int:
        """
        A combination of packet end with matching engine status.

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
    def pretty_ts_recv(self) -> datetime:
        """
        The capture-server-received timestamp as a datetime or
        `pandas.Timestamp`, if available.

        Returns
        -------
        datetime

        """
    @property
    def ts_recv(self) -> int:
        """
        The capture-server-received timestamp expressed as number of
        nanoseconds since the UNIX epoch.

        Returns
        -------
        int

        """
    @property
    def ts_in_delta(self) -> int:
        """
        The delta of `ts_recv - ts_exchange_send`, max 2 seconds.

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

class MBP1Msg(Record, _MBPBase):
    """
    Market by price implementation with a known book depth of 1.
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
        MBP1Msg contains 1 level of BidAskPair.

        """

class MBP10Msg(Record, _MBPBase):
    """
    Market by price implementation with a known book depth of 10.
    """

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
        The open price for the bar expressed as a signed integer where every 1
        unit corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

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
        The high price for the bar expressed as a signed integer where every 1
        unit corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

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
        The low price for the bar expressed as a signed integer where every 1
        unit corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

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
        The close price for the bar expressed as a signed integer where every 1
        unit corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

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
    Definition of an instrument.
    """

    @property
    def pretty_ts_recv(self) -> datetime:
        """
        The capture-server-received timestamp as a datetime or
        `pandas.Timestamp`, if available.

        Returns
        -------
        datetime

        """
    @property
    def ts_recv(self) -> int:
        """
        The capture-server-received timestamp expressed as number of
        nanoseconds since the UNIX epoch.

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
        The minimum constant tick for the instrument in units of 1e-9, i.e.
        1/1,000,000,000 or 0.000000001.

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
        The multiplier to convert the venue’s display price to the conventional
        price.

        Returns
        -------
        int

        """
    @property
    def pretty_expiration(self) -> datetime:
        """
        The last eligible trade time expressed as a datetime or
        `pandas.Timestamp`, if available.

        Returns
        -------
        datetime

        """
    @property
    def expiration(self) -> int:
        """
        The last eligible trade time expressed as a number of nanoseconds since
        the UNIX epoch.

        Returns
        -------
        int

        """
    @property
    def pretty_activation(self) -> datetime:
        """
        The time of instrument activation expressed as a datetime or
        `pandas.Timestamp`, if available.

        Returns
        -------
        datetime

        """
    @property
    def activation(self) -> int:
        """
        The time of instrument activation expressed as a number of nanoseconds
        since the UNIX epoch.

        Returns
        -------
        int

        """
    @property
    def prety_high_limit_price(self) -> float:
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
        The allowable high limit price for the trading day in units of 1e-9,
        i.e. 1/1,000,000,000 or 0.000000001.

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
        The allowable low limit price for the trading day in units of 1e-9,
        i.e. 1/1,000,000,000 or 0.000000001.

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
        The differential value for price banding in units of 1e-9, i.e.
        1/1,000,000,000 or 0.000000001.

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
        The trading session settlement price on `trading_reference_date` in units of 1e-9, i.e.
        1/1,000,000,000 or 0.000000001.

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
        The contract size for each instrument, in combination with
        `unit_of_measure`.

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
        The value currently under development by the venue. Converted to units
        of 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

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
        The value used for price calculation in spread and leg pricing in units
        of 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

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

        Retruns
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
        The type of the instrument, e.g. FUT for future or future spread.

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
        The strike price of the option. Converted to units of 1e-9, i.e.
        1/1,000,000,000 or 0.000000001.

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
        Indicates if the instrument is user defined: `Y`es or `N`o.

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

    @property
    def pretty_ts_recv(self) -> datetime:
        """
        The capture-server-received timestamp as a datetime or
        `pandas.Timestamp`, if available.

        Returns
        -------
        datetime

        """
    @property
    def ts_recv(self) -> int:
        """
        The capture-server-received timestamp expressed as the number of
        nanoseconds since the UNIX epoch.

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
        The market side of the `total_imbalance_qty`. Can be `A`sk, `B`id, or
        `N`one.

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

class StatMsg(Record):
    """
    A statistics message.

    A catchall for various data disseminated by publishers. The
    `stat_type` field indicates the statistic contained in the message.

    """

    @property
    def pretty_ts_recv(self) -> datetime:
        """
        The capture-server-received timestamp as a datetime or
        `pandas.Timestamp`, if available.

        Returns
        -------
        datetime

        """
    @property
    def ts_recv(self) -> int:
        """
        The capture-server-received timestamp expressed as the number of
        nanoseconds since the UNIX epoch.

        Returns
        -------
        int

        """
    @property
    def ts_ref(self) -> int:
        """
        Reference timestamp expressed as the number of nanoseconds since the
        UNIX epoch.

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
        The value for price statistics expressed as a signed integer where
        every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
        Will be undefined when unused.

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
        The delta of `ts_recv - ts_exchange_send`, max 2 seconds.

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

class ErrorMsg(Record):
    """
    An error message from the Databento Live Subscription Gateway (LSG).
    """

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

    @property
    def stype_in_symbol(self) -> str:
        """
        The input symbol.

        Returns
        -------
        str

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
    def pretty_start_ts(self) -> datetime:
        """
        The start of the mapping interval expressed as a datetime
        or `pandas.Timestamp`, if available.

        Returns
        -------
        datetime

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
    def pretty_end_ts(self) -> datetime:
        """
        The end of the mapping interval expressed as a datetime
        or `pandas.Timestamp`, if available.

        Returns
        -------
        datetime

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

class SystemMsg(Record):
    """
    A non-error message from the Databento Live Subscription Gateway (LSG).

    Also used for heartbeating.

    """

    @property
    def msg(self) -> str:
        """
        The message from the Databento Live Subscription Gateway (LSG).

        Returns
        -------
        str

        """
    @property
    def is_heartbeat(self) -> bool:
        """
        `true` if this message is a heartbeat, used to indicate the connection
        with the gateway is still open.

        Returns
        -------
        bool

        """

class DBNDecoder:
    """
    A class for decoding DBN data to Python objects.
    """

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
        ValueError
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
        ValueError
            When the write to the internal buffer fails.

        See Also
        --------
        decode

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
    ValueError
        When the file update fails.

    """

def write_dbn_file(
    file: BinaryIO,
    compression: str,
    dataset: str,
    schema: str,
    start: int,
    stype_in: str,
    stype_out: str,
    records: Sequence[Record],
    end: int | None = None,
) -> None:
    """
    Encode the given data in the DBN encoding and writes it to `file`.

    Parameters
    ----------
    file : BinaryIO
        The file handle to update.
    compression : str
        The DBN compression format.
    dataset : str
       The dataset code.
    schema : str
        The data record schema.
    start : int
        The UNIX nanosecond timestamp of the query start, or the
        first record if the file was split.
    stype_in : str
        The input symbology type to map from.
    stype_out : str
        The output symbology type to map to.
    records : Sequence[object]
        A sequence of DBN record objects.
    end : int | None
        The UNIX nanosecond timestamp of the query end, or the
        last record if the file was split.

    Raises
    ------
    ValueError
        When any of the enum arguments cannot be converted to their Rust equivalents.
        When there's an issue writing the encoded to bytes.
        When an expected field is missing from one of the dicts.

    """
