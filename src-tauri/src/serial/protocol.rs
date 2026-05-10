//! 协议解析模块
//! 负责多种通信协议的解析与封装

/// 支持的 CRC/校验和算法
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CrcAlgorithm {
    Modbus,
    Xor8,
    Crc16Ccitt,
    Crc32,
}

impl CrcAlgorithm {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "modbus" => Some(CrcAlgorithm::Modbus),
            "xor8" => Some(CrcAlgorithm::Xor8),
            "crc16-ccitt" => Some(CrcAlgorithm::Crc16Ccitt),
            "crc32" => Some(CrcAlgorithm::Crc32),
            _ => None,
        }
    }
}

/// 计算指定算法的 CRC/校验和，并返回要追加的字节
pub fn compute_crc(algorithm: CrcAlgorithm, data: &[u8]) -> Vec<u8> {
    match algorithm {
        CrcAlgorithm::Modbus => {
            use crc::{Crc, CRC_16_MODBUS};
            let crc = Crc::<u16>::new(&CRC_16_MODBUS);
            let checksum = crc.checksum(data);
            vec![(checksum & 0xFF) as u8, ((checksum >> 8) & 0xFF) as u8]
        }
        CrcAlgorithm::Xor8 => {
            let xor = data.iter().fold(0u8, |acc, &b| acc ^ b);
            vec![xor]
        }
        CrcAlgorithm::Crc16Ccitt => {
            // CRC-16-CCITT-FALSE (与 CRC_16_IBM_3740 等价)
            use crc::{Algorithm, Crc};
            const ALGO: Algorithm<u16> = Algorithm::<u16> {
                width: 16,
                poly: 0x1021,
                init: 0xffff,
                refin: false,
                refout: false,
                xorout: 0x0000,
                check: 0x29b1,
                residue: 0x0000,
            };
            let crc = Crc::<u16>::new(&ALGO);
            let checksum = crc.checksum(data);
            vec![(checksum & 0xFF) as u8, ((checksum >> 8) & 0xFF) as u8]
        }
        CrcAlgorithm::Crc32 => {
            use crc::{Crc, CRC_32_ISO_HDLC};
            let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
            let checksum = crc.checksum(data);
            vec![
                (checksum & 0xFF) as u8,
                ((checksum >> 8) & 0xFF) as u8,
                ((checksum >> 16) & 0xFF) as u8,
                ((checksum >> 24) & 0xFF) as u8,
            ]
        }
    }
}

/// 将 CRC 字节追加到数据末尾
pub fn append_crc(data: &mut Vec<u8>, algorithm: CrcAlgorithm) {
    let crc_bytes = compute_crc(algorithm, data);
    data.extend_from_slice(&crc_bytes);
}
