use alloc::alloc::{alloc, dealloc};
use core::{ptr::{NonNull, copy_nonoverlapping}, alloc::Layout, ops::{Deref, DerefMut}, slice, mem::ManuallyDrop};
use nrf_softdevice::ble::l2cap::Packet;

/// A Packet for use with the L2CAP driver backed by heap allocated memory.
#[derive(defmt::Format)]
pub struct BoxPacket<const N: usize> {
    len: usize,
    ptr: NonNull<u8>,
}

impl<const N: usize> BoxPacket<N> {
    /// Allocate a new empty packet.
    pub fn new() -> Option<Self> {
        Self::allocate().map(|ptr| {
            Self { len: 0, ptr }
        })
    }
    /// Append the data to the packet.
    /// Panics if the data does not fit into the buffer space.
    pub fn append(&mut self, data: &[u8]) {
        let n = data.len();
        assert!(self.len + n <= N);
        unsafe {
            copy_nonoverlapping(data.as_ptr(), self.ptr.as_ptr().add(self.len), n);
        }
        self.len += n;
    }
    /// Clear the packet and set its size to zero.
    pub fn reset(&mut self) {
        self.len = 0;
    }
}

impl<const N: usize> Drop for BoxPacket<N> {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.ptr.as_ptr(), Layout::array::<u8>(N).unwrap())
        }
    }
}

impl<const N: usize> Deref for BoxPacket<N> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        unsafe {
            slice::from_raw_parts(self.ptr.as_ptr(), self.len)
        }
    }
}

impl<const N: usize> DerefMut for BoxPacket<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len)
        }
    }
}

impl<const N: usize> Packet for BoxPacket<N> {
    const MTU: usize = N;
    fn allocate() -> Option<NonNull<u8>> {
        unsafe {
            NonNull::new(alloc(Layout::array::<u8>(N).unwrap()))
        }
    }
    fn into_raw_parts(self) -> (NonNull<u8>, usize) {
        let me = ManuallyDrop::new(self);
        (me.ptr, me.len)
    }
    unsafe fn from_raw_parts(ptr: NonNull<u8>, len: usize) -> Self {
        assert!(len <= N);
        Self { len, ptr }
    }
}
