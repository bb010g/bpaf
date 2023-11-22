//
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    argument: Vec<u32>,
    switches: Vec<bool>,
}

pub fn options() -> OptionParser<Options> {
    let argument = long("argument")
        .help("important argument")
        .argument("ARG")
        .try_fold_with();
    let switches = long("switch").help("some switch").switch().try_fold_with();
    construct!(Options { argument, switches }).to_options()
}
