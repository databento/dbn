use crate::{HasRType, SecurityUpdateAction, StatType, StatUpdateAction};

/// A trait for compatibility between different versions of symbol mapping records.
pub trait SymbolMappingRec: HasRType {
    /// Returns the input symbol as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `stype_in_symbol` contains invalid UTF-8.
    fn stype_in_symbol(&self) -> crate::Result<&str>;

    /// Returns the output symbol as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `stype_out_symbol` contains invalid UTF-8.
    fn stype_out_symbol(&self) -> crate::Result<&str>;

    /// Parses the raw start of the mapping interval into a datetime. Returns `None` if
    /// `start_ts` contains the sentinel for a null timestamp.
    fn start_ts(&self) -> Option<time::OffsetDateTime>;

    /// Parses the raw end of the mapping interval into a datetime. Returns `None` if
    /// `end_ts` contains the sentinel for a null timestamp.
    fn end_ts(&self) -> Option<time::OffsetDateTime>;
}

/// A trait for compatibility between different versions of definition records.
pub trait InstrumentDefRec: HasRType {
    /// Returns the instrument raw symbol assigned by the publisher as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `raw_symbol` contains invalid UTF-8.
    fn raw_symbol(&self) -> crate::Result<&str>;

    /// Returns the underlying asset code (product code) of the instrument as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `asset` contains invalid UTF-8.
    fn asset(&self) -> crate::Result<&str>;

    /// Returns the [Security type](https://databento.com/docs/schemas-and-data-formats/instrument-definitions#security-type)
    /// of the instrument, e.g. FUT for future or future spread as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `security_type` contains invalid UTF-8.
    fn security_type(&self) -> crate::Result<&str>;

    /// Returns the action indicating whether the instrument definition has been added,
    /// modified, or deleted.
    ///
    /// # Errors
    /// This function returns an error if the `security_update_action` field does not
    /// contain a valid [`SecurityUpdateAction`].
    fn security_update_action(&self) -> crate::Result<SecurityUpdateAction>;

    /// The channel ID assigned by Databento as an incrementing integer starting at
    /// zero.
    fn channel_id(&self) -> u16;
}

/// A trait for compatibility between different versions of statistics records.
pub trait StatRec: HasRType {
    /// The sentinel value for a null `quantity`.
    const UNDEF_STAT_QUANTITY: i64;

    /// Tries to convert the raw type of the statistic value to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `stat_type` field does not
    /// contain a valid [`StatType`].
    fn stat_type(&self) -> crate::Result<StatType>;

    /// Parses the raw capture-server-received timestamp into a datetime. Returns `None`
    /// if `ts_recv` contains the sentinel for a null timestamp.
    fn ts_recv(&self) -> Option<time::OffsetDateTime>;

    /// Parses the raw reference timestamp of the statistic value into a datetime.
    /// Returns `None` if `ts_ref` contains the sentinel for a null timestamp.
    fn ts_ref(&self) -> Option<time::OffsetDateTime>;

    /// Tries to convert the raw `update_action` to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `update_action` field does not
    /// contain a valid [`StatUpdateAction`].
    fn update_action(&self) -> crate::Result<StatUpdateAction>;

    /// The value for price statistics expressed as a signed integer where every 1 unit
    /// corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001. Will be
    /// [`UNDEF_PRICE`](crate::UNDEF_PRICE) when unused.
    fn price(&self) -> i64;

    /// The value for quantity statistics. Will be `UNDEF_STAT_QUANTITY` when unused.
    fn quantity(&self) -> i64;
}
