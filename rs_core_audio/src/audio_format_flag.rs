use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct AudioFormatFlag: u32 {
        const isFloat = 1 << 0;
        const isSignedInteger = 1 << 1;
        const isNonInterleaved = 1 << 2;
        const isBigEndian = 1 << 3;
    }
}
