use crate::{
    record::{CbboMsg, Cmbp1Msg},
    BboMsg, Error, ErrorMsg, ImbalanceMsg, InstrumentDefMsg, MboMsg, Mbp10Msg, Mbp1Msg, OhlcvMsg,
    RType, Record, RecordMut, RecordRef, StatMsg, StatusMsg, SymbolMappingMsg, SystemMsg, TradeMsg,
};

/// An owned DBN record type of flexible type. Unlike [`RecordRef`], this type allows
/// `match`ing.
///
/// Note: this type does not support `ts_out`.
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
    /// A consolidated best bid and offer message.
    Cmbp1(Cmbp1Msg),
    /// A subsampled market-by-price message with a book depth of 1.
    Bbo(BboMsg),
    /// A subsampled and consolidated market-by-price message with a book depth of 1.
    Cbbo(CbboMsg),
}

/// An immutable reference to a DBN record of flexible type. Unlike [`RecordRef`], this
/// type allows `match`ing.
///
/// Note: this type does not support `ts_out`.
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
    /// A reference to a consolidated best bid and offer message.
    Cmbp1(&'a Cmbp1Msg),
    /// A subsampled market-by-price message with a book depth of 1.
    Bbo(&'a BboMsg),
    /// A subsampled and consolidated market-by-price message with a book depth of 1.
    Cbbo(&'a CbboMsg),
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
            RecordEnum::Cmbp1(rec) => Self::Cmbp1(rec),
            RecordEnum::Bbo(rec) => Self::Bbo(rec),
            RecordEnum::Cbbo(rec) => Self::Cbbo(rec),
        }
    }
}

impl RecordRefEnum<'_> {
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
            Self::Cmbp1(rec) => RecordEnum::from((*rec).clone()),
            Self::Bbo(rec) => RecordEnum::Bbo((*rec).clone()),
            Self::Cbbo(rec) => RecordEnum::Cbbo((*rec).clone()),
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
                RType::Bbo1S | RType::Bbo1M => RecordRefEnum::Bbo(rec_ref.get_unchecked()),
                RType::Mbp10 => RecordRefEnum::Mbp10(rec_ref.get_unchecked()),
                RType::OhlcvDeprecated
                | RType::Ohlcv1S
                | RType::Ohlcv1M
                | RType::Ohlcv1H
                | RType::Ohlcv1D
                | RType::OhlcvEod => RecordRefEnum::Ohlcv(rec_ref.get_unchecked()),
                RType::Status => RecordRefEnum::Status(rec_ref.get_unchecked()),
                RType::InstrumentDef => {
                    // can't convert V1 structs here because an immutable reference
                    if rec_ref.record_size() < std::mem::size_of::<InstrumentDefMsg>() {
                        return Err(Error::conversion::<Self>(
                            "earlier version of InstrumentDefMsg (must be current version)",
                        ));
                    }
                    RecordRefEnum::InstrumentDef(rec_ref.get_unchecked())
                }
                RType::Imbalance => RecordRefEnum::Imbalance(rec_ref.get_unchecked()),
                RType::Statistics => {
                    if rec_ref.record_size() < std::mem::size_of::<StatMsg>() {
                        return Err(Error::conversion::<Self>(
                            "earlier version of StatMsg (must be current version)",
                        ));
                    }
                    RecordRefEnum::Stat(rec_ref.get_unchecked())
                }
                RType::Error => {
                    // can't convert V1 structs here because an immutable reference
                    if rec_ref.record_size() < std::mem::size_of::<ErrorMsg>() {
                        return Err(Error::conversion::<Self>(
                            "earlier version of ErrorMsg (must be current version)",
                        ));
                    }
                    RecordRefEnum::Error(rec_ref.get_unchecked())
                }
                RType::SymbolMapping => {
                    // can't convert V1 structs here because an immutable reference
                    if rec_ref.record_size() < std::mem::size_of::<SymbolMappingMsg>() {
                        return Err(Error::conversion::<Self>(
                            "earlier version of SymbolMappingMsg (must be current version)",
                        ));
                    }
                    RecordRefEnum::SymbolMapping(rec_ref.get_unchecked())
                }
                RType::System => {
                    // can't convert V1 structs here because an immutable reference
                    if rec_ref.record_size() < std::mem::size_of::<SystemMsg>() {
                        return Err(Error::conversion::<Self>(
                            "earlier version of SystemMsg (must be current version)",
                        ));
                    }
                    RecordRefEnum::System(rec_ref.get_unchecked())
                }
                RType::Cmbp1 | RType::Tcbbo => RecordRefEnum::Cmbp1(rec_ref.get_unchecked()),
                RType::Cbbo1S | RType::Cbbo1M => RecordRefEnum::Cbbo(rec_ref.get_unchecked()),
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
impl From<Cmbp1Msg> for RecordEnum {
    fn from(rec: Cmbp1Msg) -> Self {
        Self::Cmbp1(rec)
    }
}
impl<'a> From<&'a Cmbp1Msg> for RecordRefEnum<'a> {
    fn from(rec: &'a Cmbp1Msg) -> Self {
        Self::Cmbp1(rec)
    }
}
impl From<CbboMsg> for RecordEnum {
    fn from(rec: CbboMsg) -> Self {
        Self::Cbbo(rec)
    }
}
impl<'a> From<&'a CbboMsg> for RecordRefEnum<'a> {
    fn from(rec: &'a CbboMsg) -> Self {
        Self::Cbbo(rec)
    }
}

impl Record for RecordEnum {
    fn header(&self) -> &crate::RecordHeader {
        match self {
            RecordEnum::Mbo(rec) => rec.header(),
            RecordEnum::Trade(rec) => rec.header(),
            RecordEnum::Mbp1(rec) => rec.header(),
            RecordEnum::Mbp10(rec) => rec.header(),
            RecordEnum::Ohlcv(rec) => rec.header(),
            RecordEnum::Status(rec) => rec.header(),
            RecordEnum::InstrumentDef(rec) => rec.header(),
            RecordEnum::Imbalance(rec) => rec.header(),
            RecordEnum::Stat(rec) => rec.header(),
            RecordEnum::Error(rec) => rec.header(),
            RecordEnum::SymbolMapping(rec) => rec.header(),
            RecordEnum::System(rec) => rec.header(),
            RecordEnum::Cmbp1(rec) => rec.header(),
            RecordEnum::Bbo(rec) => rec.header(),
            RecordEnum::Cbbo(rec) => rec.header(),
        }
    }

    fn raw_index_ts(&self) -> u64 {
        match self {
            RecordEnum::Mbo(rec) => rec.raw_index_ts(),
            RecordEnum::Trade(rec) => rec.raw_index_ts(),
            RecordEnum::Mbp1(rec) => rec.raw_index_ts(),
            RecordEnum::Mbp10(rec) => rec.raw_index_ts(),
            RecordEnum::Ohlcv(rec) => rec.raw_index_ts(),
            RecordEnum::Status(rec) => rec.raw_index_ts(),
            RecordEnum::InstrumentDef(rec) => rec.raw_index_ts(),
            RecordEnum::Imbalance(rec) => rec.raw_index_ts(),
            RecordEnum::Stat(rec) => rec.raw_index_ts(),
            RecordEnum::Error(rec) => rec.raw_index_ts(),
            RecordEnum::SymbolMapping(rec) => rec.raw_index_ts(),
            RecordEnum::System(rec) => rec.raw_index_ts(),
            RecordEnum::Cmbp1(rec) => rec.raw_index_ts(),
            RecordEnum::Bbo(rec) => rec.raw_index_ts(),
            RecordEnum::Cbbo(rec) => rec.raw_index_ts(),
        }
    }
}

impl AsRef<[u8]> for RecordEnum {
    fn as_ref(&self) -> &[u8] {
        match self {
            RecordEnum::Mbo(rec) => rec.as_ref(),
            RecordEnum::Trade(rec) => rec.as_ref(),
            RecordEnum::Mbp1(rec) => rec.as_ref(),
            RecordEnum::Mbp10(rec) => rec.as_ref(),
            RecordEnum::Ohlcv(rec) => rec.as_ref(),
            RecordEnum::Status(rec) => rec.as_ref(),
            RecordEnum::InstrumentDef(rec) => rec.as_ref(),
            RecordEnum::Imbalance(rec) => rec.as_ref(),
            RecordEnum::Stat(rec) => rec.as_ref(),
            RecordEnum::Error(rec) => rec.as_ref(),
            RecordEnum::SymbolMapping(rec) => rec.as_ref(),
            RecordEnum::System(rec) => rec.as_ref(),
            RecordEnum::Cmbp1(rec) => rec.as_ref(),
            RecordEnum::Bbo(rec) => rec.as_ref(),
            RecordEnum::Cbbo(rec) => rec.as_ref(),
        }
    }
}

impl RecordMut for RecordEnum {
    fn header_mut(&mut self) -> &mut crate::RecordHeader {
        match self {
            RecordEnum::Mbo(rec) => rec.header_mut(),
            RecordEnum::Trade(rec) => rec.header_mut(),
            RecordEnum::Mbp1(rec) => rec.header_mut(),
            RecordEnum::Mbp10(rec) => rec.header_mut(),
            RecordEnum::Ohlcv(rec) => rec.header_mut(),
            RecordEnum::Status(rec) => rec.header_mut(),
            RecordEnum::InstrumentDef(rec) => rec.header_mut(),
            RecordEnum::Imbalance(rec) => rec.header_mut(),
            RecordEnum::Stat(rec) => rec.header_mut(),
            RecordEnum::Error(rec) => rec.header_mut(),
            RecordEnum::SymbolMapping(rec) => rec.header_mut(),
            RecordEnum::System(rec) => rec.header_mut(),
            RecordEnum::Cmbp1(rec) => rec.header_mut(),
            RecordEnum::Bbo(rec) => rec.header_mut(),
            RecordEnum::Cbbo(rec) => rec.header_mut(),
        }
    }
}

impl Record for RecordRefEnum<'_> {
    fn header(&self) -> &crate::RecordHeader {
        match self {
            RecordRefEnum::Mbo(rec) => rec.header(),
            RecordRefEnum::Trade(rec) => rec.header(),
            RecordRefEnum::Mbp1(rec) => rec.header(),
            RecordRefEnum::Mbp10(rec) => rec.header(),
            RecordRefEnum::Ohlcv(rec) => rec.header(),
            RecordRefEnum::Status(rec) => rec.header(),
            RecordRefEnum::InstrumentDef(rec) => rec.header(),
            RecordRefEnum::Imbalance(rec) => rec.header(),
            RecordRefEnum::Stat(rec) => rec.header(),
            RecordRefEnum::Error(rec) => rec.header(),
            RecordRefEnum::SymbolMapping(rec) => rec.header(),
            RecordRefEnum::System(rec) => rec.header(),
            RecordRefEnum::Cmbp1(rec) => rec.header(),
            RecordRefEnum::Bbo(rec) => rec.header(),
            RecordRefEnum::Cbbo(rec) => rec.header(),
        }
    }

    fn raw_index_ts(&self) -> u64 {
        match self {
            RecordRefEnum::Mbo(rec) => rec.raw_index_ts(),
            RecordRefEnum::Trade(rec) => rec.raw_index_ts(),
            RecordRefEnum::Mbp1(rec) => rec.raw_index_ts(),
            RecordRefEnum::Mbp10(rec) => rec.raw_index_ts(),
            RecordRefEnum::Ohlcv(rec) => rec.raw_index_ts(),
            RecordRefEnum::Status(rec) => rec.raw_index_ts(),
            RecordRefEnum::InstrumentDef(rec) => rec.raw_index_ts(),
            RecordRefEnum::Imbalance(rec) => rec.raw_index_ts(),
            RecordRefEnum::Stat(rec) => rec.raw_index_ts(),
            RecordRefEnum::Error(rec) => rec.raw_index_ts(),
            RecordRefEnum::SymbolMapping(rec) => rec.raw_index_ts(),
            RecordRefEnum::System(rec) => rec.raw_index_ts(),
            RecordRefEnum::Cmbp1(rec) => rec.raw_index_ts(),
            RecordRefEnum::Bbo(rec) => rec.raw_index_ts(),
            RecordRefEnum::Cbbo(rec) => rec.raw_index_ts(),
        }
    }
}

impl AsRef<[u8]> for RecordRefEnum<'_> {
    fn as_ref(&self) -> &[u8] {
        match self {
            RecordRefEnum::Mbo(rec) => rec.as_ref(),
            RecordRefEnum::Trade(rec) => rec.as_ref(),
            RecordRefEnum::Mbp1(rec) => rec.as_ref(),
            RecordRefEnum::Mbp10(rec) => rec.as_ref(),
            RecordRefEnum::Ohlcv(rec) => rec.as_ref(),
            RecordRefEnum::Status(rec) => rec.as_ref(),
            RecordRefEnum::InstrumentDef(rec) => rec.as_ref(),
            RecordRefEnum::Imbalance(rec) => rec.as_ref(),
            RecordRefEnum::Stat(rec) => rec.as_ref(),
            RecordRefEnum::Error(rec) => rec.as_ref(),
            RecordRefEnum::SymbolMapping(rec) => rec.as_ref(),
            RecordRefEnum::System(rec) => rec.as_ref(),
            RecordRefEnum::Cmbp1(rec) => rec.as_ref(),
            RecordRefEnum::Bbo(rec) => rec.as_ref(),
            RecordRefEnum::Cbbo(rec) => rec.as_ref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{record::*, v1, v2, HasRType};

    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::mbo(MboMsg::default(), None)]
    #[case::trade(TradeMsg::default(), None)]
    #[case::mbp1(Mbp1Msg::default(), None)]
    #[case::mbp10(Mbp10Msg::default(), None)]
    #[case::bbo(BboMsg::default_for_schema(crate::Schema::Bbo1S), None)]
    #[case::cmbp1(Cmbp1Msg::default_for_schema(crate::Schema::Cmbp1), None)]
    #[case::cbbo(CbboMsg::default_for_schema(crate::Schema::Cbbo1S), None)]
    #[case::ohlcv(OhlcvMsg::default_for_schema(crate::Schema::Ohlcv1S), None)]
    #[case::status(StatusMsg::default(), None)]
    #[case::imbalance(ImbalanceMsg::default(), None)]
    #[case::instrument_def_v1(
        v1::InstrumentDefMsg::default(),
        Some("couldn't convert earlier version of InstrumentDefMsg (must be current version) to dbn::record_enum::RecordRefEnum")
    )]
    #[case::instrument_def_v2(
        v2::InstrumentDefMsg::default(),
        Some("couldn't convert earlier version of InstrumentDefMsg (must be current version) to dbn::record_enum::RecordRefEnum")
    )]
    #[case::instrument_def_current(InstrumentDefMsg::default(), None)]
    #[case::symbol_mapping_v1(
        v1::SymbolMappingMsg::default(),
        Some("couldn't convert earlier version of SymbolMappingMsg (must be current version) to dbn::record_enum::RecordRefEnum")
    )]
    #[case::symbol_mapping_current(SymbolMappingMsg::default(), None)]
    #[case::system_v1(
        v1::SystemMsg::default(),
        Some("couldn't convert earlier version of SystemMsg (must be current version) to dbn::record_enum::RecordRefEnum")
    )]
    #[case::system_current(SystemMsg::default(), None)]
    #[case::error_v1(
        v1::ErrorMsg::default(),
        Some("couldn't convert earlier version of ErrorMsg (must be current version) to dbn::record_enum::RecordRefEnum")
    )]
    #[case::error_current(ErrorMsg::default(), None)]
    #[case::stat_v1(
        v1::StatMsg::default(),
        Some("couldn't convert earlier version of StatMsg (must be current version) to dbn::record_enum::RecordRefEnum")
    )]
    #[case::stat_current(StatMsg::default(), None)]
    fn test_v1_v2_safety<R: HasRType>(#[case] rec: R, #[case] exp_err: Option<&str>) {
        let rec_ref = RecordRef::from(&rec);
        let res = rec_ref.as_enum();
        dbg!(&res);
        if let Some(exp_err) = exp_err {
            assert!(format!("{}", res.unwrap_err()).contains(exp_err));
        } else {
            assert!(res.is_ok());
        }
    }
}
