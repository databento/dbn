use super::*;

/// Provides a _relatively safe_ method for converting a reference to
/// [`RecordHeader`] to a struct beginning with the header. Because it accepts a
/// reference, the lifetime of the returned reference is tied to the input. This
/// function checks `rtype` before casting to ensure `bytes` contains a `T`.
///
/// # Safety
/// `raw` must contain at least `std::mem::size_of::<T>()` bytes and a valid
/// [`RecordHeader`] instance.
pub unsafe fn transmute_record_bytes<T: HasRType>(bytes: &[u8]) -> Option<&T> {
    assert!(
        bytes.len() >= mem::size_of::<T>(),
        "Passing a slice smaller than `{}` to `transmute_record_bytes` is invalid",
        std::any::type_name::<T>()
    );
    let non_null = NonNull::new_unchecked(bytes.as_ptr().cast_mut());
    if T::has_rtype(non_null.cast::<RecordHeader>().as_ref().rtype) {
        Some(non_null.cast::<T>().as_ref())
    } else {
        None
    }
}

/// Provides a _relatively safe_ method for converting a view on bytes into a
/// a [`RecordHeader`].
/// Because it accepts a reference, the lifetime of the returned reference is
/// tied to the input.
///
/// # Safety
/// `bytes` must contain a complete record (not only the header). This is so that
/// the header can be subsequently passed to `transmute_record`.
///
/// # Panics
/// This function will panic if `bytes` is shorter the length of [`RecordHeader`], the
/// minimum length a record can have.
pub unsafe fn transmute_header_bytes(bytes: &[u8]) -> Option<&RecordHeader> {
    assert!(
        bytes.len() >= mem::size_of::<RecordHeader>(),
        concat!(
            "Passing a slice smaller than `",
            stringify!(RecordHeader),
            "` to `transmute_header_bytes` is invalid"
        )
    );
    let non_null = NonNull::new_unchecked(bytes.as_ptr().cast_mut());
    let header = non_null.cast::<RecordHeader>().as_ref();
    if header.record_size() > bytes.len() {
        None
    } else {
        Some(header)
    }
}

/// Provides a _relatively safe_ method for converting a reference to a
/// [`RecordHeader`] to a struct beginning with the header. Because it accepts a reference,
/// the lifetime of the returned reference is tied to the input.
///
/// # Safety
/// Although this function accepts a reference to a [`RecordHeader`], it's assumed this is
/// part of a larger `T` struct.
pub unsafe fn transmute_record<T: HasRType>(header: &RecordHeader) -> Option<&T> {
    if T::has_rtype(header.rtype) {
        // Safety: because it comes from a reference, `header` must not be null. It's ok
        // to cast to `mut` because it's never mutated.
        let non_null = NonNull::from(header);
        Some(non_null.cast::<T>().as_ref())
    } else {
        None
    }
}

/// Aliases `data` as a slice of raw bytes.
///
/// # Safety
/// `data` must be sized and plain old data (POD), i.e. no pointers.
pub(crate) unsafe fn as_u8_slice<T: Sized>(data: &T) -> &[u8] {
    slice::from_raw_parts((data as *const T).cast(), mem::size_of::<T>())
}

/// Provides a _relatively safe_ method for converting a mut reference to a
/// [`RecordHeader`] to a struct beginning with the header. Because it accepts a reference,
/// the lifetime of the returned reference is tied to the input.
///
/// # Safety
/// Although this function accepts a reference to a [`RecordHeader`], it's assumed this is
/// part of a larger `T` struct.
pub unsafe fn transmute_record_mut<T: HasRType>(header: &mut RecordHeader) -> Option<&mut T> {
    if T::has_rtype(header.rtype) {
        // Safety: because it comes from a reference, `header` must not be null.
        let non_null = NonNull::from(header);
        Some(non_null.cast::<T>().as_mut())
    } else {
        None
    }
}

/// Tries to convert a str slice to fixed-length null-terminated C char array.
///
/// # Errors
/// This function returns an error if `s` contains more than N - 1 characters. The last
/// character is reserved for the null byte.
pub fn str_to_c_chars<const N: usize>(s: &str) -> Result<[c_char; N]> {
    if s.len() > (N - 1) {
        return Err(Error::encode(format!(
            "string cannot be longer than {}; received str of length {}",
            N - 1,
            s.len(),
        )));
    }
    let mut res = [0; N];
    for (i, byte) in s.as_bytes().iter().enumerate() {
        res[i] = *byte as c_char;
    }
    Ok(res)
}

/// Tries to convert a slice of `c_char`s to a UTF-8 `str`.
///
/// # Safety
/// This should always be safe.
///
/// # Preconditions
/// None.
///
/// # Errors
/// This function returns an error if `chars` contains invalid UTF-8 or is not null-terminated.
pub fn c_chars_to_str<const N: usize>(chars: &[c_char; N]) -> Result<&str> {
    // Safety: Casting from i8 to u8 slice should be safe
    let bytes = unsafe { as_u8_slice(chars) };
    let cstr = CStr::from_bytes_until_nul(bytes).map_err(|_| Error::Conversion {
        input: format!("{chars:?}"),
        desired_type: "CStr (null-terminated)",
    })?;

    cstr.to_str()
        .map_err(|e| Error::utf8(e, format!("converting c_char array: {chars:?}")))
}

/// Parses a raw nanosecond-precision UNIX timestamp to an `OffsetDateTime`. Returns
/// `None` if `ts` contains the sentinel for a null timestamp.
pub fn ts_to_dt(ts: u64) -> Option<time::OffsetDateTime> {
    if ts == crate::UNDEF_TIMESTAMP {
        None
    } else {
        // u64::MAX is within maximum allowable range
        Some(time::OffsetDateTime::from_unix_timestamp_nanos(ts as i128).unwrap())
    }
}

#[cfg(feature = "serde")]
pub(crate) mod cstr_serde {
    use std::ffi::c_char;

    use serde::{de, ser, Deserialize, Deserializer, Serializer};

    use super::{c_chars_to_str, str_to_c_chars};

    pub fn serialize<S, const N: usize>(
        chars: &[c_char; N],
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(c_chars_to_str(chars).map_err(ser::Error::custom)?)
    }

    pub fn deserialize<'de, D, const N: usize>(deserializer: D) -> Result<[c_char; N], D::Error>
    where
        D: Deserializer<'de>,
    {
        let str = String::deserialize(deserializer)?;
        str_to_c_chars(&str).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::raw::c_char;

    #[test]
    fn test_c_chars_to_str_success() {
        let null_terminated: [c_char; 5] = [
            'A' as c_char,
            'A' as c_char,
            'A' as c_char,
            'A' as c_char,
            0,
        ];
        let result = c_chars_to_str(&null_terminated);
        assert_eq!(result.unwrap(), "AAAA");
    }

    #[test]
    fn test_c_chars_to_str_failure_on_missing_null_terminator() {
        let non_null_terminated: [c_char; 5] = ['A' as c_char; 5];
        let err = c_chars_to_str(&non_null_terminated)
            .expect_err("Expected failure on non-null terminated string");

        assert!(matches!(
            err,
            Error::Conversion {
                input: _,
                desired_type: "CStr (null-terminated)",
            }
        ));
    }
}
