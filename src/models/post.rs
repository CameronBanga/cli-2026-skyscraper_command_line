use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct PostViewModel {
    pub uri: String,
    pub cid: String,
    pub author_did: String,
    pub author_handle: String,
    pub author_display_name: String,
    pub author_avatar: Option<String>,
    pub text: String,
    pub facets: Vec<Facet>,
    pub created_at: DateTime<Utc>,
    pub like_count: i64,
    pub repost_count: i64,
    pub reply_count: i64,
    pub quote_count: i64,
    pub is_liked: bool,
    pub like_uri: Option<String>,
    pub is_reposted: bool,
    pub repost_uri: Option<String>,
    pub embed_summary: Option<EmbedSummary>,
    pub reply_parent_author: Option<String>,
    pub reposted_by: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Facet {
    pub start: usize,
    pub end: usize,
    pub kind: FacetKind,
}

#[derive(Debug, Clone)]
pub enum FacetKind {
    Mention(String),
    Link(String),
    Tag(String),
}

#[derive(Debug, Clone)]
pub struct EmbedSummary {
    pub kind: EmbedKind,
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone)]
pub enum EmbedKind {
    ExternalLink,
    Images(usize),
    Video,
    Record,
    RecordWithMedia,
}

impl PostViewModel {
    pub fn from_feed_view_post(
        fvp: &atrium_api::app::bsky::feed::defs::FeedViewPost,
    ) -> Option<Self> {
        let reply_parent_author = fvp.reply.as_ref().and_then(|r| {
            extract_author_from_reply_parent(&r.parent)
        });
        let reposted_by = fvp.reason.as_ref().and_then(extract_repost_reason);
        Self::from_post_view_inner(&fvp.post, reply_parent_author, reposted_by)
    }

    pub fn from_post_view(
        post: &atrium_api::app::bsky::feed::defs::PostView,
    ) -> Option<Self> {
        Self::from_post_view_inner(post, None, None)
    }

    fn from_post_view_inner(
        post: &atrium_api::app::bsky::feed::defs::PostView,
        reply_parent_author: Option<String>,
        reposted_by: Option<String>,
    ) -> Option<Self> {
        let author = &post.author;

        let record = match serde_json::to_value(&post.record) {
            Ok(v) => v,
            Err(_) => return None,
        };

        let text = record.get("text")?.as_str().unwrap_or("").to_string();
        let created_at_str = record.get("createdAt")?.as_str().unwrap_or("");
        let created_at = DateTime::parse_from_rfc3339(created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let facets = parse_facets_from_record(&record);

        let viewer = post.viewer.as_ref();
        let is_liked = viewer.and_then(|v| v.like.as_ref()).is_some();
        let like_uri = viewer.and_then(|v| v.like.clone());
        let is_reposted = viewer.and_then(|v| v.repost.as_ref()).is_some();
        let repost_uri = viewer.and_then(|v| v.repost.clone());

        let embed_summary = post.embed.as_ref().and_then(extract_embed_summary);

        Some(PostViewModel {
            uri: post.uri.clone(),
            cid: post.cid.as_ref().to_string(),
            author_did: author.did.to_string(),
            author_handle: author.handle.to_string(),
            author_display_name: author
                .display_name
                .clone()
                .unwrap_or_else(|| author.handle.to_string()),
            author_avatar: author.avatar.clone(),
            text,
            facets,
            created_at,
            like_count: post.like_count.unwrap_or(0),
            repost_count: post.repost_count.unwrap_or(0),
            reply_count: post.reply_count.unwrap_or(0),
            quote_count: post.quote_count.unwrap_or(0),
            is_liked,
            like_uri,
            is_reposted,
            repost_uri,
            embed_summary,
            reply_parent_author,
            reposted_by,
        })
    }
}

fn parse_facets_from_record(record: &serde_json::Value) -> Vec<Facet> {
    let mut facets = Vec::new();
    if let Some(raw_facets) = record.get("facets").and_then(|f| f.as_array()) {
        for facet in raw_facets {
            let index = match facet.get("index") {
                Some(idx) => idx,
                None => continue,
            };
            let start = index.get("byteStart").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            let end = index.get("byteEnd").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

            if let Some(features) = facet.get("features").and_then(|f| f.as_array()) {
                for feature in features {
                    let type_str = feature.get("$type").and_then(|t| t.as_str()).unwrap_or("");
                    let kind = match type_str {
                        "app.bsky.richtext.facet#mention" => {
                            let did = feature.get("did").and_then(|d| d.as_str()).unwrap_or("");
                            FacetKind::Mention(did.to_string())
                        }
                        "app.bsky.richtext.facet#link" => {
                            let uri = feature.get("uri").and_then(|u| u.as_str()).unwrap_or("");
                            FacetKind::Link(uri.to_string())
                        }
                        "app.bsky.richtext.facet#tag" => {
                            let tag = feature.get("tag").and_then(|t| t.as_str()).unwrap_or("");
                            FacetKind::Tag(tag.to_string())
                        }
                        _ => continue,
                    };
                    facets.push(Facet { start, end, kind });
                }
            }
        }
    }
    facets
}

fn extract_embed_summary(
    embed: &atrium_api::types::Union<
        atrium_api::app::bsky::feed::defs::PostViewEmbedRefs,
    >,
) -> Option<EmbedSummary> {
    use atrium_api::app::bsky::feed::defs::PostViewEmbedRefs;
    use atrium_api::types::Union;

    match embed {
        Union::Refs(PostViewEmbedRefs::AppBskyEmbedExternalView(ext)) => {
            Some(EmbedSummary {
                kind: EmbedKind::ExternalLink,
                title: Some(ext.external.title.clone()),
                description: Some(ext.external.description.clone()),
                url: Some(ext.external.uri.clone()),
            })
        }
        Union::Refs(PostViewEmbedRefs::AppBskyEmbedImagesView(imgs)) => {
            Some(EmbedSummary {
                kind: EmbedKind::Images(imgs.images.len()),
                title: None,
                description: None,
                url: None,
            })
        }
        Union::Refs(PostViewEmbedRefs::AppBskyEmbedVideoView(_)) => {
            Some(EmbedSummary {
                kind: EmbedKind::Video,
                title: None,
                description: None,
                url: None,
            })
        }
        Union::Refs(PostViewEmbedRefs::AppBskyEmbedRecordView(_)) => {
            Some(EmbedSummary {
                kind: EmbedKind::Record,
                title: None,
                description: Some("Quoted post".to_string()),
                url: None,
            })
        }
        Union::Refs(PostViewEmbedRefs::AppBskyEmbedRecordWithMediaView(_)) => {
            Some(EmbedSummary {
                kind: EmbedKind::RecordWithMedia,
                title: None,
                description: Some("Quote with media".to_string()),
                url: None,
            })
        }
        _ => None,
    }
}

fn extract_author_from_reply_parent(
    parent: &atrium_api::types::Union<
        atrium_api::app::bsky::feed::defs::ReplyRefParentRefs,
    >,
) -> Option<String> {
    use atrium_api::app::bsky::feed::defs::ReplyRefParentRefs;
    use atrium_api::types::Union;

    match parent {
        Union::Refs(ReplyRefParentRefs::PostView(pv)) => {
            Some(
                pv.author
                    .display_name
                    .clone()
                    .unwrap_or_else(|| pv.author.handle.to_string()),
            )
        }
        _ => None,
    }
}

fn extract_repost_reason(
    reason: &atrium_api::types::Union<
        atrium_api::app::bsky::feed::defs::FeedViewPostReasonRefs,
    >,
) -> Option<String> {
    use atrium_api::app::bsky::feed::defs::FeedViewPostReasonRefs;
    use atrium_api::types::Union;

    match reason {
        Union::Refs(FeedViewPostReasonRefs::ReasonRepost(rr)) => {
            Some(
                rr.by
                    .display_name
                    .clone()
                    .unwrap_or_else(|| rr.by.handle.to_string()),
            )
        }
        _ => None,
    }
}
