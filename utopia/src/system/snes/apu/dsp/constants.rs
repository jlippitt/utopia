#[rustfmt::skip]
pub const RATE: [Option<u32>; 32] = [
    None,       Some(2048), Some(1536), Some(1280),
    Some(1024), Some(768),  Some(640),  Some(512),
    Some(384),  Some(320),  Some(256),  Some(192),
    Some(160),  Some(128),  Some(96),   Some(80),
    Some(64),   Some(48),   Some(40),   Some(32),
    Some(24),   Some(20),   Some(16),   Some(12),
    Some(10),   Some(8),    Some(6),    Some(5),
    Some(4),    Some(3),    Some(2),    Some(1),
];
