#[warn(clippy::pedantic)]
/// Contains backends that lower the IR provided by `middle` to various targets
mod back;
/// Contains a frontend which lexes, parses, and lowers Desmos expressions to DesMIR
mod front;
/// Contains shared definitions and code for generating and working with DesMIR
mod middle;
/// Contains shared definitions of various useful structs
pub(crate) mod util;