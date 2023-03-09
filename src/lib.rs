#![feature(associated_type_defaults)]
#![feature(generators, generator_trait)]

pub mod binary_heap_set;
pub mod error;
pub mod gen_iter_return_result;
pub mod node;
pub mod wave_function;

#[cfg(feature = "tile2d")]
pub mod tile2d;

pub use gen_iter_return_result::GenIterReturnResult;
pub use wave_function::collapse_wave;
