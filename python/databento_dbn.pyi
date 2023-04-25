"""Type stubs for databento_dbn."""
from typing import (
    Any,
    BinaryIO,
    Dict,
    List,
    Optional,
    Sequence,
    SupportsBytes,
    Tuple,
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
]

class Metadata(SupportsBytes):
    """
    Information about the data contained in a DBN file or stream. DBN requires
    the Metadata to be included at the start of the encoded data.

    See Also
    ---------
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
    def schema(self) -> Optional[str]:
        """
        The data record schema. Specifies which record type is stored in the
        Zstd-compressed DBN file.

        Returns
        -------
        Optional[str]

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
    def stype_in(self) -> Optional[str]:
        """
        The input symbology type to map from.

        Returns
        -------
        Optional[str]

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
    def symbols(self) -> List[str]:
        """
        The original query input symbols from the request.

        Returns
        -------
        List[str]

        """
    @property
    def partial(self) -> List[str]:
        """
        Symbols that did not resolve for at least one day in the query time
        range.

        Returns
        -------
        List[str]

        """
    @property
    def not_found(self) -> List[str]:
        """
        Symbols that did not resolve for any day in the query time range.

        Returns
        -------
        List[str]

        """
    @property
    def mappings(self) -> Dict[str, List[Dict[str, Any]]]:
        """
        Symbol mappings containing a native symbol and its mapping intervals.

        Returns
        -------
        Dict[str, List[Dict[str, Any]]]:

        """
    @classmethod
    def decode(cls, data: bytes) -> "Metadata":
        """
        Decodes the given Python `bytes` to `Metadata`. Returns a `Metadata`
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
        Encodes the Metadata to bytes.

        Returns
        -------
        bytes

        Raises
        ------
        ValueError
            When the Metadata object cannot be encoded.

        """

class RecordHeader:
    """DBN Record Header."""

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
    """Base class for DBN records."""

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
    def record_size(self) -> int:
        """
        Return the size of the record.

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

class _MBOBase:
    """Base for market-by-order messages."""

    @property
    def order_id(self) -> int:
        """
        The order ID assigned at the venue.

        Returns
        -------
        int

        """
    @property
    def price(self) -> int:
        """
        The order price expressed as a signed integer where every 1 unit
        corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

        Returns
        -------
        int

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
    """A market-by-order (MBO) tick message."""

class BidAskPair:
    """A book level."""

    @property
    def bid_px(self) -> int:
        """
        The bid price.

        Returns
        -------
        int

        """
    @property
    def ask_px(self) -> int:
        """
        The ask price.

        Returns
        -------
        int

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
    """Base for market-by-price messages."""

    @property
    def price(self) -> int:
        """
        The order price expressed as a signed integer where every 1 unit
        corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

        Returns
        -------
        int

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
    """Market by price implementation with a known book depth of 1."""

    @property
    def booklevel(self) -> List[BidAskPair]:
        """
        The top of the order book.

        Returns
        -------
        List[BidAskPair]

        Notes
        -----
        MBP1Msg contains 1 level of BidAskPair.

        """

class MBP10Msg(Record, _MBPBase):
    """Market by price implementation with a known book depth of 10."""

    @property
    def booklevel(self) -> List[BidAskPair]:
        """
        The top of the order book.

        Returns
        -------
        List[BidAskPair]

        Notes
        -----
        MBP10Msg contains 10 levels of BidAskPairs.

        """

class OHLCVMsg(Record):
    """Open, high, low, close, and volume message."""

    @property
    def open(self) -> int:
        """
        The open price for the bar.

        Returns
        -------
        int

        """
    @property
    def high(self) -> int:
        """
        The high price for the bar.

        Returns
        -------
        int

        """
    @property
    def low(self) -> int:
        """
        The low price for the bar.

        Returns
        -------
        int

        """
    @property
    def close(self) -> int:
        """
        The close price for the bar.

        Returns
        -------
        int

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
    """Definition of an instrument."""

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
    def min_price_increment(self) -> int:
        """
        The minimum constant tick for the instrument in units of 1e-9, i.e.
        1/1,000,000,000 or 0.000000001.

        Returns
        -------
        int

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
    def expiration(self) -> int:
        """
        The last eligible trade time expressed as a number of nanoseconds since
        the UNIX epoch.

        Returns
        -------
        int

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
    def high_limit_price(self) -> int:
        """
        The allowable high limit price for the trading day in units of 1e-9,
        i.e. 1/1,000,000,000 or 0.000000001.

        Returns
        -------
        int

        """
    @property
    def low_limit_price(self) -> int:
        """
        The allowable low limit price for the trading day in units of 1e-9,
        i.e. 1/1,000,000,000 or 0.000000001.

        Returns
        -------
        int

        """
    @property
    def max_price_variation(self) -> int:
        """
        The differential value for price banding in units of 1e-9, i.e.
        1/1,000,000,000 or 0.000000001.

        Returns
        -------
        int

        """
    @property
    def trading_reference_price(self) -> int:
        """
        The trading session settlement price on `trading_reference_date`.

        Returns
        -------
        int

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
    def min_price_increment_amount(self) -> int:
        """
        The value currently under development by the venue. Converted to units
        of 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

        Returns
        -------
        int

        """
    @property
    def price_ratio(self) -> int:
        """
        The value used for price calculation in spread and leg pricing in units
        of 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

        Returns
        -------
        int

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
    def cleared_volume(self) -> int:
        """
        The total cleared volume of the instrument traded during the prior
        trading session.

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
    def open_interest_qty(self) -> int:
        """
        The total open interest for the market at the close of the prior
        trading session.

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
    def strike_price(self) -> int:
        """
        The strike price of the option. Converted to units of 1e-9, i.e.
        1/1,000,000,000 or 0.000000001.

        Returns
        -------
        int

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
    """An auction imbalance message."""

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
    def ref_price(self) -> int:
        """
        The price at which the imbalance shares are calculated, where every 1
        unit corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.

        Returns
        -------
        int

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
    def cont_book_clr_price(self) -> int:
        """
        The hypothetical auction-clearing price for both cross and continuous
        orders.

        Returns
        -------
        int

        """
    @property
    def auct_interest_clr_price(self) -> int:
        """
        The hypothetical auction-clearing price for cross orders only.

        Returns
        -------
        int

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

class ErrorMsg(Record):
    """An error message from the Databento Live Subscription Gateway (LSG)."""

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
    another."""

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
    def start_ts(self) -> int:
        """
        The start of the mapping interval expressed as the number of
        nanoseconds since the UNIX epoch.

        Returns
        -------
        int

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
        `true` if this message is a heartbeat, used to indicate the connection with the gateway
        is still open.

        Returns
        -------
        bool

        """

class DbnDecoder:
    """A class for decoding DBN data to Python objects."""

    def buffer(self) -> bytes:
        """
        Return the internal buffer of the decoder.

        Returns
        -------
        bytes

        """
    def decode(
        self,
    ) -> List[Tuple[_DBNRecord, Optional[int]]]:
        """
        Decode the buffered data into DBN records.

        Returns
        -------
        List[Tuple[DBNRecord, Optional[int]]]

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
        Write a sequence of bytes to the internal buffer of the DbnDecoder.

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
    end: Optional[int] = None,
    limit: Optional[int] = None,
) -> None:
    """
    Updates existing fields that have already been written to the given file.

    Parameters
    ----------
    file : BinaryIO
        The file handle to update.
    start : int
        The UNIX nanosecond timestamp of the query start, or the
        first record if the file was split.
    end : Optional[int]
        The UNIX nanosecond timestamp of the query end, or the
        last record if the file was split.
    limit : Optional[int]
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
    end: Optional[int] = None,
) -> None:
    """
    Encodes the given data in the DBN encoding and writes it to `file`.

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
    end : Optional[int]
        The UNIX nanosecond timestamp of the query end, or the
        last record if the file was split.

    Raises
    ------
    ValueError
        When any of the enum arguments cannot be converted to their Rust equivalents.
        When there's an issue writing the encoded to bytes.
        When an expected field is missing from one of the dicts.

    """
