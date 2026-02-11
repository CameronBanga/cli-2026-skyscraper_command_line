use crate::api::client::ReplyRef;
use crate::models::post::PostViewModel;
use crate::models::profile::ProfileViewModel;
use crate::models::thread::ThreadViewModel;

#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    Tick,
    Render,

    // Navigation
    SelectNext,
    SelectPrev,
    ScrollToTop,
    ScrollToBottom,
    OpenThread,
    GoBack,
    SwitchTab(usize),
    ViewAuthorProfile,

    // Auth
    ShowLogin,
    SubmitLogin {
        handle: String,
        password: String,
    },
    LoginSuccess(String),
    LoginFailed(String),
    Logout,

    // Timeline
    RefreshTimeline,
    LoadMoreTimeline,
    TimelineLoaded {
        posts: Vec<PostViewModel>,
        cursor: Option<String>,
        append: bool,
    },

    // Thread
    ThreadLoaded(Option<ThreadViewModel>),

    // Composer
    OpenComposer {
        reply_to: Option<ReplyRef>,
        reply_to_author: Option<String>,
    },
    CloseComposer,
    SubmitPost {
        text: String,
        reply_to: Option<ReplyRef>,
    },
    PostCreated(String),

    // Interactions
    ToggleLike,
    LikeSuccess {
        post_uri: String,
        like_uri: String,
    },
    UnlikeSuccess {
        post_uri: String,
    },
    ToggleRepost,
    RepostSuccess {
        post_uri: String,
        repost_uri: String,
    },
    UnrepostSuccess {
        post_uri: String,
    },

    // Profile
    LoadProfile(String),
    ProfileLoaded {
        profile: ProfileViewModel,
        posts: Vec<PostViewModel>,
        cursor: Option<String>,
    },

    // Errors
    Error(String),
    ClearError,
}
