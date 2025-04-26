macro_rules! flag {
    ({ $($variant:ident),+ $(,)? } => ($name:ident, $alias:ident): $repr:ty) => {
        #[allow(dead_code)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum $name {
            $($variant),+
        }

        #[allow(dead_code)]
        impl $name {
            pub fn pos(self) -> $repr {
                self as $repr
            }

            pub fn mask(self) -> $repr {
                1 << self.pos()
            }

            pub fn test(self, value: $repr) -> bool {
                (value & self.mask()) != 0
            }
        }

        #[allow(dead_code)]
        pub type $alias = $repr;
    };
}

flag!({
    OnCurve,
    XshortVector,
    YshortVector,
    Repeat,
    XSame,
    YSame,
} => (OutlineFlag, OutlineFlags): u8);

flag!({
    Arg1And2AreWords,
    ArgsAreXyValues,
    RoundXyToGrid,
    WeHaveAScale,
    Obsolete,
    MoreComponents,
    WeHaveAnXAndYScale,
    WeHaveATwoByTwo,
    WeHaveInstructions,
    UseMyMetrics,
    OverlapCompound,
} => (ComponentFlag, ComponentFlags): u16);
