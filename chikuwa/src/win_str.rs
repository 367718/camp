#[macro_export]
macro_rules! win_str {
    ($base:expr) => {
        $base.encode_utf16()
            .chain(Some(0))
            .collect::<Vec<u16>>()
    };
}
