//! Minecraft Java protokolunun bu bot icin gereken en kucuk parcasi.
//!
//! KAPSAM BILEREK KUCUK: bot bir oyuncu gibi girip uzakta gorunmez durmaktan
//! baska bir sey yapmiyor. Dolayisiyla dunya, envanter, fizik, varlik takibi
//! YOK. Gereken sadece: cerceveleme (framing), sikistirma, ve bir avuc paket.
//!
//! SIFRELEME YOK, cunku yalnizca offline (cracked) sunucuya baglaniyoruz.
//! Protokolun en zor parcasi olan sifreleme el sikismasi bu sayede tamamen
//! devre disi — online-mode sunucular zaten offline bir hesabi reddediyor
//! (bkz. mcbot: unverified_username).

use std::io::{Cursor, Read};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;

/// Tek bir cerceveye izin verilen en buyuk boy.
///
/// Bir sinir SART: cerceve uzunlugu telden geliyor ve dogrulanmadan
/// `vec![0; len]` demek, bozuk ya da kotu niyetli tek bir varint ile
/// gigabaytlarca ayirma anlamina gelir. 16 MB, configuration asamasinda gelen
/// registry verisinin (bir kac MB olabiliyor) rahat sigacagi bir ust sinir.
const MAX_FRAME: usize = 16 * 1024 * 1024;

// ─── Yazma yardimcilari ───

pub fn put_varint(buf: &mut Vec<u8>, v: i32) {
    let mut u = v as u32;
    loop {
        if (u & !0x7F) == 0 {
            buf.push(u as u8);
            return;
        }
        buf.push(((u & 0x7F) | 0x80) as u8);
        u >>= 7;
    }
}

pub fn put_str(buf: &mut Vec<u8>, s: &str) {
    put_varint(buf, s.len() as i32);
    buf.extend_from_slice(s.as_bytes());
}

pub fn put_u16(buf: &mut Vec<u8>, v: u16) {
    buf.extend_from_slice(&v.to_be_bytes());
}

pub fn put_i64(buf: &mut Vec<u8>, v: i64) {
    buf.extend_from_slice(&v.to_be_bytes());
}

pub fn put_f32(buf: &mut Vec<u8>, v: f32) {
    buf.extend_from_slice(&v.to_be_bytes());
}

pub fn put_uuid(buf: &mut Vec<u8>, uuid: u128) {
    buf.extend_from_slice(&uuid.to_be_bytes());
}

// ─── Okuma yardimcilari ───

pub fn read_varint(cur: &mut Cursor<Vec<u8>>) -> Result<i32, String> {
    let mut result: i32 = 0;
    let mut shift = 0;
    loop {
        let mut b = [0u8; 1];
        std::io::Read::read_exact(cur, &mut b).map_err(|e| format!("varint okunamadi: {e}"))?;
        result |= ((b[0] & 0x7F) as i32) << shift;
        if b[0] & 0x80 == 0 {
            return Ok(result);
        }
        shift += 7;
        // 5 bayt, 32 bitlik bir varint icin ust sinir. Bu kontrol olmadan
        // bozuk bir akista sonsuza kadar okunur.
        if shift >= 35 {
            return Err("varint fazla uzun".into());
        }
    }
}

pub fn read_str(cur: &mut Cursor<Vec<u8>>) -> Result<String, String> {
    let len = read_varint(cur)?;
    if len < 0 || len as usize > MAX_FRAME {
        return Err(format!("gecersiz string uzunlugu: {len}"));
    }
    let mut buf = vec![0u8; len as usize];
    std::io::Read::read_exact(cur, &mut buf).map_err(|e| format!("string okunamadi: {e}"))?;
    String::from_utf8(buf).map_err(|e| format!("string utf8 degil: {e}"))
}

pub fn read_f64(cur: &mut Cursor<Vec<u8>>) -> Result<f64, String> {
    let mut b = [0u8; 8];
    std::io::Read::read_exact(cur, &mut b).map_err(|e| format!("f64 okunamadi: {e}"))?;
    Ok(f64::from_be_bytes(b))
}

pub fn read_i64(cur: &mut Cursor<Vec<u8>>) -> Result<i64, String> {
    let mut b = [0u8; 8];
    std::io::Read::read_exact(cur, &mut b).map_err(|e| format!("i64 okunamadi: {e}"))?;
    Ok(i64::from_be_bytes(b))
}

// ─── Baglanti ───

/// Cerceveleme ve sikistirmayi yoneten baglanti sarmalayicisi.
pub struct Conn {
    reader: BufReader<OwnedReadHalf>,
    writer: OwnedWriteHalf,
    /// Sikistirma esigi. `-1` = sikistirma kapali (Set Compression gelmedi).
    threshold: i32,
}

impl Conn {
    pub async fn connect(host: &str, port: u16, timeout: std::time::Duration) -> Result<Self, String> {
        let stream = tokio::time::timeout(timeout, TcpStream::connect((host, port)))
            .await
            .map_err(|_| "tcp baglanti zaman asimi".to_string())?
            .map_err(|e| format!("tcp baglanti hatasi: {e}"))?;
        // Nagle kapali: gonderdigimiz paketler kucuk ve gecikmeye duyarli
        // (keepalive cevabi geciken bir istemci sunucu tarafindan atilir).
        let _ = stream.set_nodelay(true);
        let (r, w) = stream.into_split();
        Ok(Self {
            reader: BufReader::new(r),
            writer: w,
            threshold: -1,
        })
    }

    /// Sunucu Set Compression gonderdiginde cagrilir.
    pub fn set_threshold(&mut self, t: i32) {
        self.threshold = t;
    }

    pub async fn send(&mut self, id: i32, body: &[u8]) -> Result<(), String> {
        let mut payload = Vec::with_capacity(body.len() + 5);
        put_varint(&mut payload, id);
        payload.extend_from_slice(body);

        let mut frame = Vec::with_capacity(payload.len() + 10);
        if self.threshold >= 0 {
            // Sikistirmali bicim: [cerceve_uzunlugu][veri_uzunlugu][govde]
            // veri_uzunlugu == 0 => govde sikistirilmamis. Esigin ALTINDAKI
            // paketleri sikistirmak protokole aykiri; sunucu reddediyor.
            let mut inner = Vec::with_capacity(payload.len() + 5);
            if payload.len() as i32 >= self.threshold {
                let compressed = zlib_compress(&payload)?;
                put_varint(&mut inner, payload.len() as i32);
                inner.extend_from_slice(&compressed);
            } else {
                put_varint(&mut inner, 0);
                inner.extend_from_slice(&payload);
            }
            put_varint(&mut frame, inner.len() as i32);
            frame.extend_from_slice(&inner);
        } else {
            put_varint(&mut frame, payload.len() as i32);
            frame.extend_from_slice(&payload);
        }

        self.writer
            .write_all(&frame)
            .await
            .map_err(|e| format!("paket yazilamadi: {e}"))
    }

    /// Bir paket okur ve `(paket_id, govde)` dondurur.
    pub async fn recv(&mut self) -> Result<(i32, Vec<u8>), String> {
        let len = self.read_varint_async().await?;
        if len <= 0 || len as usize > MAX_FRAME {
            return Err(format!("gecersiz cerceve uzunlugu: {len}"));
        }
        let mut buf = vec![0u8; len as usize];
        self.reader
            .read_exact(&mut buf)
            .await
            .map_err(|e| format!("cerceve okunamadi: {e}"))?;

        let mut cur = Cursor::new(buf);
        if self.threshold >= 0 {
            let data_len = read_varint(&mut cur)?;
            let pos = cur.position() as usize;
            let inner = cur.into_inner();
            let rest = &inner[pos..];
            let decoded = if data_len == 0 {
                rest.to_vec()
            } else {
                if data_len < 0 || data_len as usize > MAX_FRAME {
                    return Err(format!("gecersiz veri uzunlugu: {data_len}"));
                }
                zlib_decompress(rest, data_len as usize)?
            };
            let mut c = Cursor::new(decoded);
            let id = read_varint(&mut c)?;
            let p = c.position() as usize;
            let v = c.into_inner();
            Ok((id, v[p..].to_vec()))
        } else {
            let id = read_varint(&mut cur)?;
            let pos = cur.position() as usize;
            let v = cur.into_inner();
            Ok((id, v[pos..].to_vec()))
        }
    }

    /// Cerceve uzunlugunu telden bayt bayt okur.
    ///
    /// Bayt bayt olmasi ZORUNLU: uzunlugu okumadan cercevenin nerede bittigini
    /// bilmiyoruz, fazla okuyan bir tampon sonraki paketi yutar.
    async fn read_varint_async(&mut self) -> Result<i32, String> {
        let mut result: i32 = 0;
        let mut shift = 0;
        loop {
            let b = self
                .reader
                .read_u8()
                .await
                .map_err(|e| format!("baglanti kapandi: {e}"))?;
            result |= ((b & 0x7F) as i32) << shift;
            if b & 0x80 == 0 {
                return Ok(result);
            }
            shift += 7;
            if shift >= 35 {
                return Err("cerceve uzunlugu varint'i fazla uzun".into());
            }
        }
    }
}

fn zlib_compress(data: &[u8]) -> Result<Vec<u8>, String> {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;
    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    e.write_all(data).map_err(|e| format!("zlib yazma: {e}"))?;
    e.finish().map_err(|e| format!("zlib bitirme: {e}"))
}

fn zlib_decompress(data: &[u8], expected: usize) -> Result<Vec<u8>, String> {
    use flate2::read::ZlibDecoder;
    let mut out = Vec::with_capacity(expected.min(MAX_FRAME));
    ZlibDecoder::new(data)
        .take(MAX_FRAME as u64)
        .read_to_end(&mut out)
        .map_err(|e| format!("zlib acilamadi: {e}"))?;
    Ok(out)
}

/// Offline (cracked) UUID: "OfflinePlayer:<isim>" MD5'i, v3 UUID olarak.
///
/// Vanilla sunucu offline modda bunu isimden kendisi turetiyor; dogrusunu
/// gondermek yine de ucuz ve isim/uuid tutarsizligindan dogabilecek surprizleri
/// eler.
pub fn offline_uuid(name: &str) -> u128 {
    use md5::{Digest, Md5};
    let mut hasher = Md5::new();
    hasher.update(format!("OfflinePlayer:{name}").as_bytes());
    let mut d = hasher.finalize();
    // RFC 4122: surum 3 ve varyant bitleri.
    d[6] = (d[6] & 0x0f) | 0x30;
    d[8] = (d[8] & 0x3f) | 0x80;
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&d);
    u128::from_be_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varint_roundtrip() {
        // Sinir degerler dahil: 0, tek bayt ust siniri, cok baytliya gecis,
        // ve negatif (protokolde negatif varint gercekten gecebiliyor).
        for v in [0i32, 1, 127, 128, 255, 2097151, 2097152, i32::MAX, -1, i32::MIN] {
            let mut buf = Vec::new();
            put_varint(&mut buf, v);
            let mut cur = Cursor::new(buf);
            assert_eq!(read_varint(&mut cur).unwrap(), v, "varint {v}");
        }
    }

    #[test]
    fn string_roundtrip() {
        let mut buf = Vec::new();
        put_str(&mut buf, "AterkeepBot");
        let mut cur = Cursor::new(buf);
        assert_eq!(read_str(&mut cur).unwrap(), "AterkeepBot");
    }

    #[test]
    fn offline_uuid_is_v3() {
        let u = offline_uuid("Notch").to_be_bytes();
        assert_eq!(u[6] & 0xf0, 0x30, "surum 3 olmali");
        assert_eq!(u[8] & 0xc0, 0x80, "RFC 4122 varyanti olmali");
    }

    #[test]
    fn offline_uuid_is_stable_and_name_dependent() {
        assert_eq!(offline_uuid("Alex"), offline_uuid("Alex"));
        assert_ne!(offline_uuid("Alex"), offline_uuid("Steve"));
    }

    #[test]
    fn zlib_roundtrip() {
        let data = b"aterkeep anti-idle bot payload".repeat(20);
        let c = zlib_compress(&data).unwrap();
        assert_eq!(zlib_decompress(&c, data.len()).unwrap(), data);
    }
}
