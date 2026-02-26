//! Aligned buffer for DBN decoding.
//!
//! Forked from [oval](https://github.com/fasterthanlime/oval)
//! (MIT license) with the following changes:
//! - uses `Box[u64]` for backing storage to guarantee 8-byte alignment
//! - use boxed slice instead of `Vec`
//! - mutable getter for readable data

use std::cmp;

/// A byte buffer backed by `Box<[u64]>` to guarantee 8-byte alignment.
///
/// Invariants: `0 <= position <= end <= byte_capacity`
#[derive(Debug, Clone)]
pub struct AlignedBuffer {
    /// The backing storage. Aligned to 8 bytes.
    memory: Box<[u64]>,
    /// Current beginning of available data (byte offset).
    position: usize,
    /// Current end of available data (byte offset).
    end: usize,
}

impl AlignedBuffer {
    /// Allocates a new buffer with at least `capacity` usable bytes.
    pub fn with_capacity(capacity: usize) -> Self {
        let u64_len = capacity.div_ceil(8);
        let memory = vec![0; u64_len].into_boxed_slice();
        Self {
            memory,
            position: 0,
            end: 0,
        }
    }

    fn as_byte_slice(&self) -> &[u8] {
        // Safety: `Box<[u64]>` is valid for reads as `[u8]`.
        unsafe { std::slice::from_raw_parts(self.memory.as_ptr().cast::<u8>(), self.capacity()) }
    }

    fn as_byte_slice_mut(&mut self) -> &mut [u8] {
        // Safety: `Box<[u64]>` is valid for writes as `[u8]`.
        unsafe {
            std::slice::from_raw_parts_mut(self.memory.as_mut_ptr().cast::<u8>(), self.capacity())
        }
    }

    /// Returns a slice of all currently readable data.
    #[inline]
    pub fn data(&self) -> &[u8] {
        &self.as_byte_slice()[self.position..self.end]
    }

    /// Returns a mutable slice of all currently readable data.
    #[inline]
    pub fn data_mut(&mut self) -> &mut [u8] {
        let pos = self.position;
        let end = self.end;
        &mut self.as_byte_slice_mut()[pos..end]
    }

    /// Returns a mutable slice of all available space to write into.
    #[inline]
    pub fn space(&mut self) -> &mut [u8] {
        let end = self.end;
        let capacity = self.capacity();
        &mut self.as_byte_slice_mut()[end..capacity]
    }

    /// Returns how much data can be read.
    #[inline]
    pub fn available_data(&self) -> usize {
        self.end - self.position
    }

    /// Returns how much free space is available to write to.
    #[inline]
    pub fn available_space(&self) -> usize {
        self.capacity() - self.end
    }

    /// Returns the buffer's byte capacity.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.memory.len() * std::mem::size_of::<u64>()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.position == self.end
    }

    /// Advances the read position. If past the halfway mark, shifts data to the front.
    #[inline]
    pub fn consume(&mut self, count: usize) -> usize {
        let cnt = cmp::min(count, self.available_data());
        self.position += cnt;
        if self.position > self.capacity() / 2 {
            self.shift();
        }
        cnt
    }

    /// Advances the read position without shifting.
    #[inline]
    pub fn consume_noshift(&mut self, count: usize) -> usize {
        let cnt = cmp::min(count, self.available_data());
        self.position += cnt;
        cnt
    }

    /// Marks `count` bytes (capped to available space) as written.
    #[inline]
    pub fn fill(&mut self, count: usize) -> usize {
        let cnt = cmp::min(count, self.available_space());
        self.end += cnt;
        if self.available_space() < self.available_data() + cnt {
            self.shift();
        }
        cnt
    }

    /// Grows the buffer to at least `new_size` bytes. Returns `true` if resized.
    pub fn grow(&mut self, new_size: usize) -> bool {
        if self.capacity() >= new_size {
            return false;
        }
        let new_u64_len = new_size.div_ceil(8);
        let mut new_memory = vec![0u64; new_u64_len].into_boxed_slice();
        // Copy existing bytes
        let src = self.as_byte_slice();
        // Safety: new memory is larger, copying byte_capacity bytes is valid.
        unsafe {
            std::ptr::copy_nonoverlapping(
                src.as_ptr(),
                new_memory.as_mut_ptr().cast::<u8>(),
                self.capacity(),
            );
        }
        self.memory = new_memory;
        true
    }

    /// Resets position and end to 0.
    #[inline]
    pub fn reset(&mut self) {
        self.position = 0;
        self.end = 0;
    }

    /// Moves available data to the beginning of the buffer.
    #[inline]
    pub fn shift(&mut self) {
        if self.position > 0 {
            let length = self.end - self.position;
            let pos = self.position;
            let end = self.end;
            self.as_byte_slice_mut().copy_within(pos..end, 0);
            self.position = 0;
            self.end = length;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alignment() {
        let buf = AlignedBuffer::with_capacity(100);
        let ptr = buf.data().as_ptr() as usize;
        assert_eq!(ptr % 8, 0, "buffer data must be 8-byte aligned");
    }

    #[test]
    fn test_basic_ops() {
        let mut buf = AlignedBuffer::with_capacity(16);
        assert_eq!(buf.available_data(), 0);
        assert_eq!(buf.available_space(), 16);

        // Write some data
        buf.space()[..4].copy_from_slice(b"abcd");
        buf.fill(4);
        assert_eq!(buf.available_data(), 4);
        assert_eq!(buf.data(), b"abcd");

        // Consume 2
        buf.consume_noshift(2);
        assert_eq!(buf.available_data(), 2);
        assert_eq!(buf.data(), b"cd");

        // Shift
        buf.shift();
        assert_eq!(buf.available_data(), 2);
        assert_eq!(buf.available_space(), 14);
        assert_eq!(buf.data(), b"cd");
    }

    #[test]
    fn test_grow() {
        let mut buf = AlignedBuffer::with_capacity(8);
        assert_eq!(buf.capacity(), 8);
        buf.space()[..4].copy_from_slice(b"test");
        buf.fill(4);

        buf.grow(32);
        assert!(buf.capacity() >= 32);
        assert_eq!(buf.available_data(), 4);
        assert_eq!(buf.data(), b"test");
    }

    #[test]
    fn test_reset() {
        let mut buf = AlignedBuffer::with_capacity(16);
        buf.space()[..4].copy_from_slice(b"data");
        buf.fill(4);
        buf.reset();
        assert_eq!(buf.available_data(), 0);
    }

    #[test]
    fn test_alignment_after_shift() {
        let mut buf = AlignedBuffer::with_capacity(32);
        buf.space()[..8].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        buf.fill(8);
        // Consume 3 bytes — position is now non-aligned
        buf.consume_noshift(3);
        // After shift, data moves back to position 0 (aligned)
        buf.shift();
        let ptr = buf.data().as_ptr() as usize;
        assert_eq!(ptr % 8, 0, "buffer data must be 8-byte aligned after shift");
        assert_eq!(buf.data(), &[4, 5, 6, 7, 8]);
    }
}
