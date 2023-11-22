//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// important argument
    #[bpaf(long, try_fold_with)]
    argument: u32,
    /// some switch
    #[bpaf(long("switch"), try_fold_with(toggle_switch))]
    switch: bool,
}
