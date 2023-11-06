use nrf_softdevice::raw;

const SERVICE_LIST: &'static [u8] = &[7, 0x3c, 0x53, 0x4c, 0xb6, 0xf0, 0x86, 0x02, 0xa1, 0x85, 0x42, 0x47, 0x83, 0x42, 0x4b, 0xb7, 0xed];

pub fn supports_data_service(adv_report: &raw::ble_gap_evt_adv_report_t) -> bool {
    AdvertisementData::new(adv_report).into_iter().find(|d| *d == SERVICE_LIST).is_some()
}

/// Iterator over advertisement data.
/// Advertisement data is a list of bytes
struct AdvertisementDataIterator<'a> {
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

struct AdvertisementData<'a> {
    data: &'a [u8],
}

impl<'a> AdvertisementData<'a> {
    fn new(adv_report: &'a raw::ble_gap_evt_adv_report_t) -> Self {
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
