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

use anyhow::{Context, Result, bail};
use apiwright::AdapterSession;

use crate::Post;

/// The place the post-title selector is mapped under. The selector itself is
/// generic across subreddits; we reuse it for any `r/<name>`.
const PLACE: &str = "subreddit_rust";
const REDDIT: &str = "https://www.reddit.com";

pub(crate) async fn list(session: &AdapterSession, subreddit: &str) -> Result<Vec<Post>> {
    let sub = normalize(subreddit);
    let selector = session
        .element_selector(PLACE, "post_titles")
        .with_context(|| {
            format!("map missing element 'post_titles' at `{PLACE}` — re-map the site")
        })?;

    session.goto(&format!("{REDDIT}/r/{sub}/")).await?;
    // The post-title anchors rendering is our "feed loaded" signal.
    session
        .wait_for(&selector, Duration::from_secs(30))
        .await
        .with_context(|| {
            format!(
                "timed out waiting for mapped post titles at `{PLACE}` using selector `{selector}`"
            )
        })?;

    let titles = session.extract_text(&selector).await?;
    let hrefs = session.extract_attr(&selector, "href").await?;
    posts_from_parts(sub, titles, hrefs)
}

fn posts_from_parts(
    subreddit: String,
    titles: Vec<String>,
    hrefs: Vec<String>,
) -> Result<Vec<Post>> {
    if titles.len() != hrefs.len() {
        bail!(
            "map looks stale at `{PLACE}`: {} titles but {} hrefs — re-map the site",
            titles.len(),
            hrefs.len(),
        );
    }
    if titles.is_empty() {
        bail!(
            "no post title nodes found at `{PLACE}` after the feed load signal; \
             re-map the site or add an explicit empty-state detector"
        );
    }

    // Dedup by permalink (the feed can repeat a pinned/ad post).
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for (idx, (title, href)) in titles.into_iter().zip(hrefs).enumerate() {
        let title = title.trim();
        if title.is_empty() {
            bail!("map looks stale at `{PLACE}`: post title at index {idx} is empty");
        }
        if href.trim().is_empty() {
            bail!("map looks stale at `{PLACE}`: post href at index {idx} is empty");
        }
        let permalink = absolutize(&href);
        if seen.insert(permalink.clone()) {
            out.push(Post {
                title: title.to_string(),
                permalink,
                subreddit: subreddit.clone(),
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

    #[test]
    fn empty_extraction_is_not_a_successful_empty_listing() {
        let err = super::posts_from_parts("rust".to_string(), vec![], vec![])
            .expect_err("empty extraction should fail loudly");
        assert!(
            err.to_string().contains("no post title nodes found"),
            "unexpected error: {err:#}",
        );
    }

    #[test]
    fn blank_rows_are_stale_map_errors() {
        let err = super::posts_from_parts("rust".to_string(), vec!["  ".into()], vec!["/x".into()])
            .expect_err("blank title should fail loudly");
        assert!(
            err.to_string().contains("post title at index 0 is empty"),
            "unexpected error: {err:#}",
        );

        let err =
            super::posts_from_parts("rust".to_string(), vec!["title".into()], vec!["".into()])
                .expect_err("blank href should fail loudly");
        assert!(
            err.to_string().contains("post href at index 0 is empty"),
            "unexpected error: {err:#}",
        );
    }

    #[test]
    fn posts_from_parts_dedups_and_preserves_subreddit() {
        let posts = super::posts_from_parts(
            "rust".to_string(),
            vec![" First ".into(), "First duplicate".into()],
            vec!["/r/rust/comments/x/".into(), "/r/rust/comments/x/".into()],
        )
        .unwrap();

        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].title, "First");
        assert_eq!(posts[0].subreddit, "rust");
        assert_eq!(
            posts[0].permalink,
            "https://www.reddit.com/r/rust/comments/x/"
        );
    }
}
