use super::post::PostViewModel;

#[derive(Debug, Clone)]
pub struct ThreadViewModel {
    pub parents: Vec<PostViewModel>,
    pub focal: PostViewModel,
    pub replies: Vec<PostViewModel>,
}

impl ThreadViewModel {
    pub fn from_thread_view_post(
        tvp: &atrium_api::app::bsky::feed::defs::ThreadViewPost,
    ) -> Option<Self> {
        let focal = PostViewModel::from_post_view(&tvp.post)?;

        let mut parents = Vec::new();
        Self::collect_parents(&tvp.parent, &mut parents);
        parents.reverse();

        let mut replies = Vec::new();
        if let Some(ref reply_list) = tvp.replies {
            for reply in reply_list {
                Self::collect_reply(reply, &mut replies);
            }
        }

        Some(ThreadViewModel {
            parents,
            focal,
            replies,
        })
    }

    fn collect_parents(
        parent: &Option<
            atrium_api::types::Union<
                atrium_api::app::bsky::feed::defs::ThreadViewPostParentRefs,
            >,
        >,
        out: &mut Vec<PostViewModel>,
    ) {
        use atrium_api::app::bsky::feed::defs::ThreadViewPostParentRefs;
        use atrium_api::types::Union;

        if let Some(Union::Refs(ThreadViewPostParentRefs::ThreadViewPost(
            tvp,
        ))) = parent
        {
            if let Some(post) = PostViewModel::from_post_view(&tvp.post) {
                out.push(post);
            }
            Self::collect_parents(&tvp.parent, out);
        }
    }

    fn collect_reply(
        reply: &atrium_api::types::Union<
            atrium_api::app::bsky::feed::defs::ThreadViewPostRepliesItem,
        >,
        out: &mut Vec<PostViewModel>,
    ) {
        use atrium_api::app::bsky::feed::defs::ThreadViewPostRepliesItem;
        use atrium_api::types::Union;

        if let Union::Refs(ThreadViewPostRepliesItem::ThreadViewPost(tvp)) =
            reply
        {
            if let Some(post) = PostViewModel::from_post_view(&tvp.post) {
                out.push(post);
            }
        }
    }
}
