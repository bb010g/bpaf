#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]
#![allow(clippy::needless_doctest_main)]

//! Lightweight and flexible command line argument parser with derive and combinator style API

//! # Quick start, derive edition
//!
//! 1. Add `bpaf` under `[dependencies]` in your `Cargo.toml`
//! ```toml
//! [dependencies]
//! bpaf = { version = "0.5", features = ["derive"] }
//! ```
//!
//! 2. Define a structure containing command line attributes and run generated function
//! ```no_run
//! use bpaf::{Bpaf, OptionParser};
//!
//! #[derive(Clone, Debug, Bpaf)]
//! #[bpaf(options, version)]
//! /// Accept speed and distance, print them
//! struct SpeedAndDistance {
//!     /// Speed in KPH
//!     speed: f64,
//!     /// Distance in miles
//!     distance: f64,
//! }
//!
//! fn main() {
//!     // #[derive(Bpaf) generates function speed_and_distance
//!     let opts = speed_and_distance().run();
//!     println!("Options: {:?}", opts);
//! }
//! ```
//!
//! 3. Try to run the app
//! ```console
//! % very_basic --help
//! Accept speed and distance, print them
//!
//! Usage: --speed ARG --distance ARG
//!
//! Available options:
//!         --speed <ARG>     Speed in KPH
//!         --distance <ARG>  Distance in miles
//!     -h, --help            Prints help information
//!     -V, --version         Prints version information
//!
//! % very_basic --speed 100
//! Expected --distance ARG, pass --help for usage information
//!
//! % very_basic --speed 100 --distance 500
//! Options: SpeedAndDistance { speed: 100.0, distance: 500.0 }
//!
//! % very_basic --version
//! Version: 0.5.0 (taken from Cargo.toml by default)
//!```

//! # Quick start, combinatoric edition
//!
//! 1. Add `bpaf` under `[dependencies]` in your `Cargo.toml`
//! ```toml
//! [dependencies]
//! bpaf = "0.5"
//! ```
//!
//! 2. Declare parsers for components, combine them and run it
//! ```no_run
//! use bpaf::{construct, long, OptionParser, Parser};
//! #[derive(Clone, Debug)]
//! struct SpeedAndDistance {
//!     /// Dpeed in KPH
//!     speed: f64,
//!     /// Distance in miles
//!     distance: f64,
//! }
//!
//! fn main() {
//!     // primitive parsers
//!     let speed = long("speed")
//!         .help("Speed in KPG")
//!         .argument("SPEED")
//!         .from_str::<f64>();
//!
//!     let distance = long("distance")
//!         .help("Distance in miles")
//!         .argument("DIST")
//!         .from_str::<f64>();
//!
//!     // parser containing information about both speed and distance
//!     let parser = construct!(SpeedAndDistance { speed, distance });
//!
//!     // option parser with metainformation attached
//!     let speed_and_distance
//!         = parser
//!         .to_options()
//!         .descr("Accept speed and distance, print them");
//!
//!     let opts = speed_and_distance.run();
//!     println!("Options: {:?}", opts);
//! }
//! ```
//!
//! 3. Try to run it, output should be similar to derive edition

//! # Getting started
//!
//! Combinatoric and derive APIs share the documentation and most of the names, recommended reading order:
//! 1. [`construct!`] - what combinations are and how you should read the examples
//! 2. [`Named`], [`positional`] and [`command`] - on consuming data
//! 3. [`Parser`] - on transforming the data
//! 4. [`OptionParser`] - on running the result
//! 5. *Using the library in derive style* section below

//! # Design goals: flexibility, reusability
//!
//! Library allows to express command line arguments by combining primitive parsers using mostly
//! regular Rust code plus one macro. For example it's possible to take a parser that requires a single
//! floating point number and transform it to a parser that takes several of them or takes it
//! optionally so different subcommands or binaries can share a lot of the code:
//!
//! ```no_run
//! use bpaf::*;
//!
//! // a regular function that doesn't depend on anything, you can export it
//! // and share across subcommands and binaries
//! fn speed() -> impl Parser<f64> {
//!     long("speed")
//!         .help("Speed in KPH")
//!         .argument("SPEED")
//!         .from_str::<f64>()
//! }
//!
//! // this parser accepts multiple `--speed` flags from a command line when used,
//! // collecting them into a vector
//! let multiple_args = speed().many(); // impl Parser<Vec<f64>>
//!
//! // this parser checks if `--speed` is present and uses value of 42 if it's not
//! let with_fallback = speed().fallback(42.0); // impl Parser<Option<f64>>
//! ```
//!
//! At any point you can apply additional validation or fallback values in terms of current parsed
//! state of each subparser and you can have several stages as well:
//!
//! ```no_run
//! use bpaf::*;
//!
//! #[derive(Clone, Debug)]
//! struct Speed(f64);
//!
//! long("speed")
//!     .help("Speed in KPH")
//!     .argument("SPEED")
//!     // After this point the type is `impl Parser<String>`
//!     .from_str::<f64>()
//!     // `from_str` uses FromStr trait to transform contained value into `f64`
//!
//!     // You can perform additional validation with `parse` and `guard` functions
//!     // in as many steps as required.
//!     // Before and after next two applications the type is still `impl Parser<f64>`
//!     .guard(|&speed| speed >= 0.0, "You need to buy a DLC to move backwards")
//!     .guard(|&speed| speed <= 100.0, "You need to buy a DLC to break the speed limits")
//!
//!     // You can transform contained values, next line gives `impl Parser<Speed>` as a result
//!     .map(|speed| Speed(speed));
//! ```

//! # Design goals: restrictions
//!
//! The main restricting library sets is that you can't use parsed values (but not the fact that
//! parser succeeded or failed) to decide how to parse subsequent values. In other words parsers
//! don't have the monadic strength, only the applicative one.
//!
//! To give an example, you can implement this description:
//!
//! > Program takes one of `--stdout` or `--file` flag to specify the output target, when it's `--file`
//! > program also requires `-f` attribute with the filename
//!
//! But not this one:
//!
//! > Program takes an `-o` attribute with possible values of `'stdout'` and `'file'`, when it's `'file'`
//! > program also requires `-f` attribute with the filename
//!
//! This set of restrictions allows to extract information about the structure of the computations
//! to generate help and overall results in less confusing enduser experience

//! # Design non goals: performance
//!
//! Library aims to optimize for flexibility, reusability and compilation time over runtime
//! performance which means it might perform some additional clones, allocations and other less
//! optimal things. In practice unless you are parsing tens of thousands of different parameters
//! and your app exits within microseconds - this won't affect you. That said - any actual
//! performance related problems with real world applications is a bug.

//! # Derive and combinatoric API
//!
//! Library supports both derive and combinatoric APIs whith combinatoric API being primary, it's
//! possible to mix and match both APIs at once. Both APIs provide access to mostly the same
//! features, some things are more convenient to do with derive (usually less typing), some -
//! with combinatoric (usually maximum flexibility and reducing boilerplate structs). In most cases
//! using just one would suffice. Whenever possible APIs share the same keywords and structure.
//! Documentation for combinatoric API also explains how to perform the same action in derive style
//! so you should read it.

//! # Using the library in combinatoric style
//!
//! 1. Define primitive field parsers using builder pattern starting with [`short`], [`long`],
//! [`command`] or [`positional`], add more information using [`help`](Named), [`env`](Named::env) and
//! other member functions.
//!
//!    For some constructors you end up with parser objects right away,
//!    some require finalization with [`argument`](Named::argument), [`flag`](Named::flag)
//!    or [`switch`](Named::switch).
//!
//!    At the end of this step you'll get one or more parser
//!    one or more objects implementing trait [`Parser`], such as `impl Parser<String>`.
//!
//! 2. If you need additional parsing and validation you can use trait [`Parser`]: [`map`](Parser::map),
//!    [`parse`](Parser::parse), [`guard`](Parser::guard), [`from_str`](Parser::from_str).
//!
//!    You can change type or shape of contained or shape with [`many`](Parser::many),
//!    [`some`](Parser::some), [`optional`](Parser::optional) and add a fallback values with
//!    [`fallback`](Parser::fallback), [`fallback_with`](Parser::fallback_with).
//!
//! 3. You can compose resulting primitive parsers using [`construct`] macro into a concrete
//!    datatype and still apply additional processing from step 2 after this.
//!
//! 4. Transform the toplevel parser created at the previous step into [`OptionParser`] with
//!    [`to_options`](Parser::to_options) and attach additional metadata with
//!    [`descr`](OptionParser::descr) and other methods available to `OptionParser`.
//!
//! 5. [`run`](OptionParser::run) the resulting option parser at the beginning of your program.
//!    If option parser succeeds you'll get the results. If there are errors or user asked for help info
//!    `bpaf` handles them and exits.

//! # Using the library in derive style
//!
//! 1. To use derive style API you need to enable `"derive"` feature for bpaf, **by default it's not
//!    enabled**.
//!
//! 2. Define primitive parsers if you want to use any. While it's possible to define most of them
//!    in derive style - doing complex parsing or validation is often easier in combinatoric style
//!
//! 3. Define types used to derive parsers, structs correspond to *AND* combination and require for
//!    all the fields to have a value, enums to *OR* combinations and require (and consume) all the
//!    values for one branch only.
//!
//! 4. Add annotations to the top level of a struct if needed, there's several to choose from and
//!    you can specify several of them. For this annotation ordering doesn't matter.
//!
//!    - Generated function name. Unlike usual derive macro bpaf generates a function with a name
//!      derived from a struct name by transforming it from `CamelCase` to `snake_case`. `generate`
//!      allows to override a name for the function
//!
//!    ```rust
//!    use bpaf::*;
//!
//!    #[derive(Debug, Clone, Bpaf)]
//!    #[bpaf(generate(make_config))] // function name is now make_config()
//!    pub struct Config {
//!        pub flag: bool
//!    }
//!    ```
//!
//!    - Generated function visibility. By default bpaf uses the same visibility as the datatype,
//!      `private` makes it module private:
//!
//!    ```rust
//!    use bpaf::*;
//!
//!    #[derive(Debug, Clone, Bpaf)]
//!    #[bpaf(private)] // config() is now private
//!    pub struct Config {
//!        pub flag: bool
//!    }
//!    ```
//!
//!    - Generated function type. By default bpaf would generate a function that parses
//!      all the fields present (`impl` [`Parser`]), it's possible instead to turn it into a
//!      one or more [`command`] with or top level `impl` [`OptionParser`] with `options`.
//!      Those annotations are mutually exclusive. `options` annotation takes an optional argument
//!      to wrap options into [`cargo_helper`], `command` annotation takes an optional argument to
//!      override a command name.
//!
//!    ```rust
//!    use bpaf::*;
//!
//!    #[derive(Debug, Clone, Bpaf)]
//!    pub struct Flag { // impl Parser by default
//!        pub flag: bool
//!    }
//!
//!    #[derive(Debug, Clone, Bpaf)]
//!    #[bpaf(command)]
//!    pub struct Make { // generates a command "make"
//!        pub level: u32,
//!    }
//!
//!
//!    #[derive(Debug, Clone, Bpaf)]
//!    #[bpaf(options)] // config() is now private
//!    pub struct Config {
//!        pub flag: bool
//!    }
//!    ```
//!
//!    - Specify version for generated command. By default bpaf would use version as defined by
//!      `"CARGO_PKG_VERSION"` env variable during compilation, usually taken from `Cargo.toml`,
//!      it's possible to override it with a custom expression. Only makes sense for `command`
//!      and `options` annotations. For more information see [`version`](OptionParser::version).
//!
//!    ```rust
//!    use bpaf::*;
//!
//!    #[derive(Debug, Clone, Bpaf)]
//!    #[bpaf(options, version("3.1415"))] // --version is now 3.1415
//!    pub struct Config {
//!        pub flag: bool
//!    }
//!    ```
//!
//! 5. Add annotations to individual fields. Structure for annotation for individual fields
//!    is similar to how you would write the same code with combinatoric API with exception
//!    of `external` and usually looks something like this:
//!
//!    `((<naming> <consumer>) | <external>) <postprocessing>`
//!
//!    - `naming` section corresponds to [`short`],  [`long`] and [`env`](env()). `short` takes an optional
//!      character literal as a parameter, `long` takes an optional string.
//!
//!      + If parameter for `short`/`long` is parameter isn't present it's derived from the field
//!      name: first character and a whole name respectively.
//!
//!      + If either of `short` or `long` is present - bpaf would not add the other one.
//!
//!      + If neither is present - bpaf would add a long one.
//!
//!      + `env` takes an arbitrary expression of type `&'static str` - could be a string literal or a constant.
//!
//!      ```rust
//!      # use bpaf::*;
//!      const DB: &str = "top_secret_database";
//!
//!      #[derive(Debug, Clone, Bpaf)]
//!      pub struct Config {
//!         pub flag_1: bool,     // no annotation: --flag_1
//!
//!         #[bpaf(short)]
//!         pub flag_2: bool,     // explicit short suppresses long: -f
//!
//!         #[bpaf(short('z'))]
//!         pub flag_3: bool,     // explicit short with custom letter: -z
//!
//!         #[bpaf(short, long)]
//!         pub deposit: bool,    // explicit short and long: -d --deposit
//!
//!         #[bpaf(env(DB))]
//!         pub database: String, // --database + env variable from DB constant
//!
//!         #[bpaf(env("USER"))]  // --user + env variable "USER"
//!         pub user: String,
//!      }
//!      ```
//!
//!    - `consumer` section corresponds to [`argument`](Named::argument), [`positional`],
//!      [`flag`](Named::flag), [`switch`](Named::switch) and similar.
//!
//!      + With no consumer annotations tuple structs (`struct Config(String)`) are usually parsed
//!      as positional items, but it's possible to override it by giving it a name:
//!
//!      ```rust
//!      # use bpaf::*;
//!      # use std::path::PathBuf;
//!
//!      #[derive(Debug, Clone, Bpaf)]
//!      struct Opt(PathBuf); // stays positional
//!
//!      #[derive(Debug, Clone, Bpaf)]
//!      struct Config(#[bpaf(long("input"))] PathBuf); // turns into a named argument
//!      ```
//!
//!      + `bpaf_derive` handles fields of type `Option<Foo>` and `Vec<Foo>` with something
//!      that can consume possibly one or many items with [`optional`](Parser::optional)
//!      and [`many`](Parser::many) respectively, see `postprocessing` for more details.
//!
//!      + `bpaf_derive` handles `bool` fields with [`switch`](Named::switch),
//!      [`OsString`](std::ffi::OsString) and [`PathBuf`](std::path::PathBuf) with
//!      either [`positional_os`] or [`argument_os`](Named::argument_os).
//!
//!      + `bpaf_derive` consumes everything else as [`String`] with [`positional`] and
//!      [`argument`](Named::argument) and transforms it into a concrete type using
//!      [`FromStr`](std::str::FromStr) instance.
//!      See documentation for corresponding consumers for more details.
//!
//!    - If `external` is present - it usually serves function of `naming` and `consumer`, allowing
//!      more for `postprocessing` annotations after it. Takes an optional parameter - a function
//!      name to call, if not present - `bpaf_derive` uses field name for this purpose.
//!      Functions should return impl [`Parser`] and you can either declare them manually
//!      or derive with `Bpaf` macro.
//!
//!      ```rust
//!      use bpaf::*;
//!
//!      fn verbosity() -> impl Parser<usize> {
//!          short('v')
//!              .help("vebosity, can specify multiple times")
//!              .req_flag(())
//!              .many()
//!              .map(|x| x.len())
//!      }
//!
//!      #[derive(Debug, Clone, Bpaf)]
//!      pub struct Username {
//!          pub user: String
//!      }
//!
//!      #[derive(Debug, Clone, Bpaf)]
//!      pub struct Config {
//!         #[bpaf(external)]
//!         pub verbosity: usize,      // implicit name - "verbosity"
//!
//!         #[bpaf(external(username))]
//!         pub custom_user: Username, // explicit name - "username"
//!      }
//!      ```
//!
//!    - `postprocessing` - what it says, various methods from [`Parser`] trait, order matters,
//!    most of them are taken literal, see documentation for the trait for more details. usually
//!    bpaf can derive what to use here depending on a type: `Option<T>`, `Vec<T>` are supported as
//!    is, everything else is assumed to be [`FromStr`](std::str::FromStr). If you put anything
//!    in the postprocessing section it will disable this logic and you will need to spell out
//!    the whole transformation chain.
//!
//!    - field-less enum variants obey slightly different set of rules, see
//!    [`req_flag`](Named::req_flag) for more details.
//!
//!
//! 6. Add documentation for help messages.
//!    Help messages are generated from doc comments, bpaf skips single empty lines and stops
//!    processing after double empty line:
//!
//!    ```rust
//!    use bpaf::*;
//!    #[derive(Debug, Clone, Bpaf)]
//!    pub struct Username {
//!        /// this is a part of a help message
//!        ///
//!        /// so is this
//!        ///
//!        ///
//!        /// but this isn't
//!        pub user: String
//!    }
//!    ```

//! # More examples
//!
//! A bunch more examples can be found here: <https://github.com/pacak/bpaf/tree/master/examples>
//!
//! They are usually documented and you can see how they work by cloning the repo and running
//!
//! ```shell
//! $ cargo run --example example_name
//! ```

//! # Testing your own parsers
//!
//! You can test your own parsers to maintain compatibility or simply checking expected output
//! with [`run_inner`](OptionParser::run_inner)
//!
//! ```rust
//! # use bpaf::*;
//!
//! #[derive(Debug, Clone, Bpaf)]
//! #[bpaf(options)]
//! pub struct Options {
//!     pub user: String
//! }
//!
//! let help = options()
//!     .run_inner(Args::from(&["--help"]))
//!     .unwrap_err()
//!     .unwrap_stdout();
//!
//! // assert_eq!(help, ...)
//! # drop(help);
//! ```

use std::marker::PhantomData;

mod params;

mod args;

#[doc(hidden)]
pub mod info;
#[doc(hidden)]
mod item;
#[doc(hidden)]
mod meta;

pub mod structs;
use crate::{info::Error, item::Item};
use info::OptionParserStruct;

use structs::{
    ParseFail, ParseFallback, ParseFallbackWith, ParseFromStr, ParseGroupHelp, ParseGuard,
    ParseHide, ParseMany, ParseMap, ParseOptional, ParseOrElse, ParsePure, ParseSome, ParseWith,
};

#[cfg(test)]
mod tests;
#[doc(inline)]
pub use crate::args::Args;
pub use crate::info::OptionParser;
pub use crate::meta::Meta;

#[doc(inline)]
pub use crate::params::{
    command, env, long, positional, positional_if, positional_os, short, Command, Named,
};

#[doc(inline)]
#[cfg(feature = "bpaf_derive")]
pub use bpaf_derive::Bpaf;

/// Compose several parsers to produce a single result
///
/// # Combinatoric usage, types of composition
/// `construct!` can compose parsers sequentially or in parallel.
///
/// Sequential composition runs each parser and if all of them succeeds you get a parser object of
/// a new type back. This new type could be struct or enum with named or unnamed fields or a tuple.
/// Placeholder names for values inside `construct!` macro must correspond to both struct/enum
/// names and parser names present in scope. In examples below `a` corresponds to a function and
/// `b` corresponds to a variable name.
///
/// ```rust
/// # use bpaf::*;
/// struct Res (u32, u32);
/// enum Ul { T { a: u32, b: u32 } }
///
/// // parameters can be shared across multiple construct invocations
/// // if defined as functions
/// fn a() -> impl Parser<u32> {
///     short('a').argument("N").from_str::<u32>()
/// }
///
/// // you can construct structs or enums with unnamed fields
/// fn res() -> impl Parser<Res> {
///     let b = short('b').argument("n").from_str::<u32>();
///     construct!(Res ( a(), b ))
/// }
///
/// // you can construct structs or enums with named fields
/// fn ult() -> impl Parser<Ul> {
///     let b = short('b').argument("n").from_str::<u32>();
///     construct!(Ul::T { a(), b })
/// }
///
/// // you can also construct simple tuples
/// fn tuple() -> impl Parser<(u32, u32)> {
///     let b = short('b').argument("n").from_str::<u32>();
///     construct!(a(), b)
/// }
/// ```
///
/// Parallel composition picks one of several available parsers and returns a parser object of the
/// same type. Similar to sequential composition you can use parsers from variables or functions:
///
/// ```rust
/// # use bpaf::*;
/// fn b() -> impl Parser<u32> {
///     short('b').argument("NUM").from_str::<u32>()
/// }
///
/// fn a_or_b() -> impl Parser<u32> {
///     let a = short('a').argument("NUM").from_str::<u32>();
///     // equivalent way of writing this would be `a.or_else(b())`
///     construct!([a, b()])
/// }
/// ```
///
/// # Derive API considerations
///
/// `bpaf_derive` would combine fields of struct or enum constructors sequentially and enum
/// variants in parallel. For enums with variants containing more than one field it's better to
/// represent them as commands: [`command`].
/// ```rust
/// # use bpaf::*;
/// // to satisfy this parser user needs to pass both -a and -b
/// #[derive(Debug, Clone, Bpaf)]
/// struct Res {
///     a: u32,
///     b: u32,
/// }
///
/// // to satisfy this parser user needs to pass one (and only one) of -a, -b, -c or -d
/// #[derive(Debug, Clone, Bpaf)]
/// enum Okay {
///     A { a: u32 },
///     B { b: u32 },
///     C { c: u32 },
///     D { d: u32 },
/// }
///
/// // here user needs to pass either both -a AND -b or both -c and -d
/// #[derive(Debug, Clone, Bpaf)]
/// enum Ult {
///     AB { a: u32, b: u32 },
///     CD { c: u32, d: u32 }
/// }
/// ```
///
/// # Examples considerations
///
/// Most of the examples declare parser as a top level function, this is done only to be able to
/// specify the type signature, you can still use them as variables,  see `a` and `b` in examples above.
///
/// Most of the examples given in the documentation are more verbose than necessary preferring
/// explicit naming and consumers. If you are trying to parse something that implements
/// [`FromStr`](std::str::FromStr), only interested in a long name and don't mind metavar being
/// `ARG` you don't need to add any extra annotations at all:
///
/// ```rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// struct PerfectlyValid {
///     /// number used by the program
///     number: u32,
/// }
/// ```
///
/// Toplevel types also require `options` annotation to generate [`OptionParser`] - it's usually
/// omitted:
///
/// ```rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// #[bpaf(options)] // <- important bit
/// struct Config {
///     /// number used by the program
///     number: u32,
/// }
/// ```
///
/// For combinatoric examples [`help`](Named::help) is usually omitted - you shouldn't do that.
///
/// For combinatoric examples usually implemented type is [`Parser`], to be able to run it you need to
/// add metainformation to get [`OptionParser`].
///
/// In addition to examples in the documentation there's a bunch more in the github repository:
/// <https://github.com/pacak/bpaf/tree/master/examples>

#[macro_export]
macro_rules! construct {
    // construct!(Enum::Cons { a, b, c })
    ($ns:ident $(:: $con:ident)* { $($tokens:tt)* }) => {{ $crate::construct!(@prepare [named [$ns $(:: $con)*]] [] $($tokens)*) }};
    (:: $ns:ident $(:: $con:ident)* { $($tokens:tt)* }) => {{ $crate::construct!(@prepare [named [:: $ns $(:: $con)*]] [] $($tokens)*) }};
    // construct!(Enum::Cons ( a, b, c ))
    ($ns:ident $(:: $con:ident)* ( $($tokens:tt)* )) => {{ $crate::construct!(@prepare [pos [$ns $(:: $con)*]] [] $($tokens)*) }};
    (:: $ns:ident $(:: $con:ident)* ( $($tokens:tt)* )) => {{ $crate::construct!(@prepare [pos [:: $ns $(:: $con)*]] [] $($tokens)*) }};

    // construct!( a, b, c )
    ($first:ident , $($tokens:tt)*) => {{ $crate::construct!(@prepare [pos] [] $first , $($tokens)*) }};
    ($first:ident (), $($tokens:tt)*) => {{ $crate::construct!(@prepare [pos] [] $first (), $($tokens)*) }};

    // construct![a, b, c]
    ([$first:ident $($tokens:tt)*]) => {{ $crate::construct!(@prepare [alt] [] $first $($tokens)*) }};

    (@prepare $ty:tt [$($fields:tt)*] $field:ident (), $($rest:tt)*) => {{
        let $field = $field();
        $crate::construct!(@prepare $ty [$($fields)* $field] $($rest)*)
    }};
    (@prepare $ty:tt [$($fields:tt)*] $field:ident () $($rest:tt)*) => {{
        let $field = $field();
        $crate::construct!(@prepare $ty [$($fields)* $field] $($rest)*)
    }};
    (@prepare $ty:tt [$($fields:tt)*] $field:ident, $($rest:tt)*) => {{
        $crate::construct!(@prepare $ty [$($fields)* $field] $($rest)*)
    }};
    (@prepare $ty:tt [$($fields:tt)*] $field:ident $($rest:tt)*) => {{
        $crate::construct!(@prepare $ty [$($fields)* $field] $($rest)*)
    }};

    (@prepare [alt] [$first:ident $($fields:ident)*]) => {{
        use $crate::Parser; $first $(.or_else($fields))*
    }};

    (@prepare $ty:tt [$($fields:tt)*]) => {{
        use $crate::Parser;
        let meta = $crate::Meta::And(vec![ $($fields.meta()),* ]);
        let inner = move |args: &mut $crate::Args| {
            $(let $fields = $fields.eval(args)?;)*
            ::std::result::Result::Ok::<_, $crate::info::Error>
                ($crate::construct!(@make $ty [$($fields)*]))
        };
        $crate::structs::ParseConstruct { inner, meta }
    }};

    (@make [named [$($con:tt)+]] [$($fields:ident)*]) => { $($con)+ { $($fields),* } };
    (@make [pos   [$($con:tt)+]] [$($fields:ident)*]) => { $($con)+ ( $($fields),* ) };
    (@make [pos] [$($fields:ident)*]) => { ( $($fields),* ) };
}

/// Simple or composed argument parser
///
/// # Overview
///
/// it's best to think of an object implementing [`Parser`] trait as a container with a value
/// inside that can be composed with other `Parser` containers using [`construct!`] and the only
/// way to extract this value is by transforming it to [`OptionParser`] with
/// [`to_options`](Parser::to_options) and running it with [`run`](OptionParser::run). At which
/// point you will either get your value out or bpaf would generate a message describing a problem
/// (missing argument, validation failure, user requested CLI help, etc) and the program would
/// exit.
///
/// Values inside can be of any type for as long as they implement `Debug`, can be cloned and
/// there's no lifetimes other than static.
///
/// When consuming the values you usually start with `Parser<String>` or `Parser<OsString>` which
/// you then transform into something that your program would actually use. it's better to perform
/// as much parsing and validation inside the `Parser` as possible so the program itself gets
/// strictly typed and correct value while user gets immediate feedback on what's wrong with the
/// arguments they pass.
///
/// For example suppose your program needs user to specify a dimensions of a rectangle, with sides
/// being 1..20 units long and the total area must not exceed 200 units square. A parser that
/// consumes it might look like this:
///
/// ```rust
/// # use bpaf::*;
/// #[derive(Debug, Copy, Clone)]
/// struct Rectangle {
///     width: u32,
///     height: u32,
/// }
///
/// fn rectangle() -> impl Parser<Rectangle> {
///     let invalid_size = "Sides of a rectangle must be 1..20 units long";
///     let invalid_area = "Area of a rectangle must not exceed 200 units square";
///     let width = long("width")
///         .help("Width of the rectangle")
///         .argument("PX")
///         .from_str::<u32>()
///         .guard(|&x| 1 <= x && x <= 10, invalid_size);
///     let height = long("height")
///         .help("Height of the rectangle")
///         .argument("PX")
///         .from_str::<u32>()
///         .guard(|&x| 1 <= x && x <= 10, invalid_size);
///     construct!(Rectangle { width, height })
///         .guard(|&r| r.width * r.height <= 400, invalid_area)
/// }
/// ```
///
///
/// # Derive specific considerations
///
/// Every method defined on this trait belongs to the `postprocessing` section of the field
/// annotation. `bpaf_derive` would try to figure out what chain to use for as long as there's no
/// options changing the type: you can use [`fallback`](Parser::fallback_with),
/// [`fallback_with`](Parser::fallback_with), [`guard`](Parser::guard), [`hide`](Parser::hide`) and
/// [`group_help`](Parser::group_help) but not the rest of them.
///
/// ```rust
/// # use bpaf::*;
/// #[derive(Debug, Clone, Bpaf)]
/// struct Options {
///     // no annotation at all - implicit `argument` and `from_str` are inserted
///     number_1: u32,
///
///     // fallback isn't changing the type so implicit items are still added
///     #[bpaf(fallback(42))]
///     number_2: u32,
///
///     // implicit `argument`, `optional` and `from_str` are inserted
///     number_3: Option<u32>,
///
///     // fails to compile: you need to specify a consumer, `argument` or `argument_os`
///     // #[bpaf(optional)]
///     // number_4: Option<u32>
///
///     // fails to compile: you also need to specify how to go from String to u32
///     // #[bpaf(argument("N"), optional)]
///     // number_5: Option<u32>,
///
///     // explicit consumer and a full postprocessing chain
///     #[bpaf(argument("N"), from_str(u32), optional)]
///     number_6: Option<u32>,
/// }
/// ```
pub trait Parser<T> {
    /// Evaluate inner function
    ///
    /// Mostly internal implementation details, you can try using it to test your parsers
    // it's possible to move this function from the trait to the structs but having it
    // in the trait ensures the composition always works: structs will have to implement it
    #[doc(hidden)]
    fn eval(&self, args: &mut Args) -> Result<T, Error>;

    /// Included information about the parser
    ///
    /// Mostly internal implementation details, you can try using it to test your parsers
    // it's possible to move this function from the trait to the structs but having it
    // in the trait ensures the composition always works: structs will have to implement it
    #[doc(hidden)]
    fn meta(&self) -> Meta;

    // change shape
    // {{{ many
    /// Consume zero or more items from a command line and collect them into [`Vec`]
    ///
    /// # Combinatoric usage:
    /// ```rust
    /// # use bpaf::*;
    /// fn numbers() -> impl Parser<Vec<u32>> {
    ///     short('n')
    ///         .argument("NUM")
    ///         .from_str::<u32>()
    ///         .many()
    /// }
    /// ```
    ///
    /// # Derive usage:
    /// Bpaf would insert implicit `many` when resulting type is a vector
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///     #[bpaf(short, argument("NUM"))]
    ///     numbers: Vec<u32>
    /// }
    /// ```
    /// But it's also possible to specify it explicitly, both cases renerate the same code.
    /// Note, since using `many` resets the postprocessing chain - you also need to specify
    /// [`from_str`](Parser::from_str)
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///     #[bpaf(short, argument("NUM"), from_str(u32), many)]
    ///     numbers: Vec<u32>
    /// }
    /// ```
    ///
    ///
    /// # Example
    /// ```console
    /// $ app -n 1 -n 2 -n 3
    /// // [1, 2, 3]
    /// ```
    ///
    /// # Panics
    /// Panics if parser succeeds without consuming any input: any parser modified with
    /// `many` must consume something: trying to parse `many` [`flag`](Named::flag) or
    /// [`switch`](Named::switch) would cause this panic, instead you should use
    /// [`req_flag`](Named::req_flag).
    ///
    /// # See also
    /// [`some`](Parser::some) also collects results to a vector but requires at least one
    /// element to succeed
    fn many(self) -> ParseMany<Self>
    where
        Self: Sized,
    {
        ParseMany { inner: self }
    }
    // }}}

    // {{{ some
    /// Consume one or more items from a command line
    ///
    /// Takes a string that will be used as an
    /// error message if there's no specified parameters
    ///
    /// # Combinatoric usage:
    /// ```rust
    /// # use bpaf::*;
    /// let numbers
    ///     = short('n')
    ///     .argument("NUM")
    ///     .from_str::<u32>()
    ///     .some("Need at least one number");
    /// # drop(numbers);
    /// ```
    ///
    /// # Derive usage
    /// Since using `some` resets the postprocessing chain - you also need to specify
    /// [`from_str`](Parser::from_str) or similar, depending on your type
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///     #[bpaf(short, argument("NUM"), from_str(u32), some("Need at least one number"))]
    ///     numbers: Vec<u32>
    /// }
    /// ```
    ///
    ///
    /// # Example
    /// ```console
    /// $ app
    /// // fails with "Need at least one number"
    /// $ app -n 1 -n 2 -n 3
    /// // [1, 2, 3]
    /// ```
    ///
    /// # Panics
    /// Panics if parser succeeds without consuming any input: any parser modified with
    /// `some` must consume something: trying to parse `many` [`flag`](Named::flag) or
    /// [`switch`](Named::switch) would cause this panic, instead you should use
    /// [`req_flag`](Named::req_flag).
    ///
    /// # See also
    /// [`many`](Parser::many) also collects results to a vector and will succeed with
    /// no matching values
    #[must_use]
    fn some(self, message: &'static str) -> ParseSome<Self>
    where
        Self: Sized + Parser<T>,
    {
        ParseSome {
            inner: self,
            message,
        }
    }
    // }}}

    // {{{ optional
    /// Turn a required parser into optional
    ///
    /// Any failure inside the parser will be turned into `None`. Failures in parser usually come
    /// from missing a command line argument, but it's possible to introduce them with
    /// [`parse`](Parser::parse) or [`guard`](Parser::guard) methods.
    ///
    /// # Combinatoric usage
    /// ```rust
    /// # use bpaf::*;
    /// fn number() -> impl Parser<Option<u32>> {
    ///     short('n')
    ///         .argument("NUM")
    ///         .from_str::<u32>()
    ///         .optional()
    /// }
    /// ```
    ///
    /// # Derive usage
    ///
    /// By default bpaf would automatically use optional for fields of type `Option<T>`,
    /// for as long as it's not prevented from doing so by present postprocessing options
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///    #[bpaf(short, argument("NUM"))]
    ///    number: Option<u32>
    /// }
    /// ```
    ///
    /// But it's also possible to specify it explicitly, in which case you need to specify
    /// a full postprocessing chain which starts from [`from_str`](Parser::from_str) in this
    /// example.
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///    #[bpaf(short, argument("NUM"), from_str(u32), optional)]
    ///    number: Option<u32>
    /// }
    /// ```
    ///
    /// # Example
    /// ```console
    /// $ app
    /// // None
    /// $ app -n 42
    /// // Some(42)
    /// ```
    #[must_use]
    fn optional(self) -> ParseOptional<Self>
    where
        Self: Sized + Parser<T>,
    {
        ParseOptional { inner: self }
    }
    // }}}

    // parse
    // {{{ parse
    /// Apply a failing transformation to a contained value
    ///
    /// This is a most general way of transforming parsers and remaining ones can be expressed in
    /// terms of it: [`map`](Parser::map), [`from_str`](Parser::from_str) and
    /// [`guard`](Parser::guard).
    ///
    /// Examples given here are a bit artificail, to parse a value from string you should use
    /// [`from_str`](Parser::from_str).
    ///
    /// # Combinatoric usage:
    /// ```rust
    /// # use bpaf::*;
    /// # use std::str::FromStr;
    /// fn number() -> impl Parser<u32> {
    ///     short('n')
    ///         .argument("NUM")
    ///         .parse(|s| u32::from_str(&s))
    /// }
    /// ```
    /// # Derive usage:
    /// `parse` takes a single parameter: function name to call. Function type should match
    /// parameter `F` used by `parse` in combinatoric API.
    /// ```rust
    /// # use bpaf::*;
    /// # use std::str::FromStr;
    /// # use std::num::ParseIntError;
    /// fn read_number(s: String) -> Result<u32, ParseIntError> {
    ///     u32::from_str(&s)
    /// }
    ///
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///     #[bpaf(short, argument("NUM"), parse(read_number))]
    ///     number: u32
    /// }
    /// ```
    ///
    /// # Example
    /// ```console
    /// $ app -n 12
    /// // 12
    /// # app -n pi
    /// // fails with "Couldn't parse "pi": invalid numeric literal"
    /// ```
    ///
    fn parse<F, R, E>(self, f: F) -> ParseWith<T, Self, F, E, R>
    where
        Self: Sized + Parser<T>,
        F: Fn(T) -> Result<R, E>,
        E: ToString,
    {
        ParseWith {
            inner: self,
            inner_res: PhantomData,
            parse_fn: f,
            res: PhantomData,
            err: PhantomData,
        }
    }
    // }}}

    // {{{ map
    /// Apply a pure transformation to a contained value
    ///
    /// A common case of [`parse`](Parser::parse) method, exists mostly for convenience.
    ///
    /// # Combinatoric usage
    /// ```rust
    /// # use bpaf::*;
    /// fn number() -> impl Parser<u32> {
    ///     short('n')
    ///         .argument("NUM")
    ///         .from_str::<u32>()
    ///         .map(|v| v * 2)
    /// }
    /// ```
    ///
    /// # Derive usage
    /// ```rust
    /// # use bpaf::*;
    /// fn double(i: u32) -> u32 {
    ///     i * 2
    /// }
    ///
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///     #[bpaf(short, argument("NUM"), from_str(u32), map(double))]
    ///     number: u32,
    /// }
    /// ```
    ///
    /// # Example
    /// ```console
    /// $ app -n 21
    /// // 42
    /// ```
    fn map<F, R>(self, map: F) -> ParseMap<T, Self, F, R>
    where
        Self: Sized + Parser<T>,
        F: Fn(T) -> R + 'static,
    {
        ParseMap {
            inner: self,
            inner_res: PhantomData,
            map_fn: map,
            res: PhantomData,
        }
    }
    // }}}

    // {{{ from_str
    /// Parse stored [`String`] using [`FromStr`](std::str::FromStr) instance
    ///
    /// A common case of [`parse`](Parser::parse) method, exists mostly for convenience.
    ///
    /// # Combinatoric usage
    /// ```rust
    /// # use bpaf::*;
    /// fn speed() -> impl Parser<f64> {
    ///     short('s')
    ///         .argument("SPEED")
    ///         .from_str::<f64>()
    /// }
    /// ```
    ///
    /// # Derive usage
    /// By default `bpaf_derive` would use [`from_str`](Parser::from_str) for any time it's not
    /// familiar with so you don't need to specify anything
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///     #[bpaf(short, argument("SPEED"))]
    ///     speed: f64
    /// }
    /// ```
    ///
    /// But it's also possible to specify it explicitly
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///     #[bpaf(short, argument("SPEED"), from_str(f64))]
    ///     speed: f64
    /// }
    /// ```
    ///
    /// # Example
    /// ```console
    /// $ app -s pi
    /// // fails with "Couldn't parse "pi": invalid float literal"
    /// $ app -s 3.1415
    /// // Version: 3.1415
    /// ```
    ///
    /// # See also
    /// Other parsing and restricting methods include [`parse`](Parser::parse) and
    /// [`guard`](Parser). For transformations that can't fail you can use [`map`](Parser::map).
    #[must_use]
    #[allow(clippy::wrong_self_convention)]
    fn from_str<R>(self) -> ParseFromStr<Self, R>
    where
        Self: Sized + Parser<T>,
    {
        ParseFromStr {
            inner: self,
            ty: PhantomData,
        }
    }
    // }}}

    // {{{ guard
    /// Validate or fail with a message
    ///
    /// Parser will reject values that fail to satisfy the constraints
    ///
    /// # Combinatoric usage
    ///
    /// ```rust
    /// # use bpaf::*;
    /// fn number() -> impl Parser<u32> {
    ///     short('n')
    ///         .argument("NUM")
    ///         .from_str::<u32>()
    ///         .guard(|n| *n <= 10, "Values greater than 10 are only available in the DLC pack!")
    /// }
    /// ```
    ///
    /// # Derive usage
    /// Unlike combinator counterpart, derive variant of `guard` takes a function name instead
    /// of a closure, mostly to keep thing clean. Second argument can be either a string literal
    /// or a constant name for a static [`str`].
    ///
    /// ```rust
    /// # use bpaf::*;
    /// fn dlc_check(number: &u32) -> bool {
    ///     *number <= 10
    /// }
    ///
    /// const DLC_NEEDED: &str = "Values greater than 10 are only available in the DLC pack!";
    ///
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///     #[bpaf(short, argument("NUM"), guard(dlc_check, DLC_NEEDED))]
    ///     number: u32,
    /// }
    /// ```
    ///
    /// # Example
    /// ```console
    /// $ app -n 100
    /// // fails with "Values greater than 10 are only available in the DLC pack!"
    /// $ app -n 5
    /// // 5
    /// ```
    #[must_use]
    fn guard<F>(self, check: F, message: &'static str) -> ParseGuard<Self, F>
    where
        Self: Sized + Parser<T>,
        F: Fn(&T) -> bool,
    {
        ParseGuard {
            inner: self,
            check,
            message,
        }
    }
    // }}}

    // combine
    // {{{ fallback
    /// Use this value as default if value isn't present on a command line
    ///
    /// Parser would still fail if value is present but failure comes from some transformation
    ///
    /// # Combinatoric usage
    /// ```rust
    /// # use bpaf::*;
    /// fn number() -> impl Parser<u32> {
    ///     short('n')
    ///         .argument("NUM")
    ///         .from_str::<u32>()
    ///         .fallback(42)
    /// }
    /// ```
    ///
    /// # Derive usage
    /// Expression in parens should have the right type, this example uses `u32` literal,
    /// but it can also be your own type if that is what you are parsing, it can also be a function
    /// call.
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///    #[bpaf(short, argument("NUM"), from_str(u32), fallback(42))]
    ///    number: u32
    /// }
    /// ```
    ///
    /// # Example
    /// ```console
    /// $ app -n 100
    /// // 10
    /// $ app
    /// // 42
    /// $ app -n pi
    /// // fails with "Couldn't parse "pi": invalid numeric literal"
    /// ```
    ///
    /// # See also
    /// [`fallback_with`](Parser::fallback_with) would allow to try to fallback to a value that
    /// comes from a failing computation such as reading a file.
    #[must_use]
    fn fallback(self, value: T) -> ParseFallback<Self, T>
    where
        Self: Sized + Parser<T>,
    {
        ParseFallback { inner: self, value }
    }
    // }}}

    // {{{ fallback_with
    /// Use value produced by this function as default if value isn't present
    ///
    /// Would still fail if value is present but failure comes from some earlier transformation
    ///
    /// # Combinatoric usage
    /// ```rust
    /// # use bpaf::*;
    /// fn username() -> impl Parser<String> {
    ///     long("user")
    ///         .argument("USER")
    ///         .fallback_with::<_, Box<dyn std::error::Error>>(||{
    ///             let output = std::process::Command::new("whoami")
    ///                 .stdout(std::process::Stdio::piped())
    ///                 .spawn()?
    ///                 .wait_with_output()?
    ///                 .stdout;
    ///             Ok(std::str::from_utf8(&output)?.to_owned())
    ///         })
    /// }
    /// ```
    ///
    /// # Derive usage
    /// ```rust
    /// # use bpaf::*;
    /// fn get_current_user() -> Result<String, Box<dyn std::error::Error>> {
    ///     let output = std::process::Command::new("whoami")
    ///         .stdout(std::process::Stdio::piped())
    ///         .spawn()?
    ///         .wait_with_output()?
    ///         .stdout;
    ///     Ok(std::str::from_utf8(&output)?.to_owned())
    /// }
    ///
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///     #[bpaf(long, argument("USER"), fallback_with(get_current_user))]
    ///     user: String,
    /// }
    /// ```
    ///
    /// # Example
    /// ```console
    /// $ app --user bobert
    /// // "bobert"
    /// $ app
    /// // "pacak"
    /// ```
    ///
    /// # See also
    /// [`fallback`](Parser::fallback) implements similar logic expect that failures
    /// are not expected.
    #[must_use]
    fn fallback_with<F, E>(self, fallback: F) -> ParseFallbackWith<T, Self, F, E>
    where
        Self: Sized + Parser<T>,
        F: Fn() -> Result<T, E>,
        E: ToString,
    {
        ParseFallbackWith {
            inner: self,
            inner_res: PhantomData,
            fallback,
            err: PhantomData,
        }
    }
    // }}}

    // {{{ or_else
    /// If first parser fails - try the second one
    ///
    /// For parser to succeed eiter of the components needs to succeed. If both succeed - bpaf
    /// would use output from one that consumed the left most value. The second flag on the command
    /// line will remain unconsumed by `or_else` and needs to be consumed by something else,
    /// otherwise this will result in an error.
    ///
    /// # Combinatoric usage:
    /// There's two ways to write this combinator with identical results:
    /// ```rust
    /// # use bpaf::*;
    /// fn a() -> impl Parser<u32> {
    ///     short('a').argument("NUM").from_str::<u32>()
    /// }
    ///
    /// fn b() -> impl Parser<u32> {
    ///     short('b').argument("NUM").from_str::<u32>()
    /// }
    ///
    /// fn a_or_b_comb() -> impl Parser<u32> {
    ///     construct!([a(), b()])
    /// }
    ///
    /// fn a_or_b_comb2() -> impl Parser<u32> {
    ///     a().or_else(b())
    /// }
    /// ```
    ///
    /// # Example
    /// ```console
    /// $ app -a 12 -b 3
    /// // 12
    /// $ app -b 3 -a 12
    /// // 3
    /// $ app -b 13
    /// // 13
    /// $ app
    /// // fails asking for either -a NUM or -b NUM
    /// ```
    ///
    /// # Derive usage:
    ///
    /// enums are translated into alternative combinations, different shapes of variants
    /// produce different results
    ///
    ///
    /// ```bpaf
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// enum Flag {
    ///     A { a: u32 }
    ///     B { b: u32 }
    /// }
    /// ```
    ///
    /// ```console
    /// $ app -a 12 -b 3
    /// // Flag::A { a: 12 }
    /// $ app -b 3 -a 12
    /// // Flag::B { b: 3 }
    /// $ app -b 3
    /// // Flag::B { b: 3 }
    /// $ app
    /// // fails asking for either -a NUM or -b NUM
    /// ```
    ///
    /// # Performance
    ///
    /// If first parser succeeds - second one will be called anyway to produce a
    /// better error message for combinations of mutually exclusive parsers:
    /// Suppose program accepts one of two mutually exclusive switches `-a` and `-b`
    /// and both are present error message should point at the second flag
    fn or_else<P>(self, alt: P) -> ParseOrElse<Self, P>
    where
        Self: Sized + Parser<T>,
        P: Sized + Parser<T>,
    {
        ParseOrElse {
            this: self,
            that: alt,
        }
    }
    // }}}

    // misc
    // {{{ hide
    /// Ignore this parser during any sort of help generation
    ///
    /// Best used for optional parsers or parsers with a defined fallback, usually for implementing
    /// backward compatibility or hidden aliases
    ///
    /// # Combinatoric usage
    ///
    /// ```rust
    /// # use bpaf::*;
    /// /// bpaf would accept both `-W` and `-H` flags, but the help message
    /// /// would contain only `-H`
    /// fn rectangle() -> impl Parser<(u32, u32)> {
    ///     let width = short('W')
    ///         .argument("PX")
    ///         .from_str::<u32>()
    ///         .fallback(10)
    ///         .hide();
    ///     let height = short('H')
    ///         .argument("PX")
    ///         .from_str::<u32>()
    ///         .fallback(10)
    ///         .hide();
    ///     construct!(width, height)
    /// }
    /// ```
    /// # Example
    /// ```console
    /// $ app -W 12 -H 15
    /// // (12, 15)
    /// $ app -H 333
    /// // (10, 333)
    /// $ app --help
    /// // contains -H but not -W
    /// ```
    ///
    /// # Derive usage
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Rectangle {
    ///     #[bpaf(short('W'), argument("PX"), from_str(u32), fallback(10), hide)]
    ///     width: u32,
    ///     #[bpaf(short('H'), argument("PX"), from_str(u32))]
    ///     height: u32,
    /// }
    /// ```
    ///
    /// # Example
    /// ```console
    /// $ app -W 12 -H 15
    /// // Rectangle { width: 12, height: 15 }
    /// $ app -H 333
    /// // Rectangle { width: 10, height: 333 }
    /// $ app --help
    /// // contains -H but not -W
    /// ```
    fn hide(self) -> ParseHide<Self>
    where
        Self: Sized + Parser<T>,
    {
        ParseHide { inner: self }
    }
    // }}}

    // {{{ group_help
    /// Attach help message to a complex parser
    ///
    /// All the fields contained in the inner parser will be surrounded
    /// by the group help message on the top and an empty line at the bottom
    ///
    /// # Combinatoric usage
    /// ```rust
    /// # use bpaf::*;
    /// fn rectangle() -> impl Parser<(u32, u32)> {
    ///     let width = short('w')
    ///         .argument("PX")
    ///         .from_str::<u32>();
    ///     let height = short('h')
    ///         .argument("PX")
    ///         .from_str::<u32>();
    ///     construct!(width, height)
    ///         .group_help("Takes a rectangle")
    /// }
    /// ```
    /// # Example
    /// ```console
    /// $ app --help
    /// ...
    ///             Takes a rectangle
    ///    -w <PX>  Width of the rectangle
    ///    -h <PX>  Height of the rectangle
    ///
    /// ...
    /// ```
    ///
    /// # Derive usage
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Rectangle {
    ///     width: u32,
    ///     height: u32,
    /// }
    ///
    /// #[derive(Debug, Clone, Bpaf)]
    /// struct Options {
    ///     #[bpaf(external, group_help("Takes a rectangle"))]
    ///     rectangle: Rectangle
    /// }
    /// ```
    fn group_help(self, message: &'static str) -> ParseGroupHelp<Self>
    where
        Self: Sized + Parser<T>,
    {
        ParseGroupHelp {
            inner: self,
            message,
        }
    }
    // }}}

    // consume
    // {{{ to_options
    /// Transform `Parser` into [`OptionParser`] to attach metadata and run
    ///
    /// # Combinatoric usage
    /// ```rust
    /// # use bpaf::*;
    /// fn parser() -> impl Parser<u32> {
    ///     short('i')
    ///         .argument("ARG")
    ///         .from_str::<u32>()
    /// }
    ///
    /// fn option_parser() -> impl OptionParser<u32> {
    ///     parser()
    ///         .to_options()
    ///         .version("3.1415")
    ///         .descr("This is a description")
    /// }
    /// ```
    ///
    /// See [`OptionParser`] for more methods available after conversion.
    ///
    /// # Derive usage
    /// Add a top level `options` annotation to generate [`OptionParser`] instead of default
    /// [`Parser`].
    ///
    /// In addition to `options` annotation you can also specify either `version` or
    /// `version(value)` annotation. Former will use version from `cargo`, later will use specified
    /// value which should be an expression of type `&'static str`, see
    /// [`version`](OptionParser::version).
    ///
    /// ```rust
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// #[bpaf(options, version("3.1415"))]
    /// /// This is a description
    /// struct Options {
    ///    verbose: bool,
    /// }
    /// ```
    ///
    /// # Example
    /// ```console
    /// $ app --version
    /// // Version: 3.1415
    /// $ app --help
    /// ...
    /// This is a description
    /// ...
    /// ```
    fn to_options(self) -> OptionParserStruct<T, Self>
    where
        Self: Sized + Parser<T>,
    {
        OptionParserStruct {
            info: info::Info::default(),
            inner_type: PhantomData,
            inner: self,
        }
    }
    // }}}
}

/// Wrap a value into a `Parser`
///
/// This parser produces `T` without consuming anything from the command line, can be useful
/// with [`construct!`]. As with any parsers `T` should be `Clone` and `Debug`.
///
/// # Combinatoric usage
/// ```rust
/// # use bpaf::*;
/// fn pair() -> impl Parser<(bool, u32)> {
///     let a = long("flag-a").switch();
///     let b = pure(42u32);
///     construct!(a, b)
/// }
/// ```
#[must_use]
pub fn pure<T>(val: T) -> ParsePure<T> {
    ParsePure(val)
}

/// Fail with a fixed error message
///
/// This parser produces `T` of any type but instead of producing it when asked - it fails
/// with a custom error message. Can be useful for creating custom logic
///
/// # Combinatoric usage
/// ```rust
/// # use bpaf::*;
/// fn must_agree() -> impl Parser<()> {
///     let a = long("accept").req_flag(());
///     let no_a = fail("You must accept the license agreement with --agree before proceeding");
///     construct!([a, no_a])
/// }
/// ```
///
/// # Example
/// ```console
/// $ app
/// // exits with "You must accept the license agreement with --agree before proceeding"
/// $ app --agree
/// // succeeds
/// ```
#[must_use]
pub fn fail<T>(msg: &'static str) -> ParseFail<T> {
    ParseFail {
        field1: msg,
        field2: PhantomData,
    }
}

/// Unsuccessful command line parsing outcome
///
/// Useful for unit testing for user parsers, intented to
/// be consumed with [`ParseFailure::unwrap_stdout`] and [`ParseFailure::unwrap_stdout`]
#[derive(Clone, Debug)]
pub enum ParseFailure {
    /// Terminate and print this to stdout
    Stdout(String),
    /// Terminate and print this to stderr
    Stderr(String),
}

impl ParseFailure {
    /// Returns the contained `stderr` values
    ///
    /// Intended to be used with unit tests
    ///
    /// # Panics
    ///
    /// Will panic if failure contains `stdout`
    #[allow(clippy::must_use_candidate)]
    pub fn unwrap_stderr(self) -> String {
        match self {
            Self::Stderr(err) => err,
            Self::Stdout(_) => {
                panic!("not an stderr: {:?}", self)
            }
        }
    }

    /// Returns the contained `stdout` values
    ///
    /// Intended to be used with unit tests
    ///
    /// # Panics
    ///
    /// Will panic if failure contains `stderr`
    #[allow(clippy::must_use_candidate)]
    pub fn unwrap_stdout(self) -> String {
        match self {
            Self::Stdout(err) => err,
            Self::Stderr(_) => {
                panic!("not an stdout: {:?}", self)
            }
        }
    }
}

/// Strip a command name if present at the front when used as a cargo command
///
/// This helper should be used on a top level parser
///
/// ```rust
/// # use bpaf::*;
/// let width = short('w').argument("PX").from_str::<u32>();
/// let height = short('h').argument("PX").from_str::<u32>();
/// let parser = cargo_helper("cmd", construct!(width, height)); // impl Parser<(u32, u32)>
/// # drop(parser);
/// ```
#[must_use]
pub fn cargo_helper<P, T>(cmd: &'static str, parser: P) -> impl Parser<T>
where
    T: 'static,
    P: Parser<T>,
{
    let skip = positional_if("", move |s| cmd == s).hide();
    construct!(skip, parser).map(|x| x.1)
}
