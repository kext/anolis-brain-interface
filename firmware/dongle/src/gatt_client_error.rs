use nrf_softdevice::ble::gatt_client::{
    DiscoverError, MtuExchangeError, ReadError, TryWriteError, WriteError,
};

/// All possible error types that can happen during operation of a GATT client.
/// To use those errors with the question mark operator we need to implement the From trait
/// for each variant.
pub enum GattClientError {
    DiscoverError(DiscoverError),
    MtuExchangeError(MtuExchangeError),
    ReadError(ReadError),
    TryWriteError(TryWriteError),
    WriteError(WriteError),
}
impl From<DiscoverError> for GattClientError {
    fn from(value: DiscoverError) -> Self {
        Self::DiscoverError(value)
    }
}
impl TryInto<DiscoverError> for GattClientError {
    type Error = ();
    fn try_into(self) -> Result<DiscoverError, Self::Error> {
        match self {
            Self::DiscoverError(value) => Ok(value),
            _ => Err(()),
        }
    }
}
impl From<MtuExchangeError> for GattClientError {
    fn from(value: MtuExchangeError) -> Self {
        Self::MtuExchangeError(value)
    }
}
impl From<ReadError> for GattClientError {
    fn from(value: ReadError) -> Self {
        Self::ReadError(value)
    }
}
impl From<TryWriteError> for GattClientError {
    fn from(value: TryWriteError) -> Self {
        Self::TryWriteError(value)
    }
}
impl From<WriteError> for GattClientError {
    fn from(value: WriteError) -> Self {
        Self::WriteError(value)
    }
}
