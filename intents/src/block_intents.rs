use super::*;

// Pattern for sealing from other implementations.
// see: https://rust-lang.github.io/api-guidelines/future-proofing.html
mod sealed {
    pub trait Sealed {}
}

pub trait BlockIntents: sealed::Sealed {
    const N: usize;
    fn len() -> usize {
        Self::N
    }
    fn as_slice(&self) -> &[Intent];
}

/// Macro defining a set of BlockIntents implementations for
/// fixed-size arrays of Intent. Call like:
/// ```ignore
/// declare_intents!(ThreeIntents, 3);
/// ```
/// to declare a BlockIntents implementation for three intents.
/// You can then use `ThreeIntents::new([intent1, intent2, intent3])`
/// to create an instance. The number of intents must match the
/// compiler-time constant given to the macro.
macro_rules! declare_intents {
    ($name:ident, 0) => {
        #[derive(Clone, Debug)]
        pub struct $name;

        impl $name {
            pub fn new() -> Self {
                Self
            }
            pub fn from_array(_: [Intent; 0]) -> Self {
                Self
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl sealed::Sealed for $name {}

        impl BlockIntents for $name {
            const N: usize = 0;
            fn as_slice(&self) -> &[Intent] {
                &[]
            }
        }
    };

    ($name:ident, $n:expr) => {
        #[derive(Clone, Debug)]
        pub struct $name([Intent; $n]);

        impl $name {
            pub fn new(intents: [Intent; $n]) -> Self {
                Self(intents)
            }
            pub fn from_array(intents: [Intent; $n]) -> Self {
                Self(intents)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self(std::array::from_fn(|i| {
                    Intent::no_intent(SlotId::new(0, i as u32))
                }))
            }
        }

        impl sealed::Sealed for $name {}

        impl BlockIntents for $name {
            const N: usize = $n;
            fn as_slice(&self) -> &[Intent] {
                &self.0
            }
        }
    };
}

declare_intents!(ZeroIntents, 0);
declare_intents!(OneIntent, 1);
declare_intents!(TwoIntents, 2);
declare_intents!(ThreeIntents, 3);
declare_intents!(FourIntents, 4);
declare_intents!(FiveIntents, 5);
