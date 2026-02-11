use super::post::PostViewModel;

#[derive(Debug, Clone)]
pub struct FeedState {
    pub posts: Vec<PostViewModel>,
    pub cursor: Option<String>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub loading: bool,
}

impl FeedState {
    pub fn new() -> Self {
        FeedState {
            posts: Vec::new(),
            cursor: None,
            selected_index: 0,
            scroll_offset: 0,
            loading: false,
        }
    }

    pub fn select_next(&mut self) {
        if !self.posts.is_empty() && self.selected_index < self.posts.len() - 1 {
            self.selected_index += 1;
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn select_first(&mut self) {
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    pub fn select_last(&mut self) {
        if !self.posts.is_empty() {
            self.selected_index = self.posts.len() - 1;
        }
    }

    pub fn selected_post(&self) -> Option<&PostViewModel> {
        self.posts.get(self.selected_index)
    }

    pub fn append_posts(&mut self, new_posts: Vec<PostViewModel>, cursor: Option<String>) {
        self.posts.extend(new_posts);
        self.cursor = cursor;
        self.loading = false;
    }

    pub fn replace_posts(&mut self, posts: Vec<PostViewModel>, cursor: Option<String>) {
        self.posts = posts;
        self.cursor = cursor;
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.loading = false;
    }

    pub fn near_bottom(&self, _visible_height: usize) -> bool {
        if self.posts.is_empty() {
            return false;
        }
        self.selected_index + 3 >= self.posts.len() && self.cursor.is_some()
    }
}

impl Default for FeedState {
    fn default() -> Self {
        Self::new()
    }
}
