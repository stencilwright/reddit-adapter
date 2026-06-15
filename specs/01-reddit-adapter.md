# 01 — reddit-adapter

Status: **early.** Public-subreddit post listing implemented; live validation pending.

A browser-automation-backed API over Reddit's web app. Built on
[apiwright](https://github.com/stencilwright/stencilwright/tree/main/crates/apiwright)
(the runtime) and an embedded [stencilwright](https://github.com/stencilwright/stencilwright)
map. Read those for vocabulary (*place*, *element*, *map*, *raw feature*, *surface*).

## 1. Goal

Interact with Reddit **as you**, in a real browser, with no API key, app
registration, or OAuth — and return structured data. The first capability is
listing a subreddit's posts; the same approach extends to your history, saved
posts, and authenticated views as those get mapped.

A user driving the web client as themselves needs no token or app review and
sees exactly what they can already see. The adapter acts with your access, in
your own session.

## 2. Architecture

```
stencilwright  ──maps (masked, once)──▶  maps/reddit/{places,elements,mask}.toml
                                               │  (embedded via include_str!)
                                               ▼  (loaded, raw)
reddit-adapter ───────────────────────▶  apiwright::AdapterSession
   subreddit(name) → r/<name> feed          │  goto feed, read mapped post anchors
   parse rows  ← post-title anchors  ◀───────┘  surface on login / captcha
```

The structural map travels **embedded** in the binary; per-user secret
references (`values.toml`, for login `auto_fill`) are supplied at runtime, never
bundled.

## 3. The map (as built)

`maps/reddit/` bundles the places from the original Reddit mapping:

- `login_password` / `login_otp` — the two auth steps at `/login/` (same URL,
  distinguished by `visible_selector`), with `auto_fill` from `{reddit_username}`
  / `{reddit_password}` / `{reddit_totp}` secret references.
- `home` — the signed-in landing page.
- `subreddit_rust` — the capture target, with the `post_titles` element
  (`a[id^="post-title-"][slot="title"]`). The selector is generic across
  subreddits; the adapter reuses it for any `r/<name>`.

## 4. Capability — subreddit posts

`Reddit::subreddit_posts(name)`:

1. Normalize `name` (strip an `r/` prefix); navigate to
   `https://www.reddit.com/r/<name>/`.
2. Wait for the mapped `post_titles` anchors to render.
3. Read each anchor's **text** (the title) and **`href`** (the permalink,
   absolutized); dedup by permalink.

Public subreddits need no login. The feed is **virtualized**, so this returns
the first screenful; full pagination is a follow-up (§6).

### Fail-fast on drift

A mismatch between the title and href counts errors (naming the suspect place)
rather than returning a misaligned `Vec` — a stale-map signal, not silent garbage.

## 5. Public API & CLI

```rust
pub struct Post { pub title: String, pub permalink: String, pub subreddit: String }

pub struct Reddit { /* … */ }
impl Reddit {
    pub async fn open(site: &str) -> anyhow::Result<Self>;            // headed
    pub async fn open_offscreen(site: &str) -> anyhow::Result<Self>;  // surfaceable
    pub async fn subreddit_posts(&self, subreddit: &str) -> anyhow::Result<Vec<Post>>;
}
```

```text
reddit-adapter-test --subreddit <name> [--site reddit] [--offscreen] [--format json|csv]
```

## 6. Follow-ups

- **Browser-automation-backed JSON listing.** Reddit's web client backs its
  feeds with JSON; calling that from the authenticated page (`evaluate` →
  `fetch`) would return richer fields (author, score, created, flair) and all
  pages — the way slack-adapter does for search. This subsumes virtualization.
- **Scroll-collect** for the full virtualized feed if staying DOM-based.
- **Authenticated capabilities** — your post/comment history, saved posts —
  by mapping those places and wiring `values.toml` for `auto_fill` login.
- **Login / consent.** `auto_fill` resolves `{reddit_*}` from a local
  `values.toml` via the daemon's 1Password; absent that, the headed window
  surfaces for manual login.
