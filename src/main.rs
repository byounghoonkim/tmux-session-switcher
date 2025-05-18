use clap::Parser;

mod fzf;
mod tmux;
use tmux::Item;
use tmux::favorite::Favorite;

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

    let mut ws: Vec<Box<dyn Item>> = Vec::new();

    // TODO: load favorites from a file or config
    ws.push(Box::new(Favorite {
        name: "TestFavorites".to_string(),
        session_name: Some("WORK".to_string()),
        //session_name: None,
        index: Some("3".to_string()),
        //index: None,
        path: Some("~/oss".to_string()),
    }));

    let current_session = tmux::get_current_session();
    let windows = tmux::get_running_windows(&current_session);
    for window in &windows {
        ws.push(Box::new(window.clone()));
    }

    fzf::sort_windows(&mut ws);
    if let Some(sw) = fzf::select_item::<dyn Item>(&ws, &args.size, &args.title) {
        sw.switch_window();
    }
}
