//! Shared I/O utility functions for batch encoding.

use std::io::{self, IoSlice};

/// Writes all vectored slices to the writer, handling partial writes.
///
/// This is taken from the unstable standard library implementation of
/// `Write::write_all_vectored`.
pub fn write_all_vectored<W: io::Write>(
    writer: &mut W,
    mut slices: &mut [IoSlice<'_>],
) -> io::Result<()> {
    // Guarantee that bufs is empty if it contains no data,
    // to avoid calling write_vectored if there is no data to be written.
    IoSlice::advance_slices(&mut slices, 0);

    while !slices.is_empty() {
        match writer.write_vectored(slices) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "failed to write whole buffer",
                ));
            }
            Ok(n) => IoSlice::advance_slices(&mut slices, n),
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }

    Ok(())
}

/// Async version of [`write_all_vectored`].
#[cfg(feature = "async")]
pub async fn write_all_vectored_async<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
    mut slices: &mut [IoSlice<'_>],
) -> io::Result<()> {
    // Guarantee that bufs is empty if it contains no data,
    // to avoid calling write_vectored if there is no data to be written.
    IoSlice::advance_slices(&mut slices, 0);

    while !slices.is_empty() {
        match writer.write_vectored(slices).await {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "failed to write whole buffer",
                ));
            }
            Ok(n) => IoSlice::advance_slices(&mut slices, n),
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }

    Ok(())
}
