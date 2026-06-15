//! `reddit-adapter-test` — CLI over [`reddit_adapter`].
//!
//! ```text
//! reddit-adapter-test --subreddit rust
//! ```
//!
//! Prints the listed posts as JSON (default) or CSV. See
//! `specs/01-reddit-adapter.md` for the contract.

use clap::Parser;
use reddit_adapter::Reddit;

#[derive(Parser, Debug)]
#[command(
    name = "reddit-adapter-test",
    about = "List a subreddit's posts via the Reddit web app, as you"
)]
struct Args {
    /// stencilwright map name (e.g. "reddit").
    #[arg(long, default_value = "reddit")]
    site: String,

    /// Subreddit to list (with or without an `r/` prefix).
    #[arg(long)]
    subreddit: String,

    /// Run off-screen; surface only for login / captcha.
    #[arg(long)]
    offscreen: bool,

    /// Output format: json or csv.
    #[arg(long, default_value = "json")]
    format: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // If this process was re-exec'd as the browser daemon (`reddit-adapter-test daemon
    // <dir>`), run it and exit — apiwright spawns it this way.
    if apiwright::run_if_daemon().await? {
        return Ok(());
    }

    let args = Args::parse();

    let reddit = if args.offscreen {
        Reddit::open_offscreen(&args.site).await?
    } else {
        Reddit::open(&args.site).await?
    };

    let posts = reddit.subreddit_posts(&args.subreddit).await?;

    match args.format.as_str() {
        "csv" => {
            println!("title,permalink,subreddit");
            for p in &posts {
                println!("{:?},{},{}", p.title, p.permalink, p.subreddit);
            }
        }
        _ => println!("{}", serde_json::to_string_pretty(&posts)?),
    }
    Ok(())
}
