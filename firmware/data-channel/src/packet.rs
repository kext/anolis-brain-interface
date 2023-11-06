use alloc::boxed::Box;
use core::ptr::NonNull;
use nrf_softdevice::ble::l2cap;

/// A Packet for use with the L2CAP driver backed by heap allocated memory.
pub struct BoxPacket<const N: usize> {
    len: usize,
    data: Box<[u8; N]>,
}

impl<const N: usize> BoxPacket<N> {
    /// Allocate a new empty packet.
    pub fn new() -> Self {
        Self {
            len: 0,
            data: Box::new([0u8; N]),
        }
    }
    /// Get the length of the packet.
    pub fn len(&self) -> usize {
        self.len
    }
    /// Get a reference to the packet data.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..self.len]
    }
    /// Append the data to the packet.
    /// Panics if the data does not fit into the buffer space.
    pub fn append(&mut self, data: &[u8]) {
        self.data[self.len..self.len + data.len()].copy_from_slice(data);
        self.len += data.len();
    }
    /// Clear the packet and set its size to zero.
    pub fn reset(&mut self) {
        self.len = 0;
    }
}

impl<const N: usize> l2cap::Packet for BoxPacket<N> {
    const MTU: usize = N;
    fn allocate() -> Option<NonNull<u8>> {
        let b = Box::<[u8; N]>::new([0u8; N]);
        NonNull::new(Box::into_raw(b).cast::<u8>())
    }
    fn into_raw_parts(self) -> (NonNull<u8>, usize) {
        (NonNull::new(Box::into_raw(self.data).cast::<u8>()).unwrap(), self.len)
    }
    unsafe fn from_raw_parts(ptr: NonNull<u8>, len: usize) -> Self {
        assert!(len <= N);
        unsafe {
            Self { len, data: Box::from_raw(ptr.cast::<[u8; N]>().as_ptr()) }
        }
    }
}
