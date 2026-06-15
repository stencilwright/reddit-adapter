//! # reddit-adapter — a browser-automation-backed API over Reddit
//!
//! Drives the Reddit web app (as *you*, in a real browser via [`apiwright`]) and
//! returns structured data — no API key, app registration, or OAuth. Listing a
//! subreddit's posts is the first capability; the same approach extends to
//! anything the web app can do (your history, saved posts, authenticated views).
//!
//! ```no_run
//! # use reddit_adapter::*;
//! # async fn demo() -> anyhow::Result<()> {
//! let reddit = Reddit::open("reddit").await?;
//! for p in reddit.subreddit_posts("rust").await? {
//!     println!("{}  {}", p.title, p.permalink);
//! }
//! # Ok(()) }
//! ```
//!
//! The structural site map (login, recognition, the post-title selector) is
//! embedded in the binary; per-user secret references are supplied at runtime,
//! never bundled. The full contract is in `specs/01-reddit-adapter.md`.
//!
//! [`apiwright`]: https://github.com/stencilwright/apiwright

use apiwright::{AdapterSession, RuntimeConfig};

mod map;
mod posts;

/// One post from a subreddit listing.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Post {
    /// The post title.
    pub title: String,
    /// Absolute permalink to the post / its comments.
    pub permalink: String,
    /// The subreddit it was listed under (without the `r/` prefix).
    pub subreddit: String,
}

/// The Reddit adapter handle.
pub struct Reddit {
    session: AdapterSession,
}

impl Reddit {
    /// Open the adapter for the embedded site map (`site` = the map name, e.g.
    /// `"reddit"`). Reuses the persistent Chrome profile; public subreddits need
    /// no login, so most listings work unauthenticated.
    pub async fn open(site: &str) -> anyhow::Result<Self> {
        Self::open_with(site, RuntimeConfig::new(site)).await
    }

    /// Open off-screen — surfaces only for login / captcha / on request.
    pub async fn open_offscreen(site: &str) -> anyhow::Result<Self> {
        Self::open_with(site, RuntimeConfig::new(site).offscreen()).await
    }

    async fn open_with(site: &str, cfg: RuntimeConfig) -> anyhow::Result<Self> {
        let graph = map::load(site)?;
        let session = AdapterSession::open_with_map(cfg, graph).await?;
        Ok(Self { session })
    }

    /// List the posts currently on a subreddit's feed. `subreddit` may be given
    /// with or without an `r/` prefix. Returns the first screenful — the feed is
    /// virtualized, so full pagination is a follow-up (spec §6).
    pub async fn subreddit_posts(&self, subreddit: &str) -> anyhow::Result<Vec<Post>> {
        posts::list(&self.session, subreddit).await
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn embedded_reddit_map_loads() {
        let g = crate::map::load("reddit").expect("embedded map loads");
        for p in ["login_password", "login_otp", "home", "subreddit_rust"] {
            assert!(g.place(p).is_some(), "missing place {p}");
        }
        let names: Vec<&str> = g
            .elements_at("subreddit_rust")
            .iter()
            .map(|e| e.name.as_str())
            .collect();
        assert!(names.contains(&"post_titles"), "subreddit_rust missing post_titles");
    }

    #[test]
    fn unknown_site_errors() {
        assert!(crate::map::load("nope").is_err());
    }
}
