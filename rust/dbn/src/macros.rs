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
                | RType::Ohlcv1D => $handler!(OhlcvMsg),
                RType::Imbalance => $handler!(ImbalanceMsg),
                RType::Status => $handler!(StatusMsg),
                RType::InstrumentDef => $handler!(InstrumentDefMsg),
                RType::Error => $handler!(ErrorMsg),
                RType::SymbolMapping => $handler!(SymbolMappingMsg),
                RType::System => $handler!(SystemMsg),
                RType::Mbo => $handler!(MboMsg),
            }),
            Err(e) => Err(e),
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
