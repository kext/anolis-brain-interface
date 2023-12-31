use nrf_softdevice::ble::l2cap::{Packet, RxError, SetupError, TxError};

/// Unified error type for all L2CAP errors.
#[derive(defmt::Format)]
pub enum L2capError<P: Packet> {
    SetupError(SetupError),
    RxError(RxError),
    TxError(TxError<P>),
}
impl<P: Packet> From<SetupError> for L2capError<P> {
    fn from(value: SetupError) -> Self {
        Self::SetupError(value)
    }
}
impl<P: Packet> From<RxError> for L2capError<P> {
    fn from(value: RxError) -> Self {
        Self::RxError(value)
    }
}
impl<P: Packet> From<TxError<P>> for L2capError<P> {
    fn from(value: TxError<P>) -> Self {
        Self::TxError(value)
    }
}
