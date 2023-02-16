#[cfg(feature = "juno")]
pub mod junoswap;
#[cfg(feature = "juno")]
pub mod wyndex;

#[cfg(any(feature = "phoenix-1", feature = "pisco-1"))]
pub mod astroport;
#[cfg(any(feature = "juno", feature = "osmosis"))]
pub mod osmosis;

pub mod resolver;
