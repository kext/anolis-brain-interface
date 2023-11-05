use nrf_softdevice::raw;

/// Iterator over advertisement data.
/// Advertisement data is a list of bytes
pub struct AdvertisementDataIterator<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> AdvertisementDataIterator<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }
}

impl<'a> Iterator for AdvertisementDataIterator<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<&'a [u8]> {
        if self.pos >= self.data.len() {
            return None;
        }
        let len = self.data[self.pos] as usize;
        if self.pos + len + 1 <= self.data.len() {
            let r = Some(&self.data[self.pos + 1..self.pos + len + 1]);
            self.pos += len + 1;
            r
        } else {
            None
        }
    }
}

/// Extract the advertisement data from a raw advertisement report.
fn get_advertisement_data(adv_report: &raw::ble_gap_evt_adv_report_t) -> &[u8] {
    if adv_report.data.len > 0 {
        // SAFETY: The advertisement data lives as long as the report.
        unsafe { core::slice::from_raw_parts(adv_report.data.p_data, adv_report.data.len as usize) }
    } else {
        &[]
    }
}

pub struct AdvertisementData<'a> {
    data: &'a [u8],
}

impl<'a> AdvertisementData<'a> {
    pub fn new(adv_report: &'a raw::ble_gap_evt_adv_report_t) -> Self {
        Self {
            data: get_advertisement_data(adv_report),
        }
    }
}

impl<'a> IntoIterator for AdvertisementData<'a> {
    type IntoIter = AdvertisementDataIterator<'a>;
    type Item = &'a [u8];
    fn into_iter(self) -> Self::IntoIter {
        AdvertisementDataIterator::new(self.data)
    }
}
