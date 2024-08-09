use std::marker::PhantomData;

// TODO: implement dev-facing API for abstract-ica
pub struct InterchainAccount<T> {
    _marker: PhantomData<T>,
}

// impl<T> InterchainAccount<T> {
//     pub fn new() -> Self {
//         Self {
//             _marker: PhantomData<T>
//         }
//     }
// }

// /// EVM API
// impl<T: ChainType<Type = Evm>> InterchainAccount<> {
//     fn execute(...)
// }

// /// Cosmos API
// impl<T: ChainType<Type = Cosmos>> InterchainAccount<> {
//     fn execute(...)
// }

// struct Polygon;

// struct Evm;

// trait ChainType {
//     type Type;
// }

// trait ChainId {
//     fn chain_id(&self) -> &'static str;
// }
// impl ChainType for Polygon {
//     type Type = Evm;
// };

// impl ChainId for Polygon {
//     fn chain_id(&self) -> u64 {
//         137
//     }
// }
