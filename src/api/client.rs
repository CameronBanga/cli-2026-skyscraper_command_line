use anyhow::Result;
use bsky_sdk::BskyAgent;
use atrium_api::types::string::Datetime;

use crate::models::post::PostViewModel;
use crate::models::profile::ProfileViewModel;
use crate::models::thread::ThreadViewModel;

pub struct BlueskyClient {
    agent: BskyAgent,
}

impl BlueskyClient {
    pub async fn new() -> Result<Self> {
        let agent = BskyAgent::builder().build().await?;
        Ok(BlueskyClient { agent })
    }

    pub async fn login_app_password(&self, identifier: &str, password: &str) -> Result<()> {
        self.agent.login(identifier, password).await?;
        Ok(())
    }

    pub async fn did(&self) -> Option<String> {
        self.agent.did().await.map(|d| d.to_string())
    }

    pub async fn get_timeline(
        &self,
        cursor: Option<String>,
        limit: Option<u8>,
    ) -> Result<(Vec<PostViewModel>, Option<String>)> {
        let params = atrium_api::app::bsky::feed::get_timeline::ParametersData {
            algorithm: None,
            cursor,
            limit: limit.and_then(|l| l.try_into().ok()),
        };
        let output = self
            .agent
            .api
            .app
            .bsky
            .feed
            .get_timeline(params.into())
            .await?;

        let posts: Vec<PostViewModel> = output
            .feed
            .iter()
            .filter_map(PostViewModel::from_feed_view_post)
            .collect();

        Ok((posts, output.cursor.clone()))
    }

    pub async fn get_thread(&self, uri: &str) -> Result<Option<ThreadViewModel>> {
        let params = atrium_api::app::bsky::feed::get_post_thread::ParametersData {
            depth: Some(6u16.try_into().unwrap()),
            parent_height: Some(10u16.try_into().unwrap()),
            uri: uri.to_string(),
        };
        let output = self
            .agent
            .api
            .app
            .bsky
            .feed
            .get_post_thread(params.into())
            .await?;

        use atrium_api::app::bsky::feed::get_post_thread::OutputThreadRefs;
        use atrium_api::types::Union;

        match &output.thread {
            Union::Refs(OutputThreadRefs::AppBskyFeedDefsThreadViewPost(tvp)) => {
                Ok(ThreadViewModel::from_thread_view_post(tvp))
            }
            _ => Ok(None),
        }
    }

    pub async fn create_post(
        &self,
        text: String,
        reply_to: Option<ReplyRef>,
    ) -> Result<String> {
        let facets = {
            let rt = bsky_sdk::rich_text::RichText::new_with_detect_facets(&text).await?;
            rt.facets
        };

        let reply = reply_to.map(|r| {
            atrium_api::app::bsky::feed::post::ReplyRefData {
                parent: atrium_api::com::atproto::repo::strong_ref::MainData {
                    cid: r.parent_cid.parse().expect("valid cid"),
                    uri: r.parent_uri.clone(),
                }
                .into(),
                root: atrium_api::com::atproto::repo::strong_ref::MainData {
                    cid: r.root_cid.parse().expect("valid cid"),
                    uri: r.root_uri.clone(),
                }
                .into(),
            }
            .into()
        });

        let record = atrium_api::app::bsky::feed::post::RecordData {
            created_at: Datetime::now(),
            embed: None,
            entities: None,
            facets,
            labels: None,
            langs: None,
            reply,
            tags: None,
            text,
        };

        let result = self.agent.create_record(record).await?;
        Ok(result.uri.to_string())
    }

    pub async fn like(&self, uri: &str, cid: &str) -> Result<String> {
        let record = atrium_api::app::bsky::feed::like::RecordData {
            created_at: Datetime::now(),
            subject: atrium_api::com::atproto::repo::strong_ref::MainData {
                cid: cid.parse().expect("valid cid"),
                uri: uri.to_string(),
            }
            .into(),
            via: None,
        };

        let result = self.agent.create_record(record).await?;
        Ok(result.uri.to_string())
    }

    pub async fn unlike(&self, like_uri: &str) -> Result<()> {
        self.agent
            .delete_record(like_uri)
            .await?;
        Ok(())
    }

    pub async fn repost(&self, uri: &str, cid: &str) -> Result<String> {
        let record = atrium_api::app::bsky::feed::repost::RecordData {
            created_at: Datetime::now(),
            subject: atrium_api::com::atproto::repo::strong_ref::MainData {
                cid: cid.parse().expect("valid cid"),
                uri: uri.to_string(),
            }
            .into(),
            via: None,
        };

        let result = self.agent.create_record(record).await?;
        Ok(result.uri.to_string())
    }

    pub async fn unrepost(&self, repost_uri: &str) -> Result<()> {
        self.agent
            .delete_record(repost_uri)
            .await?;
        Ok(())
    }

    pub async fn get_profile(&self, actor: &str) -> Result<ProfileViewModel> {
        let params = atrium_api::app::bsky::actor::get_profile::ParametersData {
            actor: actor.parse().expect("valid handle or did"),
        };
        let output = self
            .agent
            .api
            .app
            .bsky
            .actor
            .get_profile(params.into())
            .await?;

        Ok(ProfileViewModel::from_detailed(&output))
    }

    pub async fn get_author_feed(
        &self,
        actor: &str,
        cursor: Option<String>,
    ) -> Result<(Vec<PostViewModel>, Option<String>)> {
        let params = atrium_api::app::bsky::feed::get_author_feed::ParametersData {
            actor: actor.parse().expect("valid handle or did"),
            cursor,
            filter: None,
            include_pins: None,
            limit: 50u8.try_into().ok(),
        };
        let output = self
            .agent
            .api
            .app
            .bsky
            .feed
            .get_author_feed(params.into())
            .await?;

        let posts: Vec<PostViewModel> = output
            .feed
            .iter()
            .filter_map(PostViewModel::from_feed_view_post)
            .collect();

        Ok((posts, output.cursor.clone()))
    }

    pub fn agent(&self) -> &BskyAgent {
        &self.agent
    }
}

#[derive(Debug, Clone)]
pub struct ReplyRef {
    pub parent_uri: String,
    pub parent_cid: String,
    pub root_uri: String,
    pub root_cid: String,
}
