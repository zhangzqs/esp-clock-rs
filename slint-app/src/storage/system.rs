use embedded_svc::storage::RawStorage;

use super::{StorageHelper, StorageHelperMut};

const BOOT_COUNT: &str = "boot_count";

pub struct System<'a, Raw: RawStorage>(pub &'a Raw);

impl<'a, Raw: RawStorage> System<'a, Raw> {
    pub fn get_boot_count(&self) -> u32 {
        StorageHelper(self.0)
            .get_u32(BOOT_COUNT)
            .unwrap()
            .unwrap_or(0)
    }
}

pub struct SystemMut<'a, Raw: RawStorage>(&'a mut Raw);

impl<'a, Raw: RawStorage> SystemMut<'a, Raw> {
    pub fn set_boot_count(&mut self, value: u32) {
        StorageHelperMut(self.0).set_u32(BOOT_COUNT, value).unwrap();
    }
}
