#[derive(Debug, Clone)]
pub struct ProfileViewModel {
    pub did: String,
    pub handle: String,
    pub display_name: String,
    pub description: String,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    pub followers_count: i64,
    pub follows_count: i64,
    pub posts_count: i64,
}

impl ProfileViewModel {
    pub fn from_detailed(
        profile: &atrium_api::app::bsky::actor::defs::ProfileViewDetailed,
    ) -> Self {
        ProfileViewModel {
            did: profile.did.to_string(),
            handle: profile.handle.to_string(),
            display_name: profile
                .display_name
                .clone()
                .unwrap_or_else(|| profile.handle.to_string()),
            description: profile.description.clone().unwrap_or_default(),
            avatar: profile.avatar.clone(),
            banner: profile.banner.clone(),
            followers_count: profile.followers_count.unwrap_or(0),
            follows_count: profile.follows_count.unwrap_or(0),
            posts_count: profile.posts_count.unwrap_or(0),
        }
    }
}
