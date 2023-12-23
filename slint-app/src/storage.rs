use embedded_svc::storage::RawStorage;

use self::system::System;

mod system;

struct StorageHelperMut<'a, Raw: RawStorage>(pub &'a mut Raw);

impl<'a, Raw: RawStorage> StorageHelperMut<'a, Raw> {
    pub fn set_u32(&mut self, name: &str, value: u32) -> Result<bool, Raw::Error> {
        let mut buf = [0u8; 4];
        buf[0] = (value >> 24) as u8;
        buf[1] = (value >> 16) as u8;
        buf[2] = (value >> 8) as u8;
        buf[3] = value as u8;
        self.0.set_raw(name, &buf)
    }
}

struct StorageHelper<'a, Raw: RawStorage>(pub &'a Raw);

impl<'a, Raw: RawStorage> StorageHelper<'a, Raw> {
    pub fn get_u32(&self, name: &str) -> Result<Option<u32>, Raw::Error> {
        let mut buf = [0u8; 4];
        if let Some(buf) = self.0.get_raw(name, &mut buf)? {
            Ok(Some(
                ((buf[0] as u32) << 24)
                    | ((buf[1] as u32) << 16)
                    | ((buf[2] as u32) << 8)
                    | (buf[3] as u32),
            ))
        } else {
            Ok(None)
        }
    }
}

struct Storage<'a, Raw: RawStorage>(&'a Raw);

impl<'a, Raw: RawStorage> Storage<'a, Raw> {
    pub fn system(&self) -> System<'a, Raw> {
        System(&self.0)
    }
}
