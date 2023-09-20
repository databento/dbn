use crate::{
    Error, ErrorMsg, ImbalanceMsg, InstrumentDefMsg, MboMsg, Mbp10Msg, Mbp1Msg, OhlcvMsg, RType,
    RecordRef, StatMsg, StatusMsg, SymbolMappingMsg, SystemMsg, TradeMsg,
};

/// An owned DBN record type of flexible type.
#[derive(Debug, Clone)]
pub enum RecordEnum {
    /// An market-by-order message.
    Mbo(MboMsg),
    /// A trade message.
    Trade(TradeMsg),
    /// A market-by-price message with a book depth of 1.
    Mbp1(Mbp1Msg),
    /// A market-by-price message with a book depth of 10.
    Mbp10(Mbp10Msg),
    /// An open, high, low, close, and volume message.
    Ohlcv(OhlcvMsg),
    /// A trading status message.
    Status(StatusMsg),
    /// An instrument definition message.
    InstrumentDef(InstrumentDefMsg),
    /// An auction imbalance message.
    Imbalance(ImbalanceMsg),
    /// A publisher statistic message.
    Stat(StatMsg),
    /// An error message from the Databento Live Subscription Gateway (LSG).
    Error(ErrorMsg),
    /// A symbol mapping message.
    SymbolMapping(SymbolMappingMsg),
    /// A non-error message from the Databento Live Subscription Gateway (LSG).
    System(SystemMsg),
}

/// An immutable reference to a DBN record of flexible type. Unlike [`RecordRef`], this
/// type allows `match`ing.
#[derive(Debug, Copy, Clone)]
pub enum RecordRefEnum<'a> {
    /// A reference to a market-by-order message.
    Mbo(&'a MboMsg),
    /// A reference to a trade message.
    Trade(&'a TradeMsg),
    /// A reference to a market-by-price message with a book depth of 1.
    Mbp1(&'a Mbp1Msg),
    /// A reference to a market-by-price message with a book depth of 10.
    Mbp10(&'a Mbp10Msg),
    /// A reference to an open, high, low, close, and volume message.
    Ohlcv(&'a OhlcvMsg),
    /// A reference to a trading status message.
    Status(&'a StatusMsg),
    /// A reference to an instrument definition message.
    InstrumentDef(&'a InstrumentDefMsg),
    /// A reference to an auction imbalance message.
    Imbalance(&'a ImbalanceMsg),
    /// A reference to a publisher statistic message.
    Stat(&'a StatMsg),
    /// A reference to an error message from the Databento Live Subscription Gateway
    /// (LSG).
    Error(&'a ErrorMsg),
    /// A reference to a symbol mapping message.
    SymbolMapping(&'a SymbolMappingMsg),
    /// A reference to a non-error message from the Databento Live Subscription Gateway
    /// (LSG).
    System(&'a SystemMsg),
}

impl<'a> From<&'a RecordEnum> for RecordRefEnum<'a> {
    fn from(rec_enum: &'a RecordEnum) -> Self {
        match rec_enum {
            RecordEnum::Mbo(rec) => Self::Mbo(rec),
            RecordEnum::Trade(rec) => Self::Trade(rec),
            RecordEnum::Mbp1(rec) => Self::Mbp1(rec),
            RecordEnum::Mbp10(rec) => Self::Mbp10(rec),
            RecordEnum::Ohlcv(rec) => Self::Ohlcv(rec),
            RecordEnum::Status(rec) => Self::Status(rec),
            RecordEnum::InstrumentDef(rec) => Self::InstrumentDef(rec),
            RecordEnum::Imbalance(rec) => Self::Imbalance(rec),
            RecordEnum::Stat(rec) => Self::Stat(rec),
            RecordEnum::Error(rec) => Self::Error(rec),
            RecordEnum::SymbolMapping(rec) => Self::SymbolMapping(rec),
            RecordEnum::System(rec) => Self::System(rec),
        }
    }
}

impl<'a> RecordRefEnum<'a> {
    /// Converts the reference enum into an owned enum value.
    pub fn to_owned(&self) -> RecordEnum {
        #[allow(clippy::clone_on_copy)] // required for when trivial_copy feature is disabled
        match self {
            Self::Mbo(rec) => RecordEnum::from((*rec).clone()),
            Self::Trade(rec) => RecordEnum::from((*rec).clone()),
            Self::Mbp1(rec) => RecordEnum::from((*rec).clone()),
            Self::Mbp10(rec) => RecordEnum::from((*rec).clone()),
            Self::Ohlcv(rec) => RecordEnum::from((*rec).clone()),
            Self::Status(rec) => RecordEnum::from((*rec).clone()),
            Self::InstrumentDef(rec) => RecordEnum::from((*rec).clone()),
            Self::Imbalance(rec) => RecordEnum::from((*rec).clone()),
            Self::Stat(rec) => RecordEnum::from((*rec).clone()),
            Self::Error(rec) => RecordEnum::from((*rec).clone()),
            Self::SymbolMapping(rec) => RecordEnum::from((*rec).clone()),
            Self::System(rec) => RecordEnum::from((*rec).clone()),
        }
    }
}

impl<'a> TryFrom<RecordRef<'a>> for RecordRefEnum<'a> {
    type Error = Error;

    fn try_from(rec_ref: RecordRef<'a>) -> Result<Self, Error> {
        Ok(unsafe {
            #[allow(deprecated)]
            match rec_ref.rtype()? {
                RType::Mbo => RecordRefEnum::Mbo(rec_ref.get_unchecked()),
                RType::Mbp0 => RecordRefEnum::Trade(rec_ref.get_unchecked()),
                RType::Mbp1 => RecordRefEnum::Mbp1(rec_ref.get_unchecked()),
                RType::Mbp10 => RecordRefEnum::Mbp10(rec_ref.get_unchecked()),
                RType::OhlcvDeprecated
                | RType::Ohlcv1S
                | RType::Ohlcv1M
                | RType::Ohlcv1H
                | RType::Ohlcv1D
                | RType::OhlcvEod => RecordRefEnum::Ohlcv(rec_ref.get_unchecked()),
                RType::Status => RecordRefEnum::Status(rec_ref.get_unchecked()),
                RType::InstrumentDef => RecordRefEnum::InstrumentDef(rec_ref.get_unchecked()),
                RType::Imbalance => RecordRefEnum::Imbalance(rec_ref.get_unchecked()),
                RType::Statistics => RecordRefEnum::Stat(rec_ref.get_unchecked()),
                RType::Error => RecordRefEnum::Error(rec_ref.get_unchecked()),
                RType::SymbolMapping => RecordRefEnum::SymbolMapping(rec_ref.get_unchecked()),
                RType::System => RecordRefEnum::System(rec_ref.get_unchecked()),
            }
        })
    }
}

impl From<MboMsg> for RecordEnum {
    fn from(rec: MboMsg) -> Self {
        Self::Mbo(rec)
    }
}
impl<'a> From<&'a MboMsg> for RecordRefEnum<'a> {
    fn from(rec: &'a MboMsg) -> Self {
        Self::Mbo(rec)
    }
}
impl From<TradeMsg> for RecordEnum {
    fn from(rec: TradeMsg) -> Self {
        Self::Trade(rec)
    }
}
impl<'a> From<&'a TradeMsg> for RecordRefEnum<'a> {
    fn from(rec: &'a TradeMsg) -> Self {
        Self::Trade(rec)
    }
}
impl From<Mbp1Msg> for RecordEnum {
    fn from(rec: Mbp1Msg) -> Self {
        Self::Mbp1(rec)
    }
}
impl<'a> From<&'a Mbp1Msg> for RecordRefEnum<'a> {
    fn from(rec: &'a Mbp1Msg) -> Self {
        Self::Mbp1(rec)
    }
}
impl From<Mbp10Msg> for RecordEnum {
    fn from(rec: Mbp10Msg) -> Self {
        Self::Mbp10(rec)
    }
}
impl<'a> From<&'a Mbp10Msg> for RecordRefEnum<'a> {
    fn from(rec: &'a Mbp10Msg) -> Self {
        Self::Mbp10(rec)
    }
}
impl From<OhlcvMsg> for RecordEnum {
    fn from(rec: OhlcvMsg) -> Self {
        Self::Ohlcv(rec)
    }
}
impl<'a> From<&'a OhlcvMsg> for RecordRefEnum<'a> {
    fn from(rec: &'a OhlcvMsg) -> Self {
        Self::Ohlcv(rec)
    }
}
impl From<StatusMsg> for RecordEnum {
    fn from(rec: StatusMsg) -> Self {
        Self::Status(rec)
    }
}
impl<'a> From<&'a StatusMsg> for RecordRefEnum<'a> {
    fn from(rec: &'a StatusMsg) -> Self {
        Self::Status(rec)
    }
}
impl From<InstrumentDefMsg> for RecordEnum {
    fn from(rec: InstrumentDefMsg) -> Self {
        Self::InstrumentDef(rec)
    }
}
impl<'a> From<&'a InstrumentDefMsg> for RecordRefEnum<'a> {
    fn from(rec: &'a InstrumentDefMsg) -> Self {
        Self::InstrumentDef(rec)
    }
}
impl From<ImbalanceMsg> for RecordEnum {
    fn from(rec: ImbalanceMsg) -> Self {
        Self::Imbalance(rec)
    }
}
impl<'a> From<&'a ImbalanceMsg> for RecordRefEnum<'a> {
    fn from(rec: &'a ImbalanceMsg) -> Self {
        Self::Imbalance(rec)
    }
}
impl From<StatMsg> for RecordEnum {
    fn from(rec: StatMsg) -> Self {
        Self::Stat(rec)
    }
}
impl<'a> From<&'a StatMsg> for RecordRefEnum<'a> {
    fn from(rec: &'a StatMsg) -> Self {
        Self::Stat(rec)
    }
}
impl From<ErrorMsg> for RecordEnum {
    fn from(rec: ErrorMsg) -> Self {
        Self::Error(rec)
    }
}
impl<'a> From<&'a ErrorMsg> for RecordRefEnum<'a> {
    fn from(rec: &'a ErrorMsg) -> Self {
        Self::Error(rec)
    }
}
impl From<SymbolMappingMsg> for RecordEnum {
    fn from(rec: SymbolMappingMsg) -> Self {
        Self::SymbolMapping(rec)
    }
}
impl<'a> From<&'a SymbolMappingMsg> for RecordRefEnum<'a> {
    fn from(rec: &'a SymbolMappingMsg) -> Self {
        Self::SymbolMapping(rec)
    }
}
impl From<SystemMsg> for RecordEnum {
    fn from(rec: SystemMsg) -> Self {
        Self::System(rec)
    }
}
impl<'a> From<&'a SystemMsg> for RecordRefEnum<'a> {
    fn from(rec: &'a SystemMsg) -> Self {
        Self::System(rec)
    }
}
