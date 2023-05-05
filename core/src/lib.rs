mod activation;
mod aggregations;
mod connection;
mod genome;
mod map;
mod mutations;
mod neat;
mod network;
mod node;
mod speciation;

pub use genome::*;
pub use map::*;
pub use neat::*;
pub use network::*;
pub use node::EnumConversion;

use godot::prelude::*;
struct Lib;

#[macro_export]
macro_rules! bind {
  ($($x:ident),*) => {
      $(
          let $x = $x.bind();
      )*
  };
}

#[macro_export]
macro_rules! bind_mut {
  ($($x:ident),*) => {
      $(
          let mut $x = $x.bind_mut();
      )*
  };
}

#[gdextension]
unsafe impl ExtensionLibrary for Lib {}
