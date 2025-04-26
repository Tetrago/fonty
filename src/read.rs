macro_rules! read {
    ($reader:expr => ($($type:ty),+)) => {
        (|| {
            let result: io::Result<($($type),+)> = Ok(($(read!($reader => $type)?),+));
            result
        })()
    };
    ($reader:expr => $type:ty) => {{
        let mut buf = [0u8; std::mem::size_of::<$type>()];
        $reader
            .read_exact(&mut buf)
            .map(|_| <$type>::from_be_bytes(buf))
    }};
}

pub(crate) use read;
