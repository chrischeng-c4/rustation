// Brace expansion module
// Implements shell brace expansion: {a,b,c}, {1..10}, {a..z}

mod expander;
mod lexer;
mod parser;

pub use expander::expand_brace;
