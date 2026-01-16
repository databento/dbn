//! Helper macros for working with multiple RTypes, Schemas, and types of records.

// Re-export
pub use dbn_macros::{
    dbn_record, CsvSerialize, DbnAttr, JsonSerialize, PyFieldDesc, RecordDebug, WritePyRepr,
};

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
            Ok(RType::Mbp0) => Ok($handler!(TradeMsg)),
            Ok(RType::Mbp1) => Ok($handler!(Mbp1Msg)),
            Ok(RType::Mbp10) => Ok($handler!(Mbp10Msg)),
            #[allow(deprecated)]
            Ok(RType::OhlcvDeprecated)
            | Ok(RType::Ohlcv1S)
            | Ok(RType::Ohlcv1M)
            | Ok(RType::Ohlcv1H)
            | Ok(RType::Ohlcv1D)
            | Ok(RType::OhlcvEod) => Ok($handler!(OhlcvMsg)),
            Ok(RType::Imbalance) => Ok($handler!(ImbalanceMsg)),
            Ok(RType::Status) => Ok($handler!(StatusMsg)),
            Ok(RType::InstrumentDef)
                if $rec_ref.record_size() < std::mem::size_of::<$crate::v2::InstrumentDefMsg>() =>
            {
                Ok($handler!($crate::v1::InstrumentDefMsg))
            }
            Ok(RType::InstrumentDef)
                if $rec_ref.record_size() < std::mem::size_of::<$crate::v3::InstrumentDefMsg>() =>
            {
                Ok($handler!($crate::v2::InstrumentDefMsg))
            }
            Ok(RType::InstrumentDef) => Ok($handler!($crate::v3::InstrumentDefMsg)),
            Ok(RType::SymbolMapping)
                if $rec_ref.record_size() < std::mem::size_of::<SymbolMappingMsg>() =>
            {
                Ok($handler!($crate::v1::SymbolMappingMsg))
            }
            Ok(RType::SymbolMapping) => Ok($handler!(SymbolMappingMsg)),
            Ok(RType::Error) if $rec_ref.record_size() < std::mem::size_of::<ErrorMsg>() => {
                Ok($handler!($crate::v1::ErrorMsg))
            }
            Ok(RType::Error) => Ok($handler!(ErrorMsg)),
            Ok(RType::System) if $rec_ref.record_size() < std::mem::size_of::<SystemMsg>() => {
                Ok($handler!($crate::v1::SystemMsg))
            }
            Ok(RType::System) => Ok($handler!(SystemMsg)),
            Ok(RType::Statistics)
                if $rec_ref.record_size() < std::mem::size_of::<$crate::v3::StatMsg>() =>
            {
                Ok($handler!($crate::v1::StatMsg))
            }
            Ok(RType::Statistics) => Ok($handler!($crate::v3::StatMsg)),
            Ok(RType::Mbo) => Ok($handler!(MboMsg)),
            Ok(RType::Cmbp1) | Ok(RType::Tcbbo) => Ok($handler!(Cmbp1Msg)),
            Ok(RType::Bbo1S) | Ok(RType::Bbo1M) => Ok($handler!(BboMsg)),
            Ok(RType::Cbbo1S) | Ok(RType::Cbbo1M) => Ok($handler!(CbboMsg)),
            Err(e) => Err(e),
        }
    }};
}

/// Dispatches to a generic function or method based on `$rtype` and optionally `$ts_out`.
///
/// # Panics
/// This function will panic if the encoded length of the given record is shorter
/// than expected for its `rtype`.
///
/// # Errors
/// This macro returns an error if the rtype is not recognized.
#[macro_export]
macro_rules! rtype_dispatch {
    // generic async method
    ($rec_ref:expr, $this:ident.$generic_method:ident($($arg:expr),*).await$(,)?) => {{
        macro_rules! handler {
            ($r:ty) => {{
                $this.$generic_method($rec_ref.get::<$r>().unwrap() $(, $arg)*).await
            }}
        }
        $crate::rtype_dispatch_base!($rec_ref, handler)
    }};
    // generic method
    ($rec_ref:expr, $this:ident.$generic_method:ident($($arg:expr),*)$(,)?) => {{
        macro_rules! handler {
            ($r:ty) => {{
                $this.$generic_method($rec_ref.get::<$r>().unwrap() $(, $arg)*)
            }}
        }
        $crate::rtype_dispatch_base!($rec_ref, handler)
    }};
    // generic async function
    ($rec_ref:expr, $generic_fn:ident($($arg:expr),*).await$(,)?) => {{
        macro_rules! handler {
            ($r:ty) => {{
                $generic_fn($rec_ref.get::<$r>().unwrap() $(, $arg)*).await
            }}
        }
        $crate::rtype_dispatch_base!($rec_ref, handler)
    }};
    // generic function
    ($rec_ref:expr, $generic_fn:ident($($arg:expr),*)$(,)?) => {{
        macro_rules! handler {
            ($r:ty) => {{
                $generic_fn($rec_ref.get::<$r>().unwrap() $(, $arg)*)
            }}
        }
        $crate::rtype_dispatch_base!($rec_ref, handler)
    }};
    // generic always ts_out async method
    ($rec_ref:expr, ts_out: true, $this:ident.$generic_method:ident($($arg:expr),*).await$(,)?) => {{
        macro_rules! handler {
            ($r:ty) => {{
                $this.$generic_method($rec_ref.get::<$crate::WithTsOut<$r>>().unwrap() $(, $arg)*).await
            }}
        }
        $crate::rtype_dispatch_base!($rec_ref, handler)
    }};
    // generic always ts_out method
    ($rec_ref:expr, ts_out: true, $this:ident.$generic_method:ident($($arg:expr),*)$(,)?) => {{
        macro_rules! handler {
            ($r:ty) => {{
                $this.$generic_method($rec_ref.get::<$crate::WithTsOut<$r>>().unwrap(), $(, $arg)*)
            }}
        }
        $crate::rtype_dispatch_base!($rec_ref, handler)
    }};
    // generic always ts_out async function
    ($rec_ref:expr, ts_out: true, $generic_fn:ident($($arg:expr),*).await$(,)?) => {{
        macro_rules! handler {
            ($r:ty) => {{
                $generic_fn($rec_ref.get::<$crate::WithTsOut<$r>>().unwrap() $(, $arg)*).await
            }}
        }
        $crate::rtype_dispatch_base!($rec_ref, handler)
    }};
    // generic always ts_out function
    ($rec_ref:expr, ts_out: true, $generic_fn:ident($($arg:expr),*)$(,)?) => {{
        macro_rules! handler {
            ($r:ty) => {{
                $generic_fn($rec_ref.get::<$crate::WithTsOut<$r>>().unwrap() $(, $arg)*)
            }}
        }
        $crate::rtype_dispatch_base!($rec_ref, handler)
    }};
    // ts_out generic async method
    ($rec_ref:expr, ts_out: $ts_out:expr, $this:ident.$generic_method:ident($($arg:expr),*).await$(,)?) => {{
        macro_rules! handler {
            ($r:ty) => {{
                if $ts_out {
                    $this.$generic_method($rec_ref.get::<$crate::WithTsOut<$r>>().unwrap() $(, $arg)*).await
                } else {
                    $this.$generic_method($rec_ref.get::<$r>().unwrap() $(, $arg)*).await
                }
            }}
        }
        $crate::rtype_dispatch_base!($rec_ref, handler)
    }};
    // ts_out generic method
    ($rec_ref:expr, ts_out: $ts_out:expr, $this:ident.$generic_method:ident($($arg:expr),*)$(,)?) => {{
        macro_rules! handler {
            ($r:ty) => {{
                if $ts_out {
                    $this.$generic_method($rec_ref.get::<$crate::WithTsOut<$r>>().unwrap() $(, $arg)*)
                } else {
                    $this.$generic_method($rec_ref.get::<$r>().unwrap() $(, $arg)*)
                }
            }}
        }
        $crate::rtype_dispatch_base!($rec_ref, handler)
    }};
    // ts_out generic async function
    ($rec_ref:expr, ts_out: $ts_out:expr, $generic_fn:ident($($arg:expr),*).await$(,)?) => {{
        macro_rules! handler {
            ($r:ty) => {{
                if $ts_out {
                    $generic_fn($rec_ref.get::<$crate::WithTsOut<$r>>().unwrap() $(, $arg)*).await
                } else {
                    $generic_fn($rec_ref.get::<$r>().unwrap() $(, $arg)*).await
                }
            }}
        }
        $crate::rtype_dispatch_base!($rec_ref, handler)
    }};
    // ts_out generic function
    ($rec_ref:expr, ts_out: $ts_out:expr, $generic_fn:ident($($arg:expr),*)$(,)?) => {{
        macro_rules! handler {
            ($r:ty) => {{
                if $ts_out {
                    $generic_fn($rec_ref.get::<$crate::WithTsOut<$r>>().unwrap() $(, $arg)*)
                } else {
                    $generic_fn($rec_ref.get::<$r>().unwrap() $(, $arg)*)
                }
            }}
        }
        $crate::rtype_dispatch_base!($rec_ref, handler)
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
            Schema::Cmbp1 | Schema::Tcbbo => {
                $handler!(Cmbp1Msg)
            }
            Schema::Cbbo1S | Schema::Cbbo1M => {
                $handler!(CbboMsg)
            }
        }
    }};
}

/// Dispatches to a generic function or method based on `$schema` and optionally `$ts_out`.
///
/// # Errors
/// This macro returns an error when the generic function or method returns an error.
#[macro_export]
macro_rules! schema_dispatch {
    // generic async method
    ($schema:expr, $this:ident.$generic_method:ident($($arg:expr),*).await$(,)?) => {{
        macro_rules! handler {
            ($r:ty) => {{
                $this.$generic_method::<$r>($($arg),*).await
            }}
        }
        $crate::schema_dispatch_base!($schema, handler)
    }};
    // generic method
    ($schema:expr, $this:ident.$generic_method:ident($($arg:expr),*)$(,)?) => {{
        macro_rules! handler {
            ($r:ty) => {{
                $this.$generic_method::<$r>($($arg),*)
            }}
        }
        $crate::schema_dispatch_base!($schema, handler)
    }};
    // generic async function
    ($schema:expr, $generic_fn:ident($($arg:expr),*).await$(,)?) => {{
        macro_rules! handler {
            ($r:ty) => {{
                $generic_fn::<$r>($($arg),*).await
            }}
        }
        $crate::schema_dispatch_base!($schema, handler)
    }};
    // generic function
    ($schema:expr, $generic_fn:ident($($arg:expr),*)$(,)?) => {{
        macro_rules! handler {
            ($r:ty) => {{
                $generic_fn::<$r>($($arg),*)
            }}
        }
        $crate::schema_dispatch_base!($schema, handler)
    }};
    // generic async method
    ($schema:expr, ts_out: $ts_out:expr, $this:ident.$generic_method:ident($($arg:expr),*).await$(,)?) => {{
        macro_rules! handler {
            ($r:ty) => {{
                if $ts_out {
                    $this.$generic_method::<$crate::WithTsOut<$r>>($($arg),*).await
                } else {
                    $this.$generic_method::<$r>($($arg),*).await
                }
            }}
        }
        $crate::schema_dispatch_base!($schema, handler)
   }};
    // generic method
    ($schema:expr, ts_out: $ts_out:expr, $this:ident.$generic_method:ident($($arg:expr),*)$(,)?) => {{
        macro_rules! handler {
            ($r:ty) => {{
                if $ts_out {
                    $this.$generic_method::<$crate::WithTsOut<$r>>($($arg),*)
                } else {
                    $this.$generic_method::<$r>($($arg),*)
                }
            }}
        }
        $crate::schema_dispatch_base!($schema, handler)
    }};
    // generic async function
    ($schema:expr, ts_out: $ts_out:expr, $generic_fn:ident($($arg:expr),*).await$(,)?) => {{
        macro_rules! handler {
            ($r:ty) => {{
                if $ts_out {
                    $generic_fn::<$crate::WithTsOut<$r>>($($arg),*).await
                } else {
                    $generic_fn::<$r>($($arg),*).await
                }
            }}
        }
        $crate::schema_dispatch_base!($schema, handler)
    }};
    // generic function
    ($schema:expr, ts_out: $ts_out:expr, $generic_fn:ident($($arg:expr),*)$(,)?) => {{
        macro_rules! handler {
            ($r:ty) => {{
                if $ts_out {
                    $generic_fn::<$crate::WithTsOut<$r>>($($arg),*)
                } else {
                    $generic_fn::<$r>($($arg),*)
                }
            }}
        }
        $crate::schema_dispatch_base!($schema, handler)
    }};
}

// DEPRECATED macros

/// Specializes a generic function to all record types and dispatches based on the
/// `rtype` and `ts_out`.
///
/// # Safety
/// Assumes `$rec_ref` contains a record with `ts_out` appended. If this is not the
/// case, the reading the record will read beyond the end of the record.
///
/// # Errors
/// This macro returns an error if the rtype is not recognized.
#[deprecated(since = "0.32.0", note = "Use the updated rtype_dispatch macro")]
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
#[deprecated(since = "0.32.0", note = "Use the updated rtype_dispatch macro")]
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
#[deprecated(since = "0.32.0", note = "Use the updated rtype_dispatch macro")]
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
#[deprecated(since = "0.32.0", note = "Use the updated rtype_dispatch macro")]
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
/// Specializes a generic method to all record types and dispatches based `rtype`.
///
/// # Errors
/// This macro returns an error if the rtype is not recognized.
#[deprecated(since = "0.32.0", note = "Use the updated rtype_dispatch macro")]
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
#[deprecated(since = "0.32.0", note = "Use the updated rtype_dispatch macro")]
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
#[deprecated(since = "0.32.0", note = "Use the updated rtype_dispatch macro")]
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
#[deprecated(since = "0.32.0", note = "Use the updated schema_dispatch macro")]
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
#[deprecated(since = "0.32.0", note = "Use the updated schema_dispatch macro")]
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
#[deprecated(since = "0.32.0", note = "Use the updated schema_dispatch macro")]
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

#[cfg(test)]
mod tests {
    use crate::{record::HasRType, rtype};

    struct Dummy {}

    #[allow(dead_code)]
    impl Dummy {
        fn on_rtype<T: HasRType>(&self) -> bool {
            T::has_rtype(0xFF)
        }

        #[allow(clippy::extra_unused_type_parameters)]
        fn on_rtype_2<T: HasRType>(&self, x: u64, y: u64) -> u64 {
            x + y
        }

        async fn do_something<T: HasRType>(&self, arg: u8) -> bool {
            T::has_rtype(arg)
        }
    }

    fn has_rtype<T: HasRType>(arg: u8) -> bool {
        T::has_rtype(arg)
    }

    #[test]
    fn test_two_args() {
        assert!(schema_dispatch!(
            Schema::Imbalance,
            has_rtype(rtype::IMBALANCE)
        ))
    }

    #[test]
    fn test_method_two_args() {
        let dummy = Dummy {};
        let ret = schema_dispatch!(Schema::Definition, dummy.on_rtype_2(5, 6));
        assert_eq!(ret, 11);
    }

    #[test]
    fn test_method_no_args() {
        let dummy = Dummy {};
        let ret = schema_dispatch!(Schema::Definition, dummy.on_rtype());
        assert!(!ret);
    }

    #[cfg(feature = "async")]
    mod r#async {
        use super::*;

        #[tokio::test]
        async fn test_self() {
            let target = Dummy {};
            let ret_true = schema_dispatch!(
                Schema::Trades,
                target.do_something(crate::enums::rtype::MBP_0).await,
            );
            let ret_false = schema_dispatch!(Schema::Trades, target.do_something(0xff).await);
            assert!(ret_true);
            assert!(!ret_false);
        }
    }
}
