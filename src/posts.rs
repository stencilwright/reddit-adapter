//! Listing a subreddit's posts by reading the mapped feed.
//!
//! The post-title anchor (`a[id^="post-title-"][slot="title"]`, mapped as
//! `post_titles`) carries the title as its text and the permalink as its
//! `href`. We navigate to the subreddit feed and read both. The feed is
//! virtualized, so this returns the first screenful; scroll-collect for full
//! pagination is a follow-up. A future revision can call Reddit's own JSON
//! listing from the page (browser-automation-backed API) for richer fields —
//! see the adapter's issues.

use std::collections::HashSet;
use std::time::Duration;

use anyhow::{Context, Result};
use apiwright::AdapterSession;

use crate::Post;

/// The place the post-title selector is mapped under. The selector itself is
/// generic across subreddits; we reuse it for any `r/<name>`.
const PLACE: &str = "subreddit_rust";
const REDDIT: &str = "https://www.reddit.com";

pub(crate) async fn list(session: &AdapterSession, subreddit: &str) -> Result<Vec<Post>> {
    let sub = normalize(subreddit);
    let selector = session.element_selector(PLACE, "post_titles").with_context(|| {
        format!("map missing element 'post_titles' at `{PLACE}` — re-map the site")
    })?;

    session.goto(&format!("{REDDIT}/r/{sub}/")).await?;
    // The post-title anchors rendering is our "feed loaded" signal.
    let _ = session.wait_for(&selector, Duration::from_secs(30)).await;

    let titles = session.extract_text(&selector).await?;
    let hrefs = session.extract_attr(&selector, "href").await?;
    if titles.len() != hrefs.len() {
        anyhow::bail!(
            "map looks stale at `{PLACE}`: {} titles but {} hrefs — re-map the site",
            titles.len(),
            hrefs.len(),
        );
    }

    // Dedup by permalink (the feed can repeat a pinned/ad post).
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for (title, href) in titles.into_iter().zip(hrefs) {
        let permalink = absolutize(&href);
        if seen.insert(permalink.clone()) {
            out.push(Post {
                title: title.trim().to_string(),
                permalink,
                subreddit: sub.clone(),
            });
        }
    }
    Ok(out)
}

/// Strip a leading `r/` or `/r/` and any surrounding slashes/whitespace.
fn normalize(subreddit: &str) -> String {
    subreddit
        .trim()
        .trim_start_matches('/')
        .trim_start_matches("r/")
        .trim_matches('/')
        .to_string()
}

/// Make a Reddit href absolute.
fn absolutize(href: &str) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        href.to_string()
    } else {
        format!("{REDDIT}{href}")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn normalize_strips_prefixes() {
        assert_eq!(super::normalize("rust"), "rust");
        assert_eq!(super::normalize("r/rust"), "rust");
        assert_eq!(super::normalize("/r/rust/"), "rust");
        assert_eq!(super::normalize("  r/rust  "), "rust");
    }

    #[test]
    fn absolutize_handles_relative_and_absolute() {
        assert_eq!(
            super::absolutize("/r/rust/comments/x/"),
            "https://www.reddit.com/r/rust/comments/x/"
        );
        let abs = "https://www.reddit.com/r/rust/comments/x/";
        assert_eq!(super::absolutize(abs), abs);
    }
}
