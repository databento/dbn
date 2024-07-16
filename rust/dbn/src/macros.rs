//! Helper macros for working with multiple RTypes, Schemas, and types of records.

// Re-export
pub use dbn_macros::{dbn_record, CsvSerialize, DbnAttr, JsonSerialize, PyFieldDesc, RecordDebug};

/// Base macro for type dispatch based on rtype.
///
/// # Errors
/// This macro returns an error if the rtype is not recognized.
#[doc(hidden)]
#[macro_export]
macro_rules! rtype_dispatch_base {
    ($rec_ref:expr, $handler:ident) => {{
        // Introduced new scope so new `use`s are ok
        use $crate::enums::RType;
        use $crate::record::*;
        match $rec_ref.rtype() {
            Ok(rtype) => Ok(match rtype {
                RType::Mbp0 => $handler!(TradeMsg),
                RType::Mbp1 => $handler!(Mbp1Msg),
                RType::Mbp10 => $handler!(Mbp10Msg),
                #[allow(deprecated)]
                RType::OhlcvDeprecated
                | RType::Ohlcv1S
                | RType::Ohlcv1M
                | RType::Ohlcv1H
                | RType::Ohlcv1D
                | RType::OhlcvEod => $handler!(OhlcvMsg),
                RType::Imbalance => $handler!(ImbalanceMsg),
                RType::Status => $handler!(StatusMsg),
                RType::InstrumentDef => {
                    // TODO(carter): remove temporary version handling
                    if $rec_ref.record_size() < std::mem::size_of::<InstrumentDefMsg>() {
                        $handler!($crate::compat::InstrumentDefMsgV1)
                    } else {
                        $handler!(InstrumentDefMsg)
                    }
                }
                RType::SymbolMapping => {
                    // TODO(carter): remove temporary version handling
                    if $rec_ref.record_size() < std::mem::size_of::<SymbolMappingMsg>() {
                        $handler!($crate::compat::SymbolMappingMsgV1)
                    } else {
                        $handler!(SymbolMappingMsg)
                    }
                }
                RType::Error => {
                    if $rec_ref.record_size() < std::mem::size_of::<ErrorMsg>() {
                        $handler!($crate::compat::ErrorMsgV1)
                    } else {
                        $handler!(ErrorMsg)
                    }
                }
                RType::System => {
                    if $rec_ref.record_size() < std::mem::size_of::<SystemMsg>() {
                        $handler!($crate::compat::SystemMsgV1)
                    } else {
                        $handler!(SystemMsg)
                    }
                }
                RType::Statistics => $handler!(StatMsg),
                RType::Mbo => $handler!(MboMsg),
                RType::Cbbo | RType::Cbbo1S | RType::Cbbo1M | RType::Tcbbo => $handler!(CbboMsg),
                RType::Bbo1S | RType::Bbo1M => $handler!(BboMsg),
            }),
            Err(e) => Err(e),
        }
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! schema_dispatch_base {
    ($schema:expr, $handler:ident) => {{
        use $crate::enums::Schema;
        use $crate::record::*;
        match $schema {
            Schema::Mbo => $handler!(MboMsg),
            Schema::Mbp1 | Schema::Tbbo => $handler!(Mbp1Msg),
            Schema::Bbo1S | Schema::Bbo1M => $handler!(BboMsg),
            Schema::Mbp10 => $handler!(Mbp10Msg),
            Schema::Trades => $handler!(TradeMsg),
            Schema::Ohlcv1D
            | Schema::Ohlcv1H
            | Schema::Ohlcv1M
            | Schema::Ohlcv1S
            | Schema::OhlcvEod => {
                $handler!(OhlcvMsg)
            }
            Schema::Definition => $handler!(InstrumentDefMsg),
            Schema::Statistics => $handler!(StatMsg),
            Schema::Status => $handler!(StatusMsg),
            Schema::Imbalance => $handler!(ImbalanceMsg),
            Schema::Cbbo | Schema::Cbbo1S | Schema::Cbbo1M | Schema::Tcbbo => {
                $handler!(CbboMsg)
            }
        }
    }};
}

/// Specializes a generic function to all record types and dispatches based on the
/// `rtype` and `ts_out`.
///
/// # Safety
/// Assumes `$rec_ref` contains a record with `ts_out` appended. If this is not the
/// case, the reading the record will read beyond the end of the record.
///
/// # Errors
/// This macro returns an error if the rtype is not recognized.
#[macro_export]
macro_rules! rtype_ts_out_dispatch {
    ($rec_ref:expr, $ts_out:expr, $generic_fn:expr $(,$arg:expr)*) => {{
        macro_rules! maybe_ts_out {
            ($r:ty) => {{
                if $ts_out {
                    $generic_fn($rec_ref.get_unchecked::<WithTsOut<$r>>() $(, $arg)*)
                } else {
                    $generic_fn(unsafe { $rec_ref.get_unchecked::<$r>() } $(, $arg)*)
                }
            }};
        }
        $crate::rtype_dispatch_base!($rec_ref, maybe_ts_out)
    }};
}

/// Specializes a generic method to all record types and dispatches based on the
/// `rtype` and `ts_out`.
///
/// # Safety
/// Assumes `$rec_ref` contains a record with `ts_out` appended. If this is not the
/// case, the reading the record will read beyond the end of the record.
///
/// # Errors
/// This macro returns an error if the rtype is not recognized.
#[macro_export]
macro_rules! rtype_ts_out_method_dispatch {
    ($rec_ref:expr, $ts_out:expr, $this:expr, $generic_method:ident $(,$arg:expr)*) => {{
        macro_rules! maybe_ts_out {
            ($r:ty) => {{
                if $ts_out {
                    $this.$generic_method($rec_ref.get_unchecked::<WithTsOut<$r>>() $(, $arg)*)
                } else {
                    $this.$generic_method(unsafe { $rec_ref.get_unchecked::<$r>() } $(, $arg)*)
                }
            }};
        }
        $crate::rtype_dispatch_base!($rec_ref, maybe_ts_out)
    }};
}

/// Specializes a generic async function to all record types and dispatches based
/// `rtype` and `ts_out`.
///
/// # Errors
/// This macro returns an error if the rtype is not recognized.
#[macro_export]
macro_rules! rtype_ts_out_async_dispatch {
    ($rec_ref:expr, $ts_out:expr, $generic_fn:expr $(,$arg:expr)*) => {{
        macro_rules! maybe_ts_out {
            ($r:ty) => {{
                if $ts_out {
                    $generic_fn($rec_ref.get_unchecked::<WithTsOut<$r>>() $(, $arg)*).await
                } else {
                    $generic_fn(unsafe { $rec_ref.get_unchecked::<$r>() } $(, $arg)*).await
                }
            }};
        }
        $crate::rtype_dispatch_base!($rec_ref, maybe_ts_out)
    }};
}

/// Specializes a generic async method to all record types and dispatches based
/// `rtype` and `ts_out`.
///
/// # Errors
/// This macro returns an error if the rtype is not recognized.
#[macro_export]
macro_rules! rtype_ts_out_async_method_dispatch {
    ($rec_ref:expr, $ts_out:expr, $this:expr, $generic_method:ident $(,$arg:expr)*) => {{
        macro_rules! maybe_ts_out {
            ($r:ty) => {{
                if $ts_out {
                    $this.$generic_method($rec_ref.get_unchecked::<WithTsOut<$r>>() $(, $arg)*).await
                } else {
                    $this.$generic_method(unsafe { $rec_ref.get_unchecked::<$r>() } $(, $arg)*).await
                }
            }};
        }
        $crate::rtype_dispatch_base!($rec_ref, maybe_ts_out)
    }};
}

/// Specializes a generic function to all record types and dispatches based `rtype`.
///
/// # Errors
/// This macro returns an error if the rtype is not recognized.
#[macro_export]
macro_rules! rtype_dispatch {
    ($rec_ref:expr, $generic_fn:expr $(,$arg:expr)*) => {{
        macro_rules! handler {
            ($r:ty) => {{
                // Safety: checks rtype before converting.
                $generic_fn( unsafe { $rec_ref.get_unchecked::<$r>() } $(, $arg)*)
            }}
        }
        $crate::rtype_dispatch_base!($rec_ref, handler)
    }};
}

/// Specializes a generic method to all record types and dispatches based `rtype`.
///
/// # Errors
/// This macro returns an error if the rtype is not recognized.
#[macro_export]
macro_rules! rtype_method_dispatch {
    ($rec_ref:expr, $this:expr, $generic_method:ident $(,$arg:expr)*) => {{
        macro_rules! handler {
            ($r:ty) => {{
                // Safety: checks rtype before converting.
                $this.$generic_method( unsafe { $rec_ref.get_unchecked::<$r>() } $(, $arg)*)
            }}
        }
        $crate::rtype_dispatch_base!($rec_ref, handler)
    }};
}

/// Specializes a generic async function to all record types and dispatches based
/// `rtype`.
///
/// # Errors
/// This macro returns an error if the rtype is not recognized.
#[macro_export]
macro_rules! rtype_async_dispatch {
    ($rec_ref:expr, $generic_fn:expr $(,$arg:expr)*) => {{
        macro_rules! handler {
            ($r:ty) => {{
                // Safety: checks rtype before converting.
                $generic_fn( unsafe { $rec_ref.get_unchecked::<$r>() } $(, $arg)*).await
            }}
        }
        $crate::rtype_dispatch_base!($rec_ref, handler)
    }};
}

/// Specializes a generic function to all record types wrapped in
/// [`WithTsOut`](crate::record::WithTsOut) and dispatches based on the `rtype`.
///
/// # Safety
/// Assumes `$rec_ref` contains a record with `ts_out` appended. If this is not the
/// case, the reading the record will read beyond the end of the record.
///
/// # Errors
/// This macro returns an error if the rtype is not recognized.
#[macro_export]
macro_rules! rtype_dispatch_with_ts_out {
    ($rec_ref:expr, $generic_fn:expr $(,$arg:expr)*) => {{
        macro_rules! handler {
            ($r:ty) => {{
                $generic_fn( $rec_ref.get_unchecked::<WithTsOut<$r>>() $(, $arg)*)
            }}
        }
        $crate::rtype_dispatch_base!($rec_ref, handler)
    };
}}

/// Specializes a generic method to all record types with an associated schema.
#[macro_export]
macro_rules! schema_method_dispatch {
    ($schema:expr, $this:expr, $generic_method:ident $(,$arg:expr)*) => {{
        macro_rules! handler {
            ($r:ty) => {{
                $this.$generic_method::<$r>($($arg),*)
            }}
        }
        $crate::schema_dispatch_base!($schema, handler)
    }};
}

/// Specializes a generic method to all record types based on the associated type for
/// `schema` and `ts_out`.
#[macro_export]
macro_rules! schema_ts_out_method_dispatch {
    ($schema:expr, $ts_out:expr, $this:expr, $generic_method:ident $(,$arg:expr)*) => {{
        macro_rules! handler {
            ($r:ty) => {{
                if $ts_out {
                    $this.$generic_method::<WithTsOut<$r>>($($arg),*)
                } else {
                    $this.$generic_method::<$r>($($arg),*)
                }
            }}
        }
        $crate::schema_dispatch_base!($schema, handler)
    }};
}

/// Specializes a generic async method to all record types with an associated
/// schema.
#[macro_export]
macro_rules! schema_async_method_dispatch {
    ($schema:expr, $this:expr, $generic_method:ident $(,$arg:expr)*) => {{
        macro_rules! handler {
            ($r:ty) => {{
                $this.$generic_method::<$r>($($arg),*).await
            }}
        }
        $crate::schema_dispatch_base!($schema, handler)
    }};
}

/// Specializes a generic function to all record types with an associated schema.
#[macro_export]
macro_rules! schema_dispatch {
    ($schema:expr, $generic_fn:ident $(,$arg:expr)*) => {{
        macro_rules! handler {
            ($r:ty) => {{
                $generic_fn::<$r>($($arg),*)
            }}
        }
        $crate::schema_dispatch_base!($schema, handler)
    }};
}

#[cfg(test)]
mod tests {
    use crate::{record::HasRType, schema_method_dispatch};

    struct Dummy {}

    #[allow(dead_code)]
    impl Dummy {
        fn on_rtype<T: HasRType>(&self) -> bool {
            T::has_rtype(0xFF)
        }

        fn on_rtype_2<T: HasRType>(&self, x: u64, y: u64) -> u64 {
            x + y
        }

        async fn do_something<T: HasRType>(&self, arg: u8) -> bool {
            T::has_rtype(arg)
        }
    }

    #[test]
    fn test_two_args() {
        let ret = schema_method_dispatch!(Schema::Definition, Dummy {}, on_rtype_2, 5, 6);
        assert_eq!(ret, 11);
    }

    #[test]
    fn test_no_args() {
        let ret = schema_method_dispatch!(Schema::Definition, Dummy {}, on_rtype);
        assert!(!ret);
    }

    #[cfg(feature = "async")]
    mod r#async {
        use super::*;
        use crate::schema_async_method_dispatch;

        #[tokio::test]
        async fn test_self() {
            let target = Dummy {};
            let ret_true = schema_async_method_dispatch!(
                Schema::Trades,
                target,
                do_something,
                crate::enums::rtype::MBP_0
            );
            let ret_false =
                schema_async_method_dispatch!(Schema::Trades, target, do_something, 0xff);
            assert!(ret_true);
            assert!(!ret_false);
        }
    }
}
