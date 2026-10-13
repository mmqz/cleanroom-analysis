//! zip 8.6 谎言头读界行为直测（xlsx-r1 DEPS 对齐验证补充件）。
//! 构造：deflate 条目声明 16B、实际膨胀 1MB、CRC 正确（T3 3b 同构）。
//! 观察：8.6 读取行为（拒绝/截断/全量读回）。
use std::io::Read;
use std::io::Write;

fn crc32(data: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (i, t) in table.iter_mut().enumerate() {
        let mut c = i as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
        *t = c;
    }
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc = table[((crc ^ u32::from(b)) & 0xFF) as usize] ^ (crc >> 8);
    }
    crc ^ 0xFFFF_FFFF
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1MB 可压缩载荷 → deflate 压缩
    let payload: Vec<u8> = vec![0x41; 1 << 20];
    let mut enc = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
    enc.write_all(&payload)?;
    let compressed = enc.finish()?;
    let crc = crc32(&payload);
    let name = b"blob.bin";

    let mut z = Vec::new();
    // LFH（声明 16B——谎言头）
    z.extend_from_slice(b"PK\x03\x04");
    z.extend_from_slice(&20u16.to_le_bytes());
    z.extend_from_slice(&0u16.to_le_bytes()); // flags
    z.extend_from_slice(&8u16.to_le_bytes()); // deflate
    z.extend_from_slice(&0u16.to_le_bytes()); // time
    z.extend_from_slice(&0u16.to_le_bytes()); // date
    z.extend_from_slice(&crc.to_le_bytes());
    z.extend_from_slice(&(compressed.len() as u32).to_le_bytes()); // csize 真实
    z.extend_from_slice(&16u32.to_le_bytes()); // usize 谎言（真实 1MB）
    z.extend_from_slice(&(name.len() as u16).to_le_bytes());
    z.extend_from_slice(&0u16.to_le_bytes()); // extra len
    z.extend_from_slice(name);
    z.extend_from_slice(&compressed);
    // CDFH
    let cd_off = z.len() as u32;
    z.extend_from_slice(b"PK\x01\x02");
    z.extend_from_slice(&20u16.to_le_bytes());
    z.extend_from_slice(&20u16.to_le_bytes());
    z.extend_from_slice(&0u16.to_le_bytes());
    z.extend_from_slice(&8u16.to_le_bytes());
    z.extend_from_slice(&0u16.to_le_bytes());
    z.extend_from_slice(&0u16.to_le_bytes());
    z.extend_from_slice(&crc.to_le_bytes());
    z.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
    z.extend_from_slice(&16u32.to_le_bytes()); // usize 谎言
    z.extend_from_slice(&(name.len() as u16).to_le_bytes());
    z.extend_from_slice(&0u16.to_le_bytes()); // extra
    z.extend_from_slice(&0u16.to_le_bytes()); // comment
    z.extend_from_slice(&0u16.to_le_bytes()); // disk
    z.extend_from_slice(&0u16.to_le_bytes()); // iattr
    z.extend_from_slice(&0u32.to_le_bytes()); // eattr
    z.extend_from_slice(&0u32.to_le_bytes()); // LFH 偏移（在 0 位）
    z.extend_from_slice(name);
    // EOCD
    let cd_size = z.len() as u32 - cd_off;
    z.extend_from_slice(b"PK\x05\x06");
    z.extend_from_slice(&0u16.to_le_bytes());
    z.extend_from_slice(&0u16.to_le_bytes());
    z.extend_from_slice(&1u16.to_le_bytes());
    z.extend_from_slice(&1u16.to_le_bytes());
    z.extend_from_slice(&cd_size.to_le_bytes());
    z.extend_from_slice(&cd_off.to_le_bytes()); // EOCD 中央目录偏移
    z.extend_from_slice(&0u16.to_le_bytes());

    let p = "/tmp/lie86_test.zip";
    std::fs::write(p, &z)?;

    // 读观察（zip 8.6 当前依赖）
    let f = std::fs::File::open(p)?;
    let mut za = zip::ZipArchive::new(f)?;
    println!("entries: {}, declared size: {}", za.len(), za.by_index(0)?.size());
    let mut zf = za.by_index(0)?;
    let mut buf = Vec::new();
    match zf.read_to_end(&mut buf) {
        Ok(n) => println!("read_to_end OK: {n} bytes (declared 16)"),
        Err(e) => println!("read_to_end ERR: {e}"),
    }
    Ok(())
}
