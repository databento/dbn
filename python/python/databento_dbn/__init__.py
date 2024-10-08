import datetime as dt
from collections.abc import Sequence
from typing import Protocol
from typing import TypedDict

# Import native module
from ._lib import *  # noqa: F403


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
