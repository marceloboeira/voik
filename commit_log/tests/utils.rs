use crc::crc64;
use crc::crc64::Digest;

pub fn crc_digest() -> Digest {
    crc64::Digest::new(crc64::ECMA)
}

pub fn generate_random_values<F>(num_values: usize, item_size: usize, mut f: F)
where
    F: FnMut(&[u8]),
{
    for _ in 0..num_values {
        let random_bytes: Vec<u8> = (0..item_size).map(|_| rand::random::<u8>()).collect();

        f(&random_bytes);
    }
}
