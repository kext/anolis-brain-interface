//! WebUSB Device Class

use embassy_usb::driver::{Driver, Endpoint, EndpointError, EndpointIn, EndpointOut};
use embassy_usb::Builder;
use embassy_usb::types::StringIndex;

/// Packet level implementation of a WebUSB device.
///
/// This class can be used directly and it has the least overhead due to directly reading and
/// writing USB packets with no intermediate buffers, but it will not act like a stream-like serial
/// port. The following constraints must be followed if you use this class directly:
///
/// - `read_packet` must be called with a buffer large enough to hold max_packet_size bytes.
/// - `write_packet` must not be called with a buffer larger than max_packet_size bytes.
/// - If you write a packet that is exactly max_packet_size bytes long, it won't be processed by the
///   host operating system until a subsequent shorter packet is sent. A zero-length packet (ZLP)
///   can be sent if there is no other data to send. This is because USB bulk transactions must be
///   terminated with a short packet, even if the bulk endpoint is used for stream-like data.
pub struct WebUsb<'d, D: Driver<'d>> {
    read_ep: D::EndpointOut,
    write_ep: D::EndpointIn,
}

impl<'d, D: Driver<'d>> WebUsb<'d, D> {
    /// Creates a new WebUsb with the provided UsbBus and max_packet_size in bytes. For
    /// full-speed devices, max_packet_size has to be one of 8, 16, 32 or 64.
    pub fn new(builder: &mut Builder<'d, D>, max_packet_size: u16, name: Option<StringIndex>) -> Self {
        assert!(builder.control_buf_len() >= 7);

        let mut func = builder.function(255, 0, 0);

        // Data interface
        let mut iface = func.interface();
        let mut alt = iface.alt_setting(255, 0, 0, name);
        let read_ep = alt.endpoint_bulk_out(max_packet_size);
        let write_ep = alt.endpoint_bulk_in(max_packet_size);

        drop(func);

        WebUsb {
            read_ep,
            write_ep,
        }
    }

    /// Gets the maximum packet size in bytes.
    pub fn max_packet_size(&self) -> u16 {
        // The size is the same for both endpoints.
        self.read_ep.info().max_packet_size
    }

    /// Writes a single packet into the IN endpoint.
    pub async fn write_packet(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        self.write_ep.write(data).await
    }

    /// Writes a message into the IN endpoint.
    /// Takes care of sending a short packet to mark the end of the message.
    pub async fn write(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        let sz = self.max_packet_size() as usize;
        let mut start = 0;
        while start <= data.len() {
            let end = start + sz;
            self.write_ep.write(&data[start..end.min(data.len())]).await?;
            start = end;
        }
        Ok(())
    }

    /// Reads a single packet from the OUT endpoint.
    pub async fn read_packet(&mut self, data: &mut [u8]) -> Result<usize, EndpointError> {
        self.read_ep.read(data).await
    }

    /// Reads a message from the OUT endpoint.
    /// This reads packets until the buffer is full or a short packet has been received.
    pub async fn read(&mut self, data: &mut [u8]) -> Result<usize, EndpointError> {
        let mut pos = 0_usize;
        let sz = self.max_packet_size() as usize;
        loop {
            let n = self.read_ep.read(&mut data[pos..]).await?;
            pos += n;
            if n < sz {
                return Ok(pos);
            }
        }
    }

    /// Waits for the USB host to enable this interface
    pub async fn wait_connection(&mut self) {
        self.read_ep.wait_enabled().await
    }

    /// Split the class into a sender and receiver.
    ///
    /// This allows concurrently sending and receiving packets from separate tasks.
    pub fn split(self) -> (Sender<'d, D>, Receiver<'d, D>) {
        (
            Sender {
                write_ep: self.write_ep,
            },
            Receiver {
                read_ep: self.read_ep,
            },
        )
    }
}

/// WebUSB packet sender.
///
/// You can obtain a `Sender` with [`WebUsb::split`]
pub struct Sender<'d, D: Driver<'d>> {
    write_ep: D::EndpointIn,
}

impl<'d, D: Driver<'d>> Sender<'d, D> {
    /// Gets the maximum packet size in bytes.
    pub fn max_packet_size(&self) -> u16 {
        // The size is the same for both endpoints.
        self.write_ep.info().max_packet_size
    }

    /// Writes a single packet into the IN endpoint.
    pub async fn write_packet(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        self.write_ep.write(data).await
    }

    /// Writes a message into the IN endpoint.
    /// Takes care of sending a short packet to mark the end of the message.
    pub async fn write(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        let sz = self.max_packet_size() as usize;
        let mut start = 0;
        while start <= data.len() {
            let end = start + sz;
            self.write_ep.write(&data[start..end.min(data.len())]).await?;
            start = end;
        }
        Ok(())
    }

    /// Waits for the USB host to enable this interface
    pub async fn wait_connection(&mut self) {
        self.write_ep.wait_enabled().await
    }
}

/// WebUSB packet receiver.
///
/// You can obtain a `Receiver` with [`WebUsb::split`]
pub struct Receiver<'d, D: Driver<'d>> {
    read_ep: D::EndpointOut,
}

impl<'d, D: Driver<'d>> Receiver<'d, D> {
    /// Gets the maximum packet size in bytes.
    pub fn max_packet_size(&self) -> u16 {
        // The size is the same for both endpoints.
        self.read_ep.info().max_packet_size
    }

    /// Reads a single packet from the OUT endpoint.
    pub async fn read_packet(&mut self, data: &mut [u8]) -> Result<usize, EndpointError> {
        self.read_ep.read(data).await
    }

    /// Reads a message from the OUT endpoint.
    /// This reads packets until the buffer is full or a short packet has been received.
    pub async fn read(&mut self, data: &mut [u8]) -> Result<usize, EndpointError> {
        let mut pos = 0;
        let sz = self.max_packet_size() as usize;
        loop {
            let n = self.read_ep.read(&mut data[pos..]).await?;
            pos += n;
            if n < sz {
                return Ok(pos);
            }
        }
    }

    /// Waits for the USB host to enable this interface
    pub async fn wait_connection(&mut self) {
        self.read_ep.wait_enabled().await
    }
}
