pub type F2d14 = f32;

macro_rules! read {
    ($reader:expr => [$type:ty; $len:expr]) => {{
        let mut buf = vec![0u8; std::mem::size_of::<$type>() * $len];
        let out: std::io::Result<Vec<$type>> = $reader
            .read_exact(&mut buf)
            .map(|_| buf.chunks(std::mem::size_of::<$type>()).map(|slice| <$type>::from_be_bytes(slice.try_into().unwrap())).collect());
        out
    }};
    ($reader:expr => ($($type:ty),+)) => {
        (|| {
            let result: std::io::Result<($($type),+)> = Ok(($(read!($reader => $type)?),+));
            result
        })()
    };
    ($reader:expr => F2d14) => {{
        let mut buf = [0u8; 2];
        $reader
            .read_exact(&mut buf)
            .map(|_| {
                let raw = u16::from_be_bytes(buf);
                let sign = ((raw >> 15) as u32) << 31;
                let exponent = 128u32 << 23;
                let mantissa = ((raw << 1) as u32) << 7;

                f32::from_bits(sign | exponent | mantissa)
            })
    }};
    ($reader:expr => $type:ty) => {{
        let mut buf = [0u8; std::mem::size_of::<$type>()];
        $reader
            .read_exact(&mut buf)
            .map(|_| <$type>::from_be_bytes(buf))
    }};
}

pub(crate) use read;
