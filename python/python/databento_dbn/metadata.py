import datetime as dt
from collections.abc import Sequence
from typing import Protocol
from typing import TypedDict


class MappingInterval(Protocol):
    """
    Represents a symbol mapping over a start and end date range interval.

    Parameters
    ----------
    start_date : dt.date
        The start of the mapping period.
    end_date : dt.date
        The end of the mapping period.
    symbol : str
        The symbol value.

    """

    start_date: dt.date
    end_date: dt.date
    symbol: str


class MappingIntervalDict(TypedDict):
    """
    Represents a symbol mapping over a start and end date range interval.

    Parameters
    ----------
    start_date : dt.date
        The start of the mapping period.
    end_date : dt.date
        The end of the mapping period.
    symbol : str
        The symbol value.

    """

    start_date: dt.date
    end_date: dt.date
    symbol: str


class SymbolMapping(Protocol):
    """
    Represents the mappings for one native symbol.
    """

    raw_symbol: str
    intervals: Sequence[MappingInterval]


class SymbolInterval(Protocol):
    """
    Represents a symbol mapping over a nanosecond-precision time range.

    Parameters
    ----------
    start_ts : int
        The start of the interval (inclusive) as nanoseconds since the UNIX epoch.
    end_ts : int
        The end of the interval (exclusive) as nanoseconds since the UNIX epoch.
    instrument_id : int
        The numeric instrument ID.
    stype_in_symbol : str
        The symbol in `stype_in`.
    stype_out_symbol : str
        The symbol in `stype_out`.

    """

    start_ts: int
    end_ts: int
    instrument_id: int
    stype_in_symbol: str
    stype_out_symbol: str


class SymbolIntervalDict(TypedDict):
    """
    Represents a symbol mapping over a nanosecond-precision time range.

    Parameters
    ----------
    start_ts : int
        The start of the interval (inclusive) as nanoseconds since the UNIX epoch.
    end_ts : int
        The end of the interval (exclusive) as nanoseconds since the UNIX epoch.
    instrument_id : int
        The numeric instrument ID.
    stype_in_symbol : str
        The symbol in `stype_in`.
    stype_out_symbol : str
        The symbol in `stype_out`.

    """

    start_ts: int
    end_ts: int
    instrument_id: int
    stype_in_symbol: str
    stype_out_symbol: str
