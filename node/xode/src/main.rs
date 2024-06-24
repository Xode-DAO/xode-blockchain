//! Substrate Parachain Node Template CLI

#![warn(missing_docs)]

#![allow(unused_imports)]
#![allow(unused_variables)]

// Clippy lints
#![allow(clippy::expect_fun_call)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::needless_return)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::useless_conversion)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;
mod command;
mod rpc;

fn main() -> sc_cli::Result<()> {
	command::run()
}
