# ruff: noqa: F401, F405, UP007
from typing import Union

from databento_dbn.metadata import MappingInterval
from databento_dbn.metadata import MappingIntervalDict
from databento_dbn.metadata import SymbolMapping

# Import native module
from ._lib import *  # noqa: F403


DBNRecord = Union[
    MBOMsg,
    TradeMsg,
    MBP1Msg,
    MBP10Msg,
    BBOMsg,
    CMBP1Msg,
    CBBOMsg,
    OHLCVMsg,
    StatusMsg,
    InstrumentDefMsg,
    ImbalanceMsg,
    StatMsg,
    ErrorMsg,
    SymbolMappingMsg,
    SystemMsg,
    ErrorMsgV1,
    InstrumentDefMsgV1,
    StatMsgV1,
    SymbolMappingMsgV1,
    SystemMsgV1,
    InstrumentDefMsgV2,
]
