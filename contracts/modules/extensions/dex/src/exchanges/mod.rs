#[cfg(feature = "juno")]
pub mod junoswap;
#[cfg(any(feature = "juno", feature = "terra"))]
pub mod loop_dex;
#[cfg(feature = "terra")]
pub mod terraswap;

#[cfg(any(feature = "juno", feature = "osmosis"))]
pub mod osmosis;

pub(crate) mod exchange_resolver;
