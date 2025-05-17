use clap::Parser;

mod fzf;
mod tmux;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Size of the fzf window
    #[arg(short, long, default_value = "80,36")]
    size: String,

    /// Title of the fzf window
    #[arg(short, long, default_value = "Select Window")]
    title: String,
}

fn main() {
    let args = Args::parse();

    let current_session = tmux::get_current_session();
    let mut windows = tmux::get_all_windows(&current_session);
    fzf::sort_windows(&mut windows);
    if let Some(sw) = fzf::select_window(&windows, &args.size, &args.title) {
        tmux::switch_window(sw);
    }
}
