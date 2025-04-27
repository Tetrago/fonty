macro_rules! read {
    ($reader:expr => [$type:ty; $len:expr]) => {{
        let mut buf = vec![0u8; std::mem::size_of::<$type>() * $len];
        let out: io::Result<Vec<$type>> = $reader
            .read_exact(&mut buf)
            .map(|_| buf.chunks(std::mem::size_of::<$type>()).map(|slice| <$type>::from_be_bytes(slice.try_into().unwrap())).collect());
        out
    }};
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
