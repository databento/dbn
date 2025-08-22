use crate::{InstrumentClass, RType, Schema, TriState, VersionUpgradePolicy};

impl InstrumentClass {
    /// Returns `true` if the instrument class is a type of option.
    ///
    /// Note: excludes [`Self::MixedSpread`], which *may* include options.
    pub fn is_option(&self) -> bool {
        matches!(self, Self::Call | Self::Put | Self::OptionSpread)
    }

    /// Returns `true` if the instrument class is a type of future.
    ///
    /// Note: excludes [`Self::MixedSpread`], which *may* include futures.
    pub fn is_future(&self) -> bool {
        matches!(self, Self::Future | Self::FutureSpread)
    }

    /// Returns `true` if the instrument class is a type of spread, i.e. composed of two
    /// or more instrument legs.
    pub fn is_spread(&self) -> bool {
        matches!(
            self,
            Self::FutureSpread | Self::OptionSpread | Self::MixedSpread
        )
    }
}

/// Get the corresponding `rtype` for the given `schema`.
impl From<Schema> for RType {
    fn from(schema: Schema) -> Self {
        match schema {
            Schema::Mbo => RType::Mbo,
            Schema::Mbp1 | Schema::Tbbo => RType::Mbp1,
            Schema::Mbp10 => RType::Mbp10,
            Schema::Trades => RType::Mbp0,
            Schema::Ohlcv1S => RType::Ohlcv1S,
            Schema::Ohlcv1M => RType::Ohlcv1M,
            Schema::Ohlcv1H => RType::Ohlcv1H,
            Schema::Ohlcv1D => RType::Ohlcv1D,
            Schema::OhlcvEod => RType::OhlcvEod,
            Schema::Definition => RType::InstrumentDef,
            Schema::Statistics => RType::Statistics,
            Schema::Status => RType::Status,
            Schema::Imbalance => RType::Imbalance,
            Schema::Cmbp1 => RType::Cmbp1,
            Schema::Cbbo1S => RType::Cbbo1S,
            Schema::Cbbo1M => RType::Cbbo1M,
            Schema::Tcbbo => RType::Tcbbo,
            Schema::Bbo1S => RType::Bbo1S,
            Schema::Bbo1M => RType::Bbo1M,
        }
    }
}

impl RType {
    /// Tries to convert the given rtype to a [`Schema`].
    ///
    /// Returns `None` if there's no corresponding `Schema` for the given rtype or
    /// in the case of `OHLCV_DEPRECATED`, it doesn't map to a single `Schema`.
    pub fn try_into_schema(rtype: u8) -> Option<Schema> {
        use crate::enums::rtype::*;
        match rtype {
            MBP_0 => Some(Schema::Trades),
            MBP_1 => Some(Schema::Mbp1),
            MBP_10 => Some(Schema::Mbp10),
            OHLCV_1S => Some(Schema::Ohlcv1S),
            OHLCV_1M => Some(Schema::Ohlcv1M),
            OHLCV_1H => Some(Schema::Ohlcv1H),
            OHLCV_1D => Some(Schema::Ohlcv1D),
            OHLCV_EOD => Some(Schema::OhlcvEod),
            STATUS => Some(Schema::Status),
            INSTRUMENT_DEF => Some(Schema::Definition),
            IMBALANCE => Some(Schema::Imbalance),
            STATISTICS => Some(Schema::Statistics),
            MBO => Some(Schema::Mbo),
            CMBP_1 => Some(Schema::Cmbp1),
            CBBO_1S => Some(Schema::Cbbo1S),
            CBBO_1M => Some(Schema::Cbbo1M),
            TCBBO => Some(Schema::Tcbbo),
            BBO_1S => Some(Schema::Bbo1S),
            BBO_1M => Some(Schema::Bbo1M),
            _ => None,
        }
    }

    /// Returns the interval associated with the `RType` if it's a subsampled
    /// record type, otherwise `None`.
    pub const fn interval(self) -> Option<time::Duration> {
        match self {
            RType::Ohlcv1S | RType::Cbbo1S | RType::Bbo1S => Some(time::Duration::SECOND),
            RType::Ohlcv1M | RType::Cbbo1M | RType::Bbo1M => Some(time::Duration::MINUTE),
            RType::Ohlcv1H => Some(time::Duration::HOUR),
            RType::Ohlcv1D | RType::OhlcvEod => Some(time::Duration::DAY),
            _ => None,
        }
    }
}

impl Schema {
    /// Returns the interval associated with the `Schema` if it's a subsampled
    /// schema, otherwise `None`.
    pub fn interval(self) -> Option<time::Duration> {
        RType::from(self).interval()
    }
}

impl From<TriState> for Option<bool> {
    fn from(value: TriState) -> Self {
        match value {
            TriState::NotAvailable => None,
            TriState::No => Some(false),
            TriState::Yes => Some(true),
        }
    }
}

impl From<Option<bool>> for TriState {
    fn from(value: Option<bool>) -> Self {
        match value {
            Some(true) => Self::Yes,
            Some(false) => Self::No,
            None => Self::NotAvailable,
        }
    }
}

impl VersionUpgradePolicy {
    /// Validates a given DBN `version` is compatible with the upgrade policy.
    ///
    /// # Errors
    /// This function returns an error if the version and upgrade policy are
    /// incompatible.
    pub fn validate_compatibility(self, version: u8) -> crate::Result<()> {
        if version > 2 && self == Self::UpgradeToV2 {
            Err(crate::Error::decode("Invalid combination of `VersionUpgradePolicy::UpgradeToV2` and input version 3. Choose either `AsIs` and `UpgradeToV3` as an upgrade policy"))
        } else {
            Ok(())
        }
    }

    pub(crate) fn is_upgrade_situation(self, version: u8) -> bool {
        match (self, version) {
            (Self::AsIs, _) => false,
            (Self::UpgradeToV2, v) if v < 2 => true,
            (Self::UpgradeToV2, _) => false,
            (Self::UpgradeToV3, v) if v < 3 => true,
            (Self::UpgradeToV3, _) => false,
        }
    }

    /// Returns the output DBN version given the input version and upgrade policy.
    pub fn output_version(self, input_version: u8) -> u8 {
        match self {
            Self::AsIs => input_version,
            Self::UpgradeToV2 => 2,
            Self::UpgradeToV3 => 3,
        }
    }
}
