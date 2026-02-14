use std::sync::Arc;

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::prelude::*;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::error;

use crate::action::Action;
use crate::api::auth;
use crate::api::client::{BlueskyClient, ReplyRef};
use crate::api::session;
use crate::config::AppConfig;
use crate::event::{self, EventHandler};
use crate::models::feed::FeedState;
use crate::models::profile::ProfileViewModel;
use crate::models::thread::ThreadViewModel;
use crate::tui::Tui;
use crate::ui::composer::Composer;
use crate::ui::login::LoginForm;
use crate::ui::Component;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Login,
    Timeline,
    Thread,
    Profile,
    About,
}

pub struct App {
    should_quit: bool,
    screen: Screen,
    screen_stack: Vec<Screen>,
    active_tab: usize,
    client: Arc<BlueskyClient>,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    handle: Option<String>,
    prefer_app_password: bool,

    // State
    timeline: FeedState,
    thread: Option<ThreadViewModel>,
    profile: Option<ProfileViewModel>,
    profile_feed: FeedState,
    error_message: Option<String>,

    // Active data-loading task (aborted when a new load starts or on navigation)
    active_load: Option<JoinHandle<()>>,

    // Modals
    login_form: LoginForm,
    composer: Composer,
    show_composer: bool,
    show_promo: bool,
}

impl App {
    pub fn new(handle: Option<String>, prefer_app_password: bool, client: Arc<BlueskyClient>) -> Self {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let default_handle = handle
            .clone()
            .or_else(|| session::get_last_handle());

        let show_promo = !AppConfig::load()
            .map(|c| c.promo_dismissed)
            .unwrap_or(false);

        App {
            should_quit: false,
            screen: Screen::Login,
            screen_stack: Vec::new(),
            active_tab: 0,
            client,
            action_tx,
            action_rx,
            handle: handle.clone(),
            prefer_app_password,
            timeline: FeedState::new(),
            thread: None,
            profile: None,
            profile_feed: FeedState::new(),
            error_message: None,
            active_load: None,
            login_form: LoginForm::new(default_handle),
            composer: Composer::new(),
            show_composer: false,
            show_promo,
        }
    }

    pub async fn run(&mut self, terminal: &mut Tui) -> Result<()> {
        // Try to restore session
        match auth::try_restore_session(&self.client).await {
            auth::AuthResult::Success(handle) => {
                self.screen = Screen::Timeline;
                self.handle = Some(handle);
                self.dispatch(Action::RefreshTimeline);
            }
            auth::AuthResult::NeedsLogin => {
                self.screen = Screen::Login;
            }
        }

        let mut events = EventHandler::new();

        loop {
            terminal.draw(|frame| self.draw(frame))?;

            tokio::select! {
                Some(event) = events.next() => {
                    self.handle_event(event);
                }
                Some(action) = self.action_rx.recv() => {
                    self.update(action).await;
                }
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => {
                // Promo popover intercepts all keys
                if self.show_promo {
                    if key.code == KeyCode::Enter {
                        let _ = open::that("https://apps.apple.com/app/apple-store/id6754198379?pt=118981141&ct=mac_tui_application&mt=8");
                    }
                    self.dismiss_promo();
                    return;
                }

                // Let modals handle keys first
                if self.show_composer {
                    if let Some(action) = self.composer.handle_key_event(key) {
                        self.dispatch(action);
                    }
                    return;
                }

                if self.screen == Screen::Login {
                    if let Some(action) = self.login_form.handle_key_event(key) {
                        self.dispatch(action);
                    }
                    return;
                }

                // About screen: Enter opens App Store URL
                if self.screen == Screen::About && key.code == KeyCode::Enter {
                    let _ = open::that("https://apps.apple.com/app/apple-store/id6754198379?pt=118981141&ct=mac_tui_application&mt=8");
                    return;
                }

                // Global key handling
                if let Some(action) =
                    event::key_to_action(key, self.show_composer, self.screen == Screen::Login)
                {
                    // Special handling for 'r' to populate reply_to
                    let action = if matches!(key.code, KeyCode::Char('r'))
                        && key.modifiers == KeyModifiers::NONE
                    {
                        self.make_reply_action()
                            .unwrap_or(Action::OpenComposer {
                                reply_to: None,
                                reply_to_author: None,
                            })
                    } else {
                        action
                    };
                    self.dispatch(action);
                }
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
    }

    fn make_reply_action(&self) -> Option<Action> {
        let post = match self.screen {
            Screen::Timeline => self.timeline.selected_post(),
            Screen::Thread => self.thread.as_ref().map(|t| &t.focal),
            Screen::Profile => self.profile_feed.selected_post(),
            _ => None,
        };

        post.map(|p| Action::OpenComposer {
            reply_to: Some(ReplyRef {
                parent_uri: p.uri.clone(),
                parent_cid: p.cid.clone(),
                root_uri: p.uri.clone(),
                root_cid: p.cid.clone(),
            }),
            reply_to_author: Some(p.author_display_name.clone()),
        })
    }

    fn dismiss_promo(&mut self) {
        self.show_promo = false;
        if let Ok(mut config) = AppConfig::load() {
            config.promo_dismissed = true;
            let _ = config.save();
        }
    }

    fn dispatch(&self, action: Action) {
        let _ = self.action_tx.send(action);
    }

    fn spawn_load(&mut self, future: impl std::future::Future<Output = ()> + Send + 'static) {
        self.cancel_load();
        self.active_load = Some(tokio::spawn(future));
    }

    fn cancel_load(&mut self) {
        if let Some(handle) = self.active_load.take() {
            handle.abort();
        }
    }

    async fn update(&mut self, action: Action) {
        match action {
            Action::Quit => {
                self.cancel_load();
                self.should_quit = true;
            }

            Action::ShowLogin => self.screen = Screen::Login,

            Action::SubmitLogin { handle, password } => {
                let client = self.client.clone();
                let tx = self.action_tx.clone();
                tokio::spawn(async move {
                    match auth::login_with_app_password(&client, &handle, &password).await {
                        Ok(h) => {
                            let _ = tx.send(Action::LoginSuccess(h));
                        }
                        Err(e) => {
                            let _ = tx.send(Action::LoginFailed(e.to_string()));
                        }
                    }
                });
            }

            Action::LoginSuccess(handle) => {
                self.handle = Some(handle);
                self.screen = Screen::Timeline;
                self.login_form.clear_error();
                self.dispatch(Action::RefreshTimeline);
            }

            Action::LoginFailed(msg) => {
                self.login_form.set_error(msg);
            }

            Action::Logout => {
                let _ = auth::logout();
                self.handle = None;
                self.screen = Screen::Login;
                self.timeline = FeedState::new();
            }

            Action::RefreshTimeline => {
                self.timeline.loading = true;
                let client = self.client.clone();
                let tx = self.action_tx.clone();
                self.spawn_load(async move {
                    match client.get_timeline(None, Some(50u8)).await {
                        Ok((posts, cursor)) => {
                            let _ = tx.send(Action::TimelineLoaded {
                                posts,
                                cursor,
                                append: false,
                            });
                        }
                        Err(e) => {
                            let _ = tx.send(Action::Error(e.to_string()));
                        }
                    }
                });
            }

            Action::LoadMoreTimeline => {
                if self.timeline.loading || self.timeline.cursor.is_none() {
                    return;
                }
                self.timeline.loading = true;
                let client = self.client.clone();
                let cursor = self.timeline.cursor.clone();
                let tx = self.action_tx.clone();
                self.spawn_load(async move {
                    match client.get_timeline(cursor, Some(50u8)).await {
                        Ok((posts, cursor)) => {
                            let _ = tx.send(Action::TimelineLoaded {
                                posts,
                                cursor,
                                append: true,
                            });
                        }
                        Err(e) => {
                            let _ = tx.send(Action::Error(e.to_string()));
                        }
                    }
                });
            }

            Action::TimelineLoaded {
                posts,
                cursor,
                append,
            } => {
                if append {
                    self.timeline.append_posts(posts, cursor);
                } else {
                    self.timeline.replace_posts(posts, cursor);
                }
            }

            Action::SelectNext => match self.screen {
                Screen::Timeline => {
                    self.timeline.select_next();
                    if self.timeline.near_bottom(20) {
                        self.dispatch(Action::LoadMoreTimeline);
                    }
                }
                Screen::Profile => {
                    self.profile_feed.select_next();
                }
                Screen::Thread => {
                    // Thread navigation is handled at draw level
                }
                _ => {}
            },

            Action::SelectPrev => match self.screen {
                Screen::Timeline => self.timeline.select_prev(),
                Screen::Profile => self.profile_feed.select_prev(),
                _ => {}
            },

            Action::ScrollToTop => match self.screen {
                Screen::Timeline => self.timeline.select_first(),
                Screen::Profile => self.profile_feed.select_first(),
                _ => {}
            },

            Action::ScrollToBottom => match self.screen {
                Screen::Timeline => self.timeline.select_last(),
                Screen::Profile => self.profile_feed.select_last(),
                _ => {}
            },

            Action::OpenThread => {
                let uri = match self.screen {
                    Screen::Timeline => {
                        self.timeline.selected_post().map(|p| p.uri.clone())
                    }
                    Screen::Profile => {
                        self.profile_feed.selected_post().map(|p| p.uri.clone())
                    }
                    _ => None,
                };

                if let Some(uri) = uri {
                    let client = self.client.clone();
                    let tx = self.action_tx.clone();
                    self.screen_stack.push(self.screen.clone());
                    self.screen = Screen::Thread;
                    self.spawn_load(async move {
                        match client.get_thread(&uri).await {
                            Ok(thread) => {
                                let _ = tx.send(Action::ThreadLoaded(thread));
                            }
                            Err(e) => {
                                let _ = tx.send(Action::Error(e.to_string()));
                            }
                        }
                    });
                }
            }

            Action::ThreadLoaded(thread) => {
                self.thread = thread;
            }

            Action::GoBack => {
                self.cancel_load();
                if let Some(prev) = self.screen_stack.pop() {
                    self.screen = prev;
                    self.thread = None;
                }
            }

            Action::SwitchTab(idx) => {
                self.active_tab = idx;
                match idx {
                    0 => {
                        self.screen = Screen::Timeline;
                        if self.timeline.posts.is_empty() {
                            self.dispatch(Action::RefreshTimeline);
                        }
                    }
                    1 => {
                        if let Some(handle) = self.handle.clone() {
                            self.dispatch(Action::LoadProfile(handle));
                        }
                    }
                    _ => {}
                }
            }

            Action::OpenComposer {
                reply_to,
                reply_to_author,
            } => {
                self.composer = Composer::new();
                self.composer.set_reply(reply_to, reply_to_author);
                self.show_composer = true;
            }

            Action::CloseComposer => {
                self.show_composer = false;
            }

            Action::SubmitPost { text, reply_to } => {
                self.show_composer = false;
                let client = self.client.clone();
                let tx = self.action_tx.clone();
                tokio::spawn(async move {
                    match client.create_post(text, reply_to).await {
                        Ok(uri) => {
                            let _ = tx.send(Action::PostCreated(uri));
                        }
                        Err(e) => {
                            let _ = tx.send(Action::Error(e.to_string()));
                        }
                    }
                });
            }

            Action::PostCreated(_uri) => {
                self.dispatch(Action::RefreshTimeline);
            }

            Action::ToggleLike => {
                let post = match self.screen {
                    Screen::Timeline => self.timeline.selected_post().cloned(),
                    Screen::Thread => self.thread.as_ref().map(|t| t.focal.clone()),
                    Screen::Profile => self.profile_feed.selected_post().cloned(),
                    _ => None,
                };

                if let Some(post) = post {
                    let client = self.client.clone();
                    let tx = self.action_tx.clone();
                    if post.is_liked {
                        if let Some(like_uri) = post.like_uri.clone() {
                            let post_uri = post.uri.clone();
                            tokio::spawn(async move {
                                match client.unlike(&like_uri).await {
                                    Ok(_) => {
                                        let _ =
                                            tx.send(Action::UnlikeSuccess { post_uri });
                                    }
                                    Err(e) => {
                                        let _ = tx.send(Action::Error(e.to_string()));
                                    }
                                }
                            });
                        }
                    } else {
                        let uri = post.uri.clone();
                        let cid = post.cid.clone();
                        tokio::spawn(async move {
                            match client.like(&uri, &cid).await {
                                Ok(like_uri) => {
                                    let _ = tx.send(Action::LikeSuccess {
                                        post_uri: uri,
                                        like_uri,
                                    });
                                }
                                Err(e) => {
                                    let _ = tx.send(Action::Error(e.to_string()));
                                }
                            }
                        });
                    }
                }
            }

            Action::LikeSuccess { post_uri, like_uri } => {
                self.update_post(&post_uri, |p| {
                    p.is_liked = true;
                    p.like_uri = Some(like_uri.clone());
                    p.like_count += 1;
                });
            }

            Action::UnlikeSuccess { post_uri } => {
                self.update_post(&post_uri, |p| {
                    p.is_liked = false;
                    p.like_uri = None;
                    p.like_count = (p.like_count - 1).max(0);
                });
            }

            Action::ToggleRepost => {
                let post = match self.screen {
                    Screen::Timeline => self.timeline.selected_post().cloned(),
                    Screen::Thread => self.thread.as_ref().map(|t| t.focal.clone()),
                    Screen::Profile => self.profile_feed.selected_post().cloned(),
                    _ => None,
                };

                if let Some(post) = post {
                    let client = self.client.clone();
                    let tx = self.action_tx.clone();
                    if post.is_reposted {
                        if let Some(repost_uri) = post.repost_uri.clone() {
                            let post_uri = post.uri.clone();
                            tokio::spawn(async move {
                                match client.unrepost(&repost_uri).await {
                                    Ok(_) => {
                                        let _ = tx.send(Action::UnrepostSuccess {
                                            post_uri,
                                        });
                                    }
                                    Err(e) => {
                                        let _ = tx.send(Action::Error(e.to_string()));
                                    }
                                }
                            });
                        }
                    } else {
                        let uri = post.uri.clone();
                        let cid = post.cid.clone();
                        tokio::spawn(async move {
                            match client.repost(&uri, &cid).await {
                                Ok(repost_uri) => {
                                    let _ = tx.send(Action::RepostSuccess {
                                        post_uri: uri,
                                        repost_uri,
                                    });
                                }
                                Err(e) => {
                                    let _ = tx.send(Action::Error(e.to_string()));
                                }
                            }
                        });
                    }
                }
            }

            Action::RepostSuccess {
                post_uri,
                repost_uri,
            } => {
                self.update_post(&post_uri, |p| {
                    p.is_reposted = true;
                    p.repost_uri = Some(repost_uri.clone());
                    p.repost_count += 1;
                });
            }

            Action::UnrepostSuccess { post_uri } => {
                self.update_post(&post_uri, |p| {
                    p.is_reposted = false;
                    p.repost_uri = None;
                    p.repost_count = (p.repost_count - 1).max(0);
                });
            }

            Action::ViewAuthorProfile => {
                let did = match self.screen {
                    Screen::Timeline => {
                        self.timeline.selected_post().map(|p| p.author_did.clone())
                    }
                    Screen::Thread => {
                        self.thread.as_ref().map(|t| t.focal.author_did.clone())
                    }
                    _ => None,
                };
                if let Some(did) = did {
                    self.screen_stack.push(self.screen.clone());
                    self.dispatch(Action::LoadProfile(did));
                }
            }

            Action::LoadProfile(actor) => {
                self.screen = Screen::Profile;
                self.profile = None;
                self.profile_feed = FeedState::new();
                self.profile_feed.loading = true;
                let client = self.client.clone();
                let tx = self.action_tx.clone();
                self.spawn_load(async move {
                    let profile_result = client.get_profile(&actor).await;
                    let feed_result = client.get_author_feed(&actor, None).await;
                    match (profile_result, feed_result) {
                        (Ok(profile), Ok((posts, cursor))) => {
                            let _ = tx.send(Action::ProfileLoaded {
                                profile,
                                posts,
                                cursor,
                            });
                        }
                        (Err(e), _) | (_, Err(e)) => {
                            let _ = tx.send(Action::Error(e.to_string()));
                        }
                    }
                });
            }

            Action::ProfileLoaded {
                profile,
                posts,
                cursor,
            } => {
                self.profile = Some(profile);
                self.profile_feed.replace_posts(posts, cursor);
            }

            Action::ShowAbout => {
                self.screen_stack.push(self.screen.clone());
                self.screen = Screen::About;
            }

            Action::Error(msg) => {
                error!("Error: {}", msg);
                self.error_message = Some(msg);
            }

            Action::ClearError => {
                self.error_message = None;
            }

            _ => {}
        }
    }

    fn update_post(&mut self, uri: &str, f: impl Fn(&mut crate::models::post::PostViewModel)) {
        for post in &mut self.timeline.posts {
            if post.uri == uri {
                f(post);
            }
        }
        for post in &mut self.profile_feed.posts {
            if post.uri == uri {
                f(post);
            }
        }
        if let Some(ref mut thread) = self.thread {
            if thread.focal.uri == uri {
                f(&mut thread.focal);
            }
            for post in &mut thread.parents {
                if post.uri == uri {
                    f(post);
                }
            }
            for post in &mut thread.replies {
                if post.uri == uri {
                    f(post);
                }
            }
        }
    }

    fn draw(&self, frame: &mut ratatui::Frame) {
        let area = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(area);

        // Tab bar
        crate::ui::tabs::draw_tabs(frame, chunks[0], self.active_tab);

        // Main content
        match self.screen {
            Screen::Login => {
                self.login_form.draw(frame, chunks[1]);
            }
            Screen::Timeline => {
                crate::ui::timeline::draw_timeline(frame, chunks[1], &self.timeline);
            }
            Screen::Thread => {
                crate::ui::thread::draw_thread(frame, chunks[1], self.thread.as_ref());
            }
            Screen::Profile => {
                crate::ui::profile::draw_profile(
                    frame,
                    chunks[1],
                    self.profile.as_ref(),
                    &self.profile_feed,
                );
            }
            Screen::About => {
                crate::ui::about::draw_about(frame, chunks[1]);
            }
        }

        // Status bar
        crate::ui::statusbar::draw_statusbar(
            frame,
            chunks[2],
            &self.screen,
            self.show_composer,
            self.error_message.as_deref(),
        );

        // Composer overlay
        if self.show_composer {
            self.composer.draw(frame, area);
        }

        // Promo popover overlay
        if self.show_promo {
            self.draw_promo_popover(frame, area);
        }
    }

    fn draw_promo_popover(&self, frame: &mut ratatui::Frame, area: Rect) {
        use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph, Wrap};

        let popup_width = 62.min(area.width.saturating_sub(4));
        let popup_height = 9.min(area.height.saturating_sub(2));
        let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
        let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
        let popup_area = Rect::new(x, y, popup_width, popup_height);

        frame.render_widget(Clear, popup_area);

        let text = "\
Skyscraper is also available on iOS!\n\
\n\
Download the companion app for a full\n\
Bluesky experience on your iPhone or iPad.\n\
\n\
Enter: open App Store  |  any key: dismiss";

        let popup = Paragraph::new(text)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(" Welcome ")
                    .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                    .padding(Padding::horizontal(1)),
            );
        frame.render_widget(popup, popup_area);
    }
}
