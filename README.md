# reddit-adapter

**Use Reddit programmatically — on your terms.** Drives the Reddit web app in a
real browser *as you* and returns structured data — **no API key, app
registration, or OAuth.** First capability: list a subreddit's posts (title +
permalink). The same approach extends to anything the web app can do (your
history, saved posts, authenticated views).

Built on [apiwright](https://github.com/stencilwright/stencilwright/tree/main/crates/apiwright)
(the runtime) and mapped with [stencilwright](https://github.com/stencilwright/stencilwright)
(the masked, LLM-collaborative site-mapper).

```rust
use reddit_adapter::Reddit;

let reddit = Reddit::open("reddit").await?;
for p in reddit.subreddit_posts("rust").await? {
    println!("{}  {}", p.title, p.permalink);   // title, permalink, subreddit
}
```

Dev/test CLI (a harness for exercising the adapter during development — not meant to be installed):

```sh
reddit-adapter-test --subreddit rust
```

## Consent first

The browser is **headed by default**, or **off-screen but surfaceable** for
batch runs — never truly headless. Login, 2FA, captcha, or any unrecognized page
brings the window forward. You can always watch what's being done in your name.

## Status

Early. Public-subreddit post listing is implemented against the embedded map
(login + recognition + the post-title selector); live end-to-end validation is
pending. Contract and follow-ups: [specs/01-reddit-adapter.md](specs/01-reddit-adapter.md).

## License

MIT
