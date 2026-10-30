//! Chat-surface rendering: the active tab, its split menus, and the pane tree.
mod dialogs;
mod drawer;
mod header;
mod pane;
mod panel;
mod start;
mod tab;
mod tree;

pub(crate) use drawer::*;
use panel::*;
use start::*;

use super::*;

use crate::*;

use gpui::{
    canvas, linear_color_stop, linear_gradient, quad, svg, BorderStyle, Div, FontWeight, Pixels,
    Point, Size,
};

use runner_app::ui::resize::{resize_strip, resize_strip_inset, ResizeAxis};

use runner_app::ui::{
    ButtonVariant, Modal, OverlayWidth, SessionControlVariant, SessionOverlay, SessionOverlayKind,
};

use runner_core::protocol::model::Runtime;

use crate::chat_icon::runtime_mark;

use crate::surfaces::chat_lifecycle::{
    ended_subtitle, resolve_pane_overlay, PaneOverlayState, TransitionKind,
};

use crate::surfaces::front_page::model::SessionDisplay;

use crate::surfaces::profile_page::{
    clamped_markdown, column_text, prompt_meta, setup_row, setup_value, ClickHandler,
};

use crate::surfaces::roles::logic::{role_setting_label, runtime_display_name};

use runner_app::ui::RoleAvatar;

const CHAT_PANEL_TRANSITION_MS: u64 = 200;

pub(crate) const UNFOCUSED_PANE_OPACITY: f32 = 0.7;

/// The smallest pane a split may leave behind, at 1× zoom.
const MIN_SPLIT_PANE_WIDTH: f32 = 240.;

const MIN_SPLIT_PANE_HEIGHT: f32 = 160.;

const TOO_SMALL_TO_SPLIT: &str = "ui.too_small_to_split";

impl NativeRoot {
    pub(crate) fn reset_chat_panel_prompt(&mut self, selected: Option<String>) {
        if self.chat_panel_prompt_session != selected {
            self.chat_panel_prompt_session = selected;
            self.chat_panel_prompt_expanded = false;
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum SplitMenuSurface {
    Header,
    Pane,
}

impl SplitMenuSurface {
    fn key(self) -> &'static str {
        match self {
            Self::Header => "header",
            Self::Pane => "pane",
        }
    }
}

/// One pane, named the only way a pane id is unique: inside its tab.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct PaneKey {
    pub(crate) tab_id: String,
    pub(crate) pane_id: String,
}

impl PaneKey {
    pub(crate) fn new(tab_id: &str, pane_id: &str) -> Self {
        Self {
            tab_id: tab_id.to_owned(),
            pane_id: pane_id.to_owned(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct SplitMenuKey {
    pub(crate) surface: SplitMenuSurface,
    pub(crate) tab_id: String,
    pub(crate) pane_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PaneCloseBehavior {
    LayoutOnly,
    CloseTerminal,
    ArchiveChat,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PaneRenameKey {
    Submit,
    Cancel,
}

fn pane_rename_key(key: &str) -> Option<PaneRenameKey> {
    match key {
        "enter" => Some(PaneRenameKey::Submit),
        "escape" => Some(PaneRenameKey::Cancel),
        _ => None,
    }
}

fn pane_identity_visible(pane_count: usize) -> bool {
    pane_count > 1
}

fn split_tab_archiving(layout: &PaneLayout, archiving: impl Fn(&str) -> bool) -> bool {
    let session_ids = layout.session_ids();
    layout.root.leaves().len() > 1
        && !session_ids.is_empty()
        && session_ids.iter().all(|session_id| archiving(session_id))
}

fn start_surface_visible(layout: Option<&PaneLayout>) -> bool {
    layout.is_none_or(|layout| {
        let leaves = layout.root.leaves();
        leaves.len() == 1 && leaves[0].session_id.is_none()
    })
}

pub(crate) fn terminal_drawer_tooltip(open: bool, overrides: &keymap::KeymapOverrides) -> String {
    let label = if open {
        crate::i18n::text("ui.hide_terminal_drawer")
    } else {
        crate::i18n::text("ui.show_terminal_drawer")
    };
    keymap::effective_binding("toggle-terminal-drawer", overrides).map_or_else(
        || label.to_owned(),
        |combo| format!("{label} · {}", keymap::format_combo(&combo)),
    )
}

/// A split leaves two panes where one stood, so each side needs half of the
/// floor. `None` bounds mean the pane has not been laid out yet — allow it
/// rather than refuse on a measurement we do not have.
fn split_allowed(pane: Option<Size<Pixels>>, orientation: SplitOrientation, zoom: f32) -> bool {
    let Some(pane) = pane else {
        return true;
    };
    match orientation {
        SplitOrientation::Row => f32::from(pane.width) >= 2. * MIN_SPLIT_PANE_WIDTH * zoom,
        SplitOrientation::Column => f32::from(pane.height) >= 2. * MIN_SPLIT_PANE_HEIGHT * zoom,
    }
}

fn drop_side(pane: Bounds<Pixels>, pointer: Point<Pixels>) -> DropSide {
    [
        (DropSide::Left, (pointer.x - pane.left()) / pane.size.width),
        (DropSide::Up, (pointer.y - pane.top()) / pane.size.height),
        (
            DropSide::Right,
            (pane.right() - pointer.x) / pane.size.width,
        ),
        (
            DropSide::Down,
            (pane.bottom() - pointer.y) / pane.size.height,
        ),
    ]
    .into_iter()
    .min_by(|(_, a), (_, b)| a.total_cmp(b))
    .expect("the four sides are a non-empty array")
    .0
}

fn drop_preview(mut pane: Bounds<Pixels>, side: DropSide) -> Bounds<Pixels> {
    match side {
        DropSide::Left | DropSide::Right => {
            pane.size.width /= 2.;
            if side == DropSide::Right {
                pane.origin.x += pane.size.width;
            }
        }
        DropSide::Up | DropSide::Down => {
            pane.size.height /= 2.;
            if side == DropSide::Down {
                pane.origin.y += pane.size.height;
            }
        }
    }
    pane
}

pub(crate) fn drop_allowed(pane: Option<Size<Pixels>>, side: DropSide, zoom: f32) -> bool {
    split_allowed(pane, side.orientation(), zoom)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SplitDecision {
    Allowed,
    /// Why the split cannot happen — menu tooltip and error-banner copy alike.
    Blocked(&'static str),
}

/// The one place a split request is judged. Both routes — the menu item and
/// `⌘D` / `⇧⌘D` — go through it, so the size floor cannot be walked around
/// by reaching for the keyboard.
pub(crate) fn split_decision(
    pane: Option<Size<Pixels>>,
    orientation: SplitOrientation,
    zoom: f32,
) -> SplitDecision {
    if split_allowed(pane, orientation, zoom) {
        SplitDecision::Allowed
    } else {
        SplitDecision::Blocked(crate::i18n::text(TOO_SMALL_TO_SPLIT))
    }
}

fn split_menu_items(
    pane: Option<Size<Pixels>>,
    zoom: f32,
    overrides: &keymap::KeymapOverrides,
) -> Vec<UiMenuItem> {
    [
        (
            crate::i18n::message("ui.split_right"),
            "split-pane-right",
            SplitOrientation::Row,
        ),
        (
            crate::i18n::message("ui.split_down"),
            "split-pane-down",
            SplitOrientation::Column,
        ),
    ]
    .into_iter()
    .map(|(label, binding, orientation)| {
        let blocked = match split_decision(pane, orientation, zoom) {
            SplitDecision::Allowed => None,
            SplitDecision::Blocked(reason) => Some(reason),
        };
        let mut item = UiMenuItem::new(label).disabled(blocked.is_some());
        if let Some(combo) = keymap::effective_binding(binding, overrides) {
            item = item.shortcut(keymap::format_combo(&combo));
        }
        if blocked.is_some() {
            item = item.tooltip(crate::i18n::message(TOO_SMALL_TO_SPLIT));
        }
        item
    })
    .collect()
}

fn workspace_header_icon(
    grouped: bool,
    focused_session_id: Option<&str>,
    focused_runtime: Option<&str>,
) -> ChatIcon {
    if grouped {
        ChatIcon::split()
    } else if focused_session_id.is_none() {
        ChatIcon::generic("square-dashed.svg")
    } else {
        ChatIcon::for_runtime(focused_runtime.unwrap_or_default())
    }
}

fn pane_body_opacity(grouped: bool, focused: bool) -> f32 {
    if grouped && !focused {
        UNFOCUSED_PANE_OPACITY
    } else {
        1.
    }
}

fn pane_identity_icon(runtime: Option<&str>) -> ChatIcon {
    runtime.map_or_else(
        || ChatIcon::generic("square-dashed.svg"),
        ChatIcon::for_runtime,
    )
}

fn pane_identity_shows_status(runtime: &str, program_status: bool) -> bool {
    program_status || !Runtime::parse(runtime).is_some_and(Runtime::is_shell)
}

fn starting_overlay_label(runtime: &str, fork_materializing: bool) -> Option<&'static str> {
    if Runtime::parse(runtime).is_some_and(Runtime::is_shell) {
        Some(crate::i18n::text("ui.starting_terminal"))
    } else if fork_materializing {
        Some(crate::i18n::text("ui.forking_chat"))
    } else {
        None
    }
}

fn side_panel_open(setting_open: bool, focused_shell: bool) -> bool {
    setting_open && !focused_shell
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HeaderForkState {
    Enabled,
    Disabled(Option<&'static str>),
    Hidden,
}

fn header_fork_state(
    focused: Option<&DirectSessionEntry>,
    focused_secondary: bool,
) -> HeaderForkState {
    let Some(entry) = focused
        .filter(|entry| !Runtime::parse(&entry.agent_runtime).is_some_and(Runtime::is_shell))
    else {
        return HeaderForkState::Hidden;
    };
    if focused_secondary {
        return HeaderForkState::Hidden;
    }
    if !entry.native_fork {
        HeaderForkState::Disabled(None)
    } else if !entry.forkable {
        HeaderForkState::Disabled(Some(crate::i18n::text("ui.no_session_key_captured_yet")))
    } else {
        HeaderForkState::Enabled
    }
}

pub(crate) fn pane_close_behavior(runtime: Option<&str>) -> PaneCloseBehavior {
    match runtime {
        None => PaneCloseBehavior::LayoutOnly,
        Some(runtime) if Runtime::parse(runtime).is_some_and(Runtime::is_shell) => {
            PaneCloseBehavior::CloseTerminal
        }
        Some(_) => PaneCloseBehavior::ArchiveChat,
    }
}

fn archive_chat_confirm_body(label: &str, running: bool) -> String {
    if running {
        crate::i18n::tf(
            "format.is_still_running_archiving_stops_it_you_can_restore_the_chat_from_settings_arc",
            &[("label", (label).to_string())],
        )
    } else {
        crate::i18n::tf(
            "format.you_can_restore_from_settings_archived",
            &[("label", (label).to_string())],
        )
    }
}

fn pane_action_indexes(
    entry: &SessionDisplay,
    client: &crate::surfaces::front_page::FrontPageClient,
) -> Vec<usize> {
    let mut indexes = Vec::new();
    if entry.agent_runtime != "shell"
        && client.supports(&[if entry.status == SessionStatus::Running {
            "stop"
        } else {
            "resume"
        }])
    {
        indexes.push(0);
    }
    if client.supports(&["rename"]) {
        indexes.push(if entry.agent_runtime == "shell" { 0 } else { 1 });
    }
    if entry.agent_runtime != "shell" && client.supports(&["archive", "layout"]) {
        indexes.push(2);
    }
    indexes
}

fn pane_action_items_for(
    runtime: &str,
    running: bool,
    disabled: bool,
    stop_shortcut: Option<String>,
    resume_shortcut: Option<String>,
) -> Vec<UiMenuItem> {
    let shell = Runtime::parse(runtime).is_some_and(Runtime::is_shell);
    if shell {
        return vec![
            UiMenuItem::new(crate::i18n::message("ui.rename_chat_action"))
                .icon("pencil.svg")
                .disabled(disabled),
        ];
    }
    let (label, icon, shortcut) = if running {
        (crate::i18n::message("ui.stop"), "square.svg", stop_shortcut)
    } else if shell {
        (
            crate::i18n::message("ui.restart"),
            "rotate-ccw.svg",
            resume_shortcut,
        )
    } else {
        (
            crate::i18n::message("ui.resume"),
            "play.svg",
            resume_shortcut,
        )
    };
    let primary = UiMenuItem::new(label).icon(icon).disabled(disabled);
    let primary = match shortcut {
        Some(shortcut) => primary.shortcut(shortcut),
        None => primary,
    };
    let mut items = vec![
        primary,
        UiMenuItem::new(crate::i18n::message("ui.rename_chat_action"))
            .icon("pencil.svg")
            .disabled(disabled),
    ];
    if !shell {
        items.push(
            UiMenuItem::new(crate::i18n::message("ui.archive_chat_action"))
                .icon("archive.svg")
                .separator_before(true)
                .destructive(true)
                .disabled(disabled),
        );
    }
    items
}

pub(crate) fn pane_fractions(node: &PaneNode, pane_id: &str) -> Option<(f32, f32)> {
    match node {
        PaneNode::Leaf(leaf) => (leaf.id == pane_id).then_some((1., 1.)),
        PaneNode::Split(split) => {
            let orientation = split.orientation;
            let first = split.sizes[0] / 100.;
            let second = split.sizes[1] / 100.;
            pane_fractions(&split.a, pane_id)
                .map(|(width, height)| match orientation {
                    SplitOrientation::Row => (width * first, height),
                    SplitOrientation::Column => (width, height * first),
                })
                .or_else(|| {
                    pane_fractions(&split.b, pane_id).map(|(width, height)| match orientation {
                        SplitOrientation::Row => (width * second, height),
                        SplitOrientation::Column => (width, height * second),
                    })
                })
        }
    }
}

pub(crate) fn adjacent_pane_index(
    current: usize,
    pane_count: usize,
    direction: isize,
) -> Option<usize> {
    if pane_count < 2 || current >= pane_count {
        return None;
    }
    Some((current as isize + direction).rem_euclid(pane_count as isize) as usize)
}

#[cfg(test)]
mod tests {
    use super::{
        adjacent_pane_index, archive_chat_confirm_body, drop_allowed, drop_preview, drop_side,
        header_fork_state, pane_action_items_for, pane_body_opacity, pane_close_behavior,
        pane_identity_icon, pane_identity_shows_status, pane_identity_visible, pane_rename_key,
        side_panel_open, split_allowed, split_decision, split_menu_items, split_tab_archiving,
        starting_overlay_label, terminal_drawer_tooltip, workspace_header_icon, HeaderForkState,
        PaneCloseBehavior, PaneRenameKey, SplitDecision, MIN_SPLIT_PANE_HEIGHT,
        MIN_SPLIT_PANE_WIDTH, TOO_SMALL_TO_SPLIT, UNFOCUSED_PANE_OPACITY,
    };
    use crate::keymap;
    use gpui::{point, px, size, AppContext, Bounds};
    use runner_app::pane_layout::{DropSide, PaneLayout, SplitOrientation};
    use runner_core::protocol::model::SessionStatus;
    use runner_core::protocol::session::DirectSessionEntry;

    fn direct_session(runtime: &str, native_fork: bool, forkable: bool) -> DirectSessionEntry {
        DirectSessionEntry {
            claude_account: None,
            session_id: format!("{runtime}-session"),
            project_id: None,
            role_id: None,
            handle: None,
            agent_runtime: runtime.into(),
            agent_command: runtime.into(),
            agent_model: None,
            agent_effort: None,
            display_name: runtime.into(),
            status: SessionStatus::Running,
            title: None,
            live_title: None,
            cwd: None,
            started_at: None,
            stopped_at: None,
            resumable: forkable,
            native_fork,
            forkable,
            agent_session_key: forkable.then(|| "key".into()),
            pinned: false,
            archived_at: None,
        }
    }

    #[test]
    fn terminal_drawer_header_has_no_fill_of_its_own() {
        use crate::theme;
        use crate::theme_snapshot::{assert_fill, ThemeGuard};
        use gpui::prelude::*;
        use gpui::{div, App, Context, Render, TestAppContext, VisualTestContext, Window};
        use std::rc::Rc;

        struct DrawerHeader;
        impl Render for DrawerHeader {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div()
                    .size_full()
                    .bg(theme::bg())
                    .child(super::render_terminal_drawer_strip(
                        "probe",
                        &[],
                        None,
                        &[],
                        super::TerminalDrawerCallbacks {
                            can_add: true,
                            can_close: true,
                            activate: Rc::new(|_, _: &mut Window, _: &mut App| {}),
                            close: Rc::new(|_, _, _| {}),
                            add: Rc::new(|_, _| {}),
                            hide: Rc::new(|_, _| {}),
                        },
                    ))
            }
        }
        let _theme = ThemeGuard::new();
        let mut cx = TestAppContext::single();
        let host = cx.add_window(|_, _| DrawerHeader);
        let mut visual = VisualTestContext::from_window(host.into(), &cx);
        visual.run_until_parked();
        assert_fill(&mut visual, "DRAWER_HEADER", None);
    }

    #[test]
    fn cached_chat_header_spans_the_chat_column_with_or_without_the_side_panel() {
        use crate::{AppRoute, NativeRoot};
        use gpui::{TestAppContext, VisualTestContext};

        let _theme = crate::theme_snapshot::ThemeGuard::new();
        for panel_open in [false, true] {
            for zoom in [1., 1.5] {
                let temp = tempfile::tempdir().unwrap();
                let mut cx = TestAppContext::single();
                let store = crate::app_store::test_lifecycle_store(&mut cx, temp.path());
                store.update(&mut cx, |store, cx| {
                    crate::app_store::seed_mixed_working_sessions(store);
                    store
                        .test_core
                        .db
                        .get()
                        .unwrap()
                        .execute(
                            "UPDATE sessions SET status = 'stopped' WHERE mission_id IS NULL",
                            [],
                        )
                        .unwrap();
                    store.refresh(crate::app_store::StoreRefreshKind::All, cx);
                    store.settings.chat_panel_open = panel_open;
                    store.settings.app_zoom = zoom;
                });
                let host = cx.add_window(|window, cx| {
                    let mut root = NativeRoot::new(
                        "chat-header-width".into(),
                        temp.path().join("logs"),
                        None,
                        None,
                        store.clone(),
                        window,
                        cx,
                    );
                    root.apply_tab_rows(cx);
                    assert!(root.tabs.activate_session("direct-agent"));
                    root.route = AppRoute::Chat;
                    root.sync_active_chat_detail(cx);
                    root
                });
                let mut visual = VisualTestContext::from_window(host.into(), &cx);
                for width in [1000., 1600.] {
                    visual.simulate_resize(size(px(width), px(900.)));
                    visual.run_until_parked();
                    let column = visual.debug_bounds("CHAT_TAB_HEADER").unwrap();
                    let header = visual.debug_bounds("CHAT_TAB_HEADER_CONTENT").unwrap();
                    assert_eq!(header.left(), column.left());
                    assert_eq!(
                        header.right(),
                        column.right(),
                        "panel={panel_open}, zoom={zoom}, width={width}"
                    );
                }
            }
        }
    }

    fn panel_role() -> runner_core::protocol::model::Role {
        runner_core::protocol::model::Role {
            id: "role-architect".into(),
            handle: "architect".into(),
            display_name: "Architect".into(),
            runtime: "claude-code".into(),
            command: "claude".into(),
            args: Vec::new(),
            working_dir: None,
            system_prompt: None,
            env: Default::default(),
            model: Some("fable".into()),
            effort: Some("xhigh".into()),
            codex_speed: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    struct ChatPanel {
        detail: DirectSessionEntry,
        role: Option<runner_core::protocol::model::Role>,
        width: f32,
        copy: gpui::Entity<runner_app::ui::CopyValueButton>,
    }

    impl gpui::Render for ChatPanel {
        fn render(
            &mut self,
            _: &mut gpui::Window,
            cx: &mut gpui::Context<Self>,
        ) -> impl gpui::IntoElement {
            use gpui::prelude::*;
            gpui::div()
                .w(gpui::rems(self.width / 16.))
                .p_5()
                .child(super::chat_panel_content(
                    &self.detail,
                    self.role.as_ref(),
                    self.width,
                    self.copy.clone(),
                    super::ChatPanelActions {
                        open_role: self.role.as_ref().map(|_| {
                            std::rc::Rc::new(|_: &mut gpui::Window, _: &mut gpui::App| {})
                                as super::ClickHandler
                        }),
                        toggle_prompt: std::rc::Rc::new(|_, _| {}),
                        expanded: false,
                        view: cx.entity_id(),
                    },
                    self.detail.claude_account.as_ref().map(|account| {
                        (
                            "Work".into(),
                            Some("fixture@example.test".into()),
                            account.folder.clone(),
                        )
                    }),
                    cx,
                ))
        }
    }

    fn panel_harness(
        detail: DirectSessionEntry,
        role: Option<runner_core::protocol::model::Role>,
        width: f32,
    ) -> (gpui::TestAppContext, gpui::VisualTestContext) {
        let mut cx = gpui::TestAppContext::single();
        let window = cx.add_window(move |_, cx| ChatPanel {
            copy: cx.new(|cx| {
                runner_app::ui::CopyValueButton::new(cx.focus_handle(), None, "Copy session_key")
            }),
            detail,
            role,
            width,
        });
        let visual = gpui::VisualTestContext::from_window(window.into(), &cx);
        visual.simulate_resize(size(px(500.), px(800.)));
        visual.run_until_parked();
        (cx, visual)
    }

    #[test]
    fn single_claude_tab_shows_its_account_in_the_top_header() {
        let _theme = crate::theme_snapshot::ThemeGuard::new();
        let temp = tempfile::tempdir().unwrap();
        let mut cx = gpui::TestAppContext::single();
        let store = crate::app_store::test_lifecycle_store(&mut cx, temp.path());
        store.update(&mut cx,|store,cx| {
            crate::app_store::seed_mixed_working_sessions(store);
            {
                let conn = store.test_core.db.get().unwrap();
                conn.execute("UPDATE sessions SET status='stopped', agent_runtime='claude-code', agent_command='claude' WHERE id='direct-agent'",[]).unwrap();
                let accounts = serde_json::json!({"accounts":[{"id":"native","name":"Personal","folder":"/tmp/native","native":true},{"id":"work","name":"Work","folder":"/tmp/work","native":false}],"main_id":"native","active_id":"work"});
                conn.execute("INSERT INTO _app_state(key,value) VALUES('claude_accounts',?1)",[accounts.to_string()]).unwrap();
            }
            store.refresh(crate::app_store::StoreRefreshKind::All,cx);
        });
        let host = cx.add_window(|window, cx| {
            let mut root = crate::NativeRoot::new(
                "account-header".into(),
                temp.path().join("logs"),
                None,
                None,
                store.clone(),
                window,
                cx,
            );
            root.apply_tab_rows(cx);
            assert!(root.tabs.activate_session("direct-agent"));
            root.route = crate::AppRoute::Chat;
            root.sync_active_chat_detail(cx);
            root
        });
        let mut visual = gpui::VisualTestContext::from_window(host.into(), &cx);
        visual.simulate_resize(size(px(1200.), px(900.)));
        visual.run_until_parked();
        let header = visual.debug_bounds("CHAT_TAB_HEADER_CONTENT").unwrap();
        let chip = visual.debug_bounds("CHAT_ACCOUNT_CHIP").unwrap();
        assert!(chip.top() >= header.top() && chip.bottom() <= header.bottom());
    }

    #[test]
    fn account_is_beside_runtime_and_config_is_in_the_metadata_rows() {
        let _theme = crate::theme_snapshot::ThemeGuard::new();
        let mut detail = direct_session("claude-code", true, true);
        detail.claude_account = Some(runner_core::protocol::ClaudeSessionAccount {
            account_id: Some("work".into()),
            folder: "/tmp/work".into(),
            native: false,
            pinned: false,
            origin_folder: "/tmp/native".into(),
            permission_mode: "default".into(),
        });
        for role in [None, Some(panel_role())] {
            let (_cx, mut visual) = panel_harness(detail.clone(), role, 320.);
            let runtime = visual.debug_bounds("SETUP_ROW RUNTIME").unwrap();
            let account = visual.debug_bounds("SETUP_ROW Account").unwrap();
            assert!((runtime.top() - account.top()).abs() <= px(1.));
            assert!(account.left() > runtime.left());
            let config = visual.debug_bounds("CHAT_PANEL_META /tmp/work").unwrap();
            assert!(config.top() > account.bottom());
        }
    }

    #[test]
    fn panel_prompt_only_shows_nonblank_text_and_only_long_prompts_have_a_toggle() {
        let _theme = crate::theme_snapshot::ThemeGuard::new();
        for prompt in ["", " \n ", "# Short\n\nPrompt", &"line\n".repeat(60)] {
            let mut role = panel_role();
            role.system_prompt = Some(prompt.into());
            let (_cx, mut visual) = panel_harness(
                direct_session("codex", true, true),
                Some(role),
                crate::app_settings::CHAT_PANEL_MIN,
            );
            let link = visual.debug_bounds("CHAT_PANEL_OPEN_ROLE").unwrap();
            let label = visual.debug_bounds("CHAT_PANEL_LABEL Role").unwrap();
            assert!((label.center().y - link.center().y).abs() <= gpui::px(1.));
            assert!(link.right() <= gpui::px(crate::app_settings::CHAT_PANEL_MIN));
            if !prompt.trim().is_empty() {
                let label = visual
                    .debug_bounds("CHAT_PANEL_LABEL System prompt")
                    .unwrap();
                let meta = visual.debug_bounds("CHAT_PANEL_PROMPT_META").unwrap();
                assert!((label.center().y - meta.center().y).abs() <= gpui::px(1.));
                assert!(meta.right() <= gpui::px(crate::app_settings::CHAT_PANEL_MIN));
            }
            assert_eq!(
                visual.debug_bounds("CHAT_PANEL_PROMPT_TEXT").is_some(),
                !prompt.trim().is_empty()
            );
            assert_eq!(
                visual.debug_bounds("CHAT_PANEL_PROMPT_TOGGLE").is_some(),
                prompt.lines().count() > crate::surfaces::profile_page::PREVIEW_LINES
            );
        }
    }

    #[test]
    fn role_chat_panel_renders_avatar_runtime_model_and_effort() {
        let _theme = crate::theme_snapshot::ThemeGuard::new();
        let mut detail = direct_session("claude-code", true, true);
        detail.agent_model = Some("fable".into());
        detail.agent_effort = Some("xhigh".into());
        let (_cx, mut visual) = panel_harness(detail, Some(panel_role()), 320.);
        let avatar = visual.debug_bounds("CHAT_PANEL_AVATAR architect").unwrap();
        assert_eq!(avatar.size, size(px(40.), px(40.)));
        for text in [
            "COLUMN_TEXT Architect",
            "COLUMN_TEXT @architect",
            "COLUMN_TEXT Claude Code",
            "COLUMN_TEXT fable",
            "COLUMN_TEXT xhigh",
        ] {
            assert!(
                visual.debug_bounds(text).is_some(),
                "missing rendered {text}"
            );
        }
        assert!(visual.debug_bounds("SETUP_ROW RUNTIME").is_some());
        let model = visual.debug_bounds("SETUP_ROW MODEL").unwrap();
        let effort = visual.debug_bounds("SETUP_ROW EFFORT").unwrap();
        assert_eq!(model.top(), effort.top());
        assert_eq!(model.size.width, effort.size.width);
    }

    #[test]
    fn runtime_chat_panel_renders_mark_and_defaults_without_runtime_row() {
        let _theme = crate::theme_snapshot::ThemeGuard::new();
        for variant in [
            crate::theme::ThemeVariant::Carbon,
            crate::theme::ThemeVariant::RunnerLight,
        ] {
            crate::theme::set_active_variant(variant);
            for (model, effort, default_row) in [
                (None, None, "SETUP_ROW EFFORT"),
                (None, Some("xhigh"), "SETUP_ROW MODEL"),
                (Some("gpt-5.6-sol"), None, "SETUP_ROW EFFORT"),
            ] {
                let mut detail = direct_session("codex", true, true);
                detail.agent_model = model.map(str::to_owned);
                detail.agent_effort = effort.map(str::to_owned);
                let (_cx, mut visual) = panel_harness(detail, None, 320.);
                assert!(visual.debug_bounds("CHAT_PANEL_OPEN_ROLE").is_none());
                assert!(visual.debug_bounds("CHAT_PANEL_PROMPT_TEXT").is_none());
                let mark = visual.debug_bounds("CHAT_PANEL_MARK codex").unwrap();
                assert_eq!(mark.size, size(px(40.), px(40.)));
                assert!(visual.debug_bounds("COLUMN_TEXT Codex").is_some());
                assert!(visual.debug_bounds("SETUP_ROW RUNTIME").is_none());
                let model = visual.debug_bounds("SETUP_ROW MODEL").unwrap();
                let effort = visual.debug_bounds("SETUP_ROW EFFORT").unwrap();
                let row = visual.debug_bounds(default_row).unwrap();
                let default = visual.debug_bounds("COLUMN_TEXT default").unwrap();
                assert_eq!(model.top(), effort.top());
                assert_eq!(model.size, effort.size);
                assert!(default.top() >= row.top() && default.bottom() <= row.bottom());
            }
        }
    }

    #[test]
    fn role_chat_panel_renders_effective_runtime_and_command_override() {
        let _theme = crate::theme_snapshot::ThemeGuard::new();
        let mut detail = direct_session("codex", true, true);
        detail.agent_command = "codex-override".into();
        detail.agent_model = Some("gpt-5.6-sol".into());
        detail.agent_effort = Some("high".into());
        let (_cx, mut visual) = panel_harness(detail, Some(panel_role()), 320.);
        assert!(visual.debug_bounds("COLUMN_TEXT Codex").is_some());
        assert!(visual.debug_bounds("COLUMN_TEXT Claude Code").is_none());
        assert!(visual
            .debug_bounds("CHAT_PANEL_META codex-override")
            .is_some());
        assert!(visual.debug_bounds("CHAT_PANEL_META claude").is_none());
        assert!(visual.debug_bounds("COLUMN_TEXT gpt-5.6-sol").is_some());
        assert!(visual.debug_bounds("COLUMN_TEXT high").is_some());
    }

    #[test]
    fn role_chat_panel_renders_chat_cwd_with_legacy_role_fallback() {
        let _theme = crate::theme_snapshot::ThemeGuard::new();
        for (chat_cwd, role_cwd, expected) in [
            (Some("/chat"), Some("/role"), "CHAT_PANEL_META /chat"),
            (Some("/chat"), None, "CHAT_PANEL_META /chat"),
            (None, Some("/role"), "CHAT_PANEL_META /role"),
        ] {
            let mut detail = direct_session("claude-code", true, true);
            detail.cwd = chat_cwd.map(str::to_owned);
            let mut role = panel_role();
            role.working_dir = role_cwd.map(str::to_owned);
            let (_cx, mut visual) = panel_harness(detail, Some(role), 320.);
            assert!(
                visual.debug_bounds(expected).is_some(),
                "missing rendered {expected}"
            );
            if chat_cwd.is_some() {
                assert!(visual.debug_bounds("CHAT_PANEL_META /role").is_none());
            }
        }
    }

    #[test]
    fn chat_panel_identity_and_setup_fit_at_minimum_width() {
        let _theme = crate::theme_snapshot::ThemeGuard::new();
        let mut role = panel_role();
        role.display_name = "A very long architect display name that must truncate".into();
        role.handle = "a-very-long-architect-handle".into();
        let mut detail = direct_session("antigravity", false, false);
        detail.agent_model = Some("a-very-long-model-name-that-must-truncate".into());
        detail.agent_effort = Some("xhigh".into());
        let (_cx, mut visual) =
            panel_harness(detail, Some(role), crate::app_settings::CHAT_PANEL_MIN);
        for text in [
            "COLUMN_TEXT A very long architect display name that must truncate",
            "COLUMN_TEXT @a-very-long-architect-handle",
            "COLUMN_TEXT Antigravity CLI",
            "COLUMN_TEXT a-very-long-model-name-that-must-truncate",
            "COLUMN_TEXT xhigh",
        ] {
            let bounds = visual
                .debug_bounds(text)
                .unwrap_or_else(|| panic!("missing rendered {text}"));
            assert!(bounds.size.width > px(0.), "{text}: {bounds:?}");
            assert!(
                bounds.left() >= px(35.) && bounds.right() <= px(165.),
                "{text}: {bounds:?}"
            );
            assert!(bounds.size.height <= px(24.), "{text} wrapped: {bounds:?}");
        }
    }

    #[test]
    fn split_tab_shows_one_archiving_pill_only_when_every_pane_archives() {
        let single = PaneLayout::single(Some("a"), &["a".into()]);
        assert!(!split_tab_archiving(&single, |_| true));

        let mut split = PaneLayout::single(Some("a"), &["a".into()]);
        let second = split.split("p1", SplitOrientation::Row).unwrap();
        assert!(split_tab_archiving(&split, |_| true));

        split.assign_session(&second, "b").unwrap();
        assert!(split_tab_archiving(&split, |_| true));
        assert!(!split_tab_archiving(&split, |session_id| session_id == "a"));
        assert!(!split_tab_archiving(&split, |_| false));
    }

    #[test]
    fn adjacent_pane_index_wraps_in_both_directions() {
        assert_eq!(adjacent_pane_index(0, 3, -1), Some(2));
        assert_eq!(adjacent_pane_index(2, 3, 1), Some(0));
        assert_eq!(adjacent_pane_index(1, 3, 1), Some(2));
        assert_eq!(adjacent_pane_index(0, 1, 1), None);
        assert_eq!(adjacent_pane_index(3, 3, -1), None);
    }

    #[test]
    fn drop_side_uses_normalized_edges_and_breaks_diagonal_ties_left_then_up() {
        let pane = Bounds::new(point(px(100.), px(200.)), size(px(800.), px(400.)));
        for (x, y, side) in [
            (0.1, 0.5, DropSide::Left),
            (0.9, 0.5, DropSide::Right),
            (0.5, 0.1, DropSide::Up),
            (0.5, 0.9, DropSide::Down),
            (0.375, 0.5, DropSide::Left),
            (0.25, 0.25, DropSide::Left),
            (0.25, 0.75, DropSide::Left),
            (0.75, 0.25, DropSide::Up),
            (0.75, 0.75, DropSide::Right),
            (0.5, 0.5, DropSide::Left),
            (0., 0., DropSide::Left),
            (1., 0., DropSide::Up),
            (0., 1., DropSide::Left),
            (1., 1., DropSide::Right),
        ] {
            assert_eq!(
                drop_side(pane, pane.origin + point(px(800. * x), px(400. * y))),
                side,
                "({x}, {y})"
            );
        }
    }

    #[test]
    fn drop_preview_is_the_destination_half_in_pane_coordinates() {
        let pane = Bounds::new(point(px(100.), px(200.)), size(px(800.), px(400.)));
        for (side, x, y, width, height) in [
            (DropSide::Left, 100., 200., 400., 400.),
            (DropSide::Right, 500., 200., 400., 400.),
            (DropSide::Up, 100., 200., 800., 200.),
            (DropSide::Down, 100., 400., 800., 200.),
        ] {
            assert_eq!(
                drop_preview(pane, side),
                Bounds::new(point(px(x), px(y)), size(px(width), px(height)))
            );
        }
    }

    #[test]
    fn drop_allowed_respects_both_split_axes_at_each_zoom() {
        for zoom in [1., 1.5] {
            for side in [DropSide::Left, DropSide::Right] {
                assert!(drop_allowed(
                    Some(size(px(480. * zoom), px(160. * zoom))),
                    side,
                    zoom
                ));
                assert!(!drop_allowed(
                    Some(size(px(480. * zoom - 1.), px(160. * zoom))),
                    side,
                    zoom
                ));
            }
            for side in [DropSide::Up, DropSide::Down] {
                assert!(drop_allowed(
                    Some(size(px(240. * zoom), px(320. * zoom))),
                    side,
                    zoom
                ));
                assert!(!drop_allowed(
                    Some(size(px(240. * zoom), px(320. * zoom - 1.))),
                    side,
                    zoom
                ));
            }
        }
    }

    #[test]
    fn header_fork_state_uses_capability_key_and_focused_pane_kind() {
        for runtime in ["claude-code", "codex", "pi"] {
            let entry = direct_session(runtime, true, true);
            assert_eq!(
                header_fork_state(Some(&entry), false),
                HeaderForkState::Enabled
            );
        }

        for runtime in ["trae", "copilot", "antigravity"] {
            let entry = direct_session(runtime, false, false);
            assert_eq!(
                header_fork_state(Some(&entry), false),
                HeaderForkState::Disabled(None)
            );
        }
        let waiting = direct_session("codex", true, false);
        assert_eq!(
            header_fork_state(Some(&waiting), false),
            HeaderForkState::Disabled(Some("No session key captured yet"))
        );
        let shell = direct_session("shell", false, false);
        assert_eq!(
            header_fork_state(Some(&shell), false),
            HeaderForkState::Hidden
        );
        assert_eq!(header_fork_state(None, false), HeaderForkState::Hidden);
        assert_eq!(
            header_fork_state(Some(&waiting), true),
            HeaderForkState::Hidden
        );
    }

    #[test]
    fn starting_overlay_names_fork_materialization_before_normal_startup() {
        assert_eq!(
            starting_overlay_label("claude-code", true),
            Some("Forking chat…")
        );
        assert_eq!(starting_overlay_label("codex", false), None);
        assert_eq!(
            starting_overlay_label("shell", false),
            Some("Starting terminal…")
        );
    }

    #[test]
    fn pane_identity_branches_for_chat_terminal_and_empty_panes() {
        assert_eq!(pane_identity_icon(Some("codex")).asset_path(), "openai.svg");
        assert_eq!(
            pane_identity_icon(Some("claude-code")).asset_path(),
            "claude.svg"
        );
        assert_eq!(pane_identity_icon(Some("trae")).asset_path(), "trae.svg");
        assert_eq!(
            pane_identity_icon(Some("copilot")).asset_path(),
            "copilot.svg"
        );
        assert_eq!(pane_identity_icon(Some("pi")).asset_path(), "pi.svg");
        assert_eq!(
            pane_identity_icon(Some("antigravity")).asset_path(),
            "antigravity-icon.png"
        );
        assert_eq!(
            pane_identity_icon(Some("unknown")).asset_path(),
            "message-square.svg"
        );
        assert!(pane_identity_shows_status("codex", false));

        assert_eq!(
            pane_identity_icon(Some("shell")).asset_path(),
            "square-terminal.svg"
        );
        assert!(!pane_identity_shows_status("shell", false));
        assert!(pane_identity_shows_status("shell", true));
        assert!(!side_panel_open(true, true));

        assert_eq!(pane_identity_icon(None).asset_path(), "square-dashed.svg");
        assert_eq!(pane_close_behavior(None), PaneCloseBehavior::LayoutOnly);
    }

    #[test]
    fn only_unfocused_panes_in_a_split_are_faded() {
        assert_eq!(UNFOCUSED_PANE_OPACITY, 0.7);
        assert_eq!(pane_body_opacity(true, false), UNFOCUSED_PANE_OPACITY);
        assert_eq!(pane_body_opacity(true, true), 1.);
        assert_eq!(pane_body_opacity(false, false), 1.);
    }

    #[test]
    fn workspace_header_icon_follows_the_split_then_the_focused_runtime() {
        for (runtime, path) in [
            (Some("shell"), "square-terminal.svg"),
            (Some("claude-code"), "claude.svg"),
            (Some("codex"), "openai.svg"),
            (Some("trae"), "trae.svg"),
            (Some("copilot"), "copilot.svg"),
            (Some("pi"), "pi.svg"),
            (Some("antigravity"), "antigravity-icon.png"),
            (Some("unknown"), "message-square.svg"),
            (None, "message-square.svg"),
        ] {
            assert_eq!(
                workspace_header_icon(true, Some("session"), runtime).asset_path(),
                "columns-2.svg"
            );
            assert_eq!(
                workspace_header_icon(false, Some("session"), runtime).asset_path(),
                path
            );
        }
        assert_eq!(
            workspace_header_icon(false, None, None).asset_path(),
            "square-dashed.svg"
        );
        assert_eq!(
            workspace_header_icon(true, None, None).asset_path(),
            "columns-2.svg"
        );
    }

    #[test]
    fn pane_actions_follow_session_status_and_terminals_cannot_be_archived() {
        let chat = pane_action_items_for(
            "codex",
            true,
            false,
            Some("⇧⌘X".to_owned()),
            Some("⇧⌘R".to_owned()),
        );
        assert_eq!(
            chat.iter()
                .map(|item| item.label.as_ref())
                .collect::<Vec<_>>(),
            ["Stop", "Rename…", "Archive chat"]
        );
        assert_eq!(
            chat[0].shortcut.as_ref().map(|shortcut| shortcut.as_ref()),
            Some("⇧⌘X")
        );
        assert!(chat[2].separator_before);
        assert!(chat.iter().all(|item| item.label.as_ref() != "Close pane"));

        let stopped_chat = pane_action_items_for(
            "codex",
            false,
            false,
            Some("⇧⌘X".to_owned()),
            Some("⇧⌘R".to_owned()),
        );
        assert_eq!(stopped_chat[0].label.as_ref(), "Resume");
        assert_eq!(
            stopped_chat[0]
                .shortcut
                .as_ref()
                .map(|shortcut| shortcut.as_ref()),
            Some("⇧⌘R")
        );

        let terminal = pane_action_items_for(
            "shell",
            false,
            false,
            Some("⇧⌘X".to_owned()),
            Some("⇧⌘R".to_owned()),
        );
        assert_eq!(
            terminal
                .iter()
                .map(|item| item.label.as_ref())
                .collect::<Vec<_>>(),
            ["Rename…"]
        );
        assert_eq!(
            terminal[0]
                .shortcut
                .as_ref()
                .map(|shortcut| shortcut.as_ref()),
            None
        );
        assert_eq!(
            pane_close_behavior(Some("shell")),
            PaneCloseBehavior::CloseTerminal
        );
        assert_eq!(
            pane_close_behavior(Some("codex")),
            PaneCloseBehavior::ArchiveChat
        );
    }

    #[test]
    fn pane_close_archives_chats_closes_terminals_and_drops_empty_panes() {
        assert_eq!(pane_close_behavior(None), PaneCloseBehavior::LayoutOnly);
        assert_eq!(
            pane_close_behavior(Some("shell")),
            PaneCloseBehavior::CloseTerminal
        );
        assert_eq!(
            pane_close_behavior(Some("codex")),
            PaneCloseBehavior::ArchiveChat
        );
        assert_eq!(
            pane_close_behavior(Some("claude-code")),
            PaneCloseBehavior::ArchiveChat
        );
    }

    #[test]
    fn archive_chat_confirm_body_warns_only_while_the_agent_runs() {
        assert_eq!(
            archive_chat_confirm_body("@coder", true),
            "@coder is still running. Archiving stops it. You can restore the chat from Settings → Archived."
        );
        assert_eq!(
            archive_chat_confirm_body("@coder", false),
            "You can restore @coder from Settings → Archived."
        );
        assert_eq!(
            archive_chat_confirm_body("this chat", false),
            "You can restore this chat from Settings → Archived."
        );
    }

    #[test]
    fn pane_identity_renders_only_for_split_tabs() {
        assert!(!pane_identity_visible(1));
        assert!(pane_identity_visible(2));
        assert!(pane_identity_visible(3));
    }

    #[test]
    fn start_surface_badges_follow_default_rebound_and_unbound_bindings() {
        let mut overrides = keymap::KeymapOverrides::new();
        for (binding, mac, windows) in [
            ("new-chat", "⌘N", "Ctrl+N"),
            ("new-terminal", "⌘T", "Ctrl+Shift+T"),
            ("new-mission", "⇧⌘M", "Ctrl+Shift+M"),
        ] {
            let expected = if cfg!(any(windows, target_os = "linux")) {
                windows
            } else {
                mac
            };
            assert_eq!(
                super::start_action_shortcut(binding, &overrides).as_deref(),
                Some(expected)
            );
            let mut rebound = keymap::entry(binding).unwrap().default.clone();
            rebound.meta = false;
            rebound.ctrl = true;
            rebound.shift = false;
            rebound.code = "KeyJ".into();
            rebound.label = None;
            overrides.insert(binding.into(), Some(rebound));
            let expected = if cfg!(any(windows, target_os = "linux")) {
                "Ctrl+J"
            } else {
                "⌃J"
            };
            assert_eq!(
                super::start_action_shortcut(binding, &overrides).as_deref(),
                Some(expected)
            );
            overrides.insert(binding.into(), None);
            assert_eq!(super::start_action_shortcut(binding, &overrides), None);
        }
    }

    #[test]
    fn start_surface_is_shown_for_no_tab_or_a_single_empty_pane() {
        assert!(super::start_surface_visible(None));
        let mut layout = PaneLayout::single(None, &[]);
        assert!(super::start_surface_visible(Some(&layout)));
        layout.split("p1", SplitOrientation::Row).unwrap();
        assert!(!super::start_surface_visible(Some(&layout)));
        let filled = PaneLayout::single(Some("chat"), &[]);
        assert!(!super::start_surface_visible(Some(&filled)));
    }

    #[cfg(unix)]
    #[test]
    fn start_surface_and_split_pane_variants_render_in_their_own_contexts() {
        use crate::{AppRoute, NativeRoot};
        use gpui::{TestAppContext, VisualTestContext};
        use runner_app::pane_layout::TabSet;
        use runner_core::protocol::node::{NodeRow, NodeType};

        let _theme = crate::theme_snapshot::ThemeGuard::new();
        for variant in [
            "no-tab",
            "no-tab-error",
            "single-empty",
            "single-empty-error",
            "single-empty-drawer",
            "split-empty",
            "filled",
        ] {
            let temp = tempfile::tempdir().unwrap();
            let mut cx = TestAppContext::single();
            let store = crate::app_store::test_lifecycle_store(&mut cx, temp.path());
            store.update(&mut cx, |store, cx| {
                crate::app_store::seed_mixed_working_sessions(store);
                store
                    .test_core
                    .db
                    .get()
                    .unwrap()
                    .execute("UPDATE sessions SET status = 'stopped'", [])
                    .unwrap();
                store.refresh(crate::app_store::StoreRefreshKind::All, cx);
            });
            let host = cx.add_window(|window, cx| {
                let mut root = NativeRoot::new(
                    "surface-variants".into(),
                    temp.path().join("logs"),
                    None,
                    None,
                    store.clone(),
                    window,
                    cx,
                );
                let mut layout =
                    PaneLayout::single((variant == "filled").then_some("direct-agent"), &[]);
                if variant == "split-empty" {
                    layout.assign_session("p1", "direct-agent").unwrap();
                    layout.split("p1", SplitOrientation::Row).unwrap();
                }
                if variant == "single-empty-drawer" {
                    layout.add_drawer_shell("busy-shell".into());
                }
                let rows = if variant.starts_with("no-tab") {
                    Vec::new()
                } else {
                    vec![NodeRow {
                        id: layout.id.clone(),
                        parent_id: None,
                        position: 0,
                        node_type: NodeType::Tab,
                        name: None,
                        ref_id: None,
                        layout: Some(layout.serialize().unwrap()),
                        pinned_position: None,
                        last_completed_at: None,
                        last_viewed_at: None,
                        created_at: String::new(),
                    }]
                };
                root.tabs = TabSet::from_rows(&rows);
                root.route = AppRoute::Chat;
                if variant.ends_with("error") {
                    root.chat_error = Some("Shell launch failed".into());
                }
                root
            });
            let mut visual = VisualTestContext::from_window(host.into(), &cx);
            visual.simulate_resize(size(px(1440.), px(900.)));
            visual.run_until_parked();
            assert_eq!(
                visual.debug_bounds("START_SURFACE").is_some(),
                variant.starts_with("no-tab") || variant.starts_with("single-empty"),
                "{variant}"
            );
            assert_eq!(
                visual.debug_bounds("EMPTY_PANE_START").is_some(),
                variant == "split-empty",
                "{variant}"
            );
            assert_eq!(
                visual.debug_bounds("CHAT_ERROR_BANNER").is_some(),
                variant.ends_with("error"),
                "{variant}"
            );
            assert_eq!(
                visual.debug_bounds("CHAT_TERMINAL_DRAWER").is_some(),
                variant == "single-empty-drawer",
                "{variant}"
            );
            if let Some(surface) = visual.debug_bounds("START_SURFACE") {
                assert!(surface.size.height > px(0.), "{variant}");
                assert!(
                    visual.debug_bounds("CHAT_TAB_HEADER").is_none(),
                    "{variant}"
                );
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn start_surface_column_stays_480_pixels_wide_and_centred() {
        use crate::{AppRoute, NativeRoot};
        use gpui::{TestAppContext, VisualTestContext};
        use runner_app::pane_layout::TabSet;

        let _theme = crate::theme_snapshot::ThemeGuard::new();
        let temp = tempfile::tempdir().unwrap();
        let mut cx = TestAppContext::single();
        let store = crate::app_store::test_lifecycle_store(&mut cx, temp.path());
        let host = cx.add_window(|window, cx| {
            let mut root = NativeRoot::new(
                "start-surface-width".into(),
                temp.path().join("logs"),
                None,
                None,
                store.clone(),
                window,
                cx,
            );
            root.tabs = TabSet::default();
            root.route = AppRoute::Chat;
            root
        });
        let mut visual = VisualTestContext::from_window(host.into(), &cx);
        for width in [900., 1440.] {
            visual.simulate_resize(size(px(width), px(900.)));
            visual.run_until_parked();
            let surface = visual.debug_bounds("START_SURFACE").unwrap();
            let column = visual.debug_bounds("START_SURFACE_COLUMN").unwrap();
            assert_eq!(column.size.width, px(480.), "window width {width}");
            assert_eq!(
                column.center().x,
                surface.center().x,
                "window width {width}"
            );
        }
    }

    #[test]
    fn terminal_drawer_tooltip_tracks_rebound_and_unbound_shortcuts() {
        let mut overrides = keymap::KeymapOverrides::new();
        #[cfg(target_os = "macos")]
        assert_eq!(
            terminal_drawer_tooltip(false, &overrides),
            "Show terminal drawer · ⌥F12"
        );
        #[cfg(any(windows, target_os = "linux"))]
        assert_eq!(
            terminal_drawer_tooltip(false, &overrides),
            "Show terminal drawer · Alt+F12"
        );
        #[cfg(target_os = "macos")]
        assert_eq!(
            terminal_drawer_tooltip(true, &overrides),
            "Hide terminal drawer · ⌥F12"
        );
        #[cfg(any(windows, target_os = "linux"))]
        assert_eq!(
            terminal_drawer_tooltip(true, &overrides),
            "Hide terminal drawer · Alt+F12"
        );

        let mut rebound = keymap::entry("toggle-terminal-drawer")
            .unwrap()
            .default
            .clone();
        rebound.alt = false;
        rebound.ctrl = true;
        rebound.code = "Backquote".into();
        overrides.insert("toggle-terminal-drawer".into(), Some(rebound));
        #[cfg(target_os = "macos")]
        assert_eq!(
            terminal_drawer_tooltip(false, &overrides),
            "Show terminal drawer · ⌃`"
        );
        #[cfg(any(windows, target_os = "linux"))]
        assert_eq!(
            terminal_drawer_tooltip(false, &overrides),
            "Show terminal drawer · Ctrl+`"
        );

        overrides.insert("toggle-terminal-drawer".into(), None);
        assert_eq!(
            terminal_drawer_tooltip(false, &overrides),
            "Show terminal drawer"
        );
    }

    #[test]
    fn a_split_needs_room_for_both_halves_and_the_floor_scales_with_zoom() {
        let wide = size(
            px(2. * MIN_SPLIT_PANE_WIDTH),
            px(2. * MIN_SPLIT_PANE_HEIGHT),
        );
        assert!(split_allowed(Some(wide), SplitOrientation::Row, 1.));
        assert!(split_allowed(Some(wide), SplitOrientation::Column, 1.));

        let narrow = size(
            px(2. * MIN_SPLIT_PANE_WIDTH - 1.),
            px(2. * MIN_SPLIT_PANE_HEIGHT),
        );
        assert!(!split_allowed(Some(narrow), SplitOrientation::Row, 1.));
        assert!(split_allowed(Some(narrow), SplitOrientation::Column, 1.));

        let short = size(
            px(2. * MIN_SPLIT_PANE_WIDTH),
            px(2. * MIN_SPLIT_PANE_HEIGHT - 1.),
        );
        assert!(split_allowed(Some(short), SplitOrientation::Row, 1.));
        assert!(!split_allowed(Some(short), SplitOrientation::Column, 1.));

        // The same pixels are too small once the app is zoomed in.
        assert!(!split_allowed(Some(wide), SplitOrientation::Row, 1.5));
        assert!(!split_allowed(Some(wide), SplitOrientation::Column, 1.5));
        let zoomed = size(
            px(3. * MIN_SPLIT_PANE_WIDTH),
            px(3. * MIN_SPLIT_PANE_HEIGHT),
        );
        assert!(split_allowed(Some(zoomed), SplitOrientation::Row, 1.5));
        assert!(split_allowed(Some(zoomed), SplitOrientation::Column, 1.5));

        // An unmeasured pane is not disabled on a size we do not have.
        assert!(split_allowed(None, SplitOrientation::Row, 1.));
    }

    // `split_pane` is the single gate both routes pass through — the menu
    // item and `⌘D` / `⇧⌘D` alike — so what it decides is what is pinned here.
    #[test]
    fn chat_and_terminal_splits_meet_the_same_floor_on_every_route() {
        for zoom in [1., 1.5] {
            let roomy = size(
                px(2. * MIN_SPLIT_PANE_WIDTH * zoom),
                px(2. * MIN_SPLIT_PANE_HEIGHT * zoom),
            );
            let narrow = size(roomy.width - px(1.), roomy.height);
            let short = size(roomy.width, roomy.height - px(1.));
            for orientation in [SplitOrientation::Row, SplitOrientation::Column] {
                assert_eq!(
                    split_decision(Some(roomy), orientation, zoom),
                    SplitDecision::Allowed
                );
                assert_eq!(
                    split_decision(None, orientation, zoom),
                    SplitDecision::Allowed
                );
            }
            assert_eq!(
                split_decision(Some(narrow), SplitOrientation::Row, zoom),
                SplitDecision::Blocked(crate::i18n::text(TOO_SMALL_TO_SPLIT))
            );
            assert_eq!(
                split_decision(Some(narrow), SplitOrientation::Column, zoom),
                SplitDecision::Allowed
            );
            assert_eq!(
                split_decision(Some(short), SplitOrientation::Column, zoom),
                SplitDecision::Blocked(crate::i18n::text(TOO_SMALL_TO_SPLIT))
            );
            assert_eq!(
                split_decision(Some(short), SplitOrientation::Row, zoom),
                SplitDecision::Allowed
            );
        }
    }

    #[test]
    fn a_disabled_menu_item_carries_the_reason_the_action_path_would_report() {
        let narrow = size(px(300.), px(800.));
        let items = split_menu_items(Some(narrow), 1., &keymap::KeymapOverrides::new());
        let SplitDecision::Blocked(reason) =
            split_decision(Some(narrow), SplitOrientation::Row, 1.)
        else {
            panic!("a 300 px pane cannot split to the right");
        };

        assert!(items[0].disabled);
        assert_eq!(
            items[0].tooltip.as_ref().map(|tip| tip.as_ref()),
            Some(reason)
        );
    }

    #[test]
    fn the_split_menu_offers_two_directions_with_live_pills_and_says_why_it_is_off() {
        let mut overrides = keymap::KeymapOverrides::new();
        let roomy = size(px(1200.), px(800.));
        let items = split_menu_items(Some(roomy), 1., &overrides);
        assert_eq!(
            items
                .iter()
                .map(|item| item.label.as_ref())
                .collect::<Vec<_>>(),
            ["Split Right", "Split Down"]
        );
        assert!(items.iter().all(|item| !item.disabled));
        assert!(items.iter().all(|item| item.tooltip.is_none()));
        #[cfg(target_os = "macos")]
        assert_eq!(
            items
                .iter()
                .map(|item| item.shortcut.as_ref().map(|pill| pill.as_ref()))
                .collect::<Vec<_>>(),
            [Some("⌘D"), Some("⇧⌘D")]
        );

        overrides.insert("split-pane-right".into(), None);
        let unbound = split_menu_items(Some(roomy), 1., &overrides);
        assert_eq!(unbound[0].shortcut, None);
        assert!(unbound[1].shortcut.is_some());

        let cramped = size(px(300.), px(800.));
        let gated = split_menu_items(Some(cramped), 1., &keymap::KeymapOverrides::new());
        assert!(gated[0].disabled);
        assert_eq!(
            gated[0].tooltip.as_ref().map(|tip| tip.as_ref()),
            Some(crate::i18n::text(TOO_SMALL_TO_SPLIT))
        );
        assert!(!gated[1].disabled);
        assert_eq!(gated[1].tooltip, None);
    }

    #[test]
    fn pane_rename_keys_submit_or_revert_inline_edits() {
        assert_eq!(pane_rename_key("enter"), Some(PaneRenameKey::Submit));
        assert_eq!(pane_rename_key("escape"), Some(PaneRenameKey::Cancel));
        assert_eq!(pane_rename_key("tab"), None);
    }

    #[test]
    fn dropped_files_focus_the_pane_they_land_on() {
        use crate::{AppRoute, NativeRoot};
        use gpui::{ExternalPaths, FileDropEvent, TestAppContext, VisualTestContext};
        use std::sync::Arc;

        let _theme = crate::theme_snapshot::ThemeGuard::new();
        let temp = tempfile::tempdir().unwrap();
        let mut cx = TestAppContext::single();
        let store = crate::app_store::test_lifecycle_store(&mut cx, temp.path());
        let target_pane = store.update(&mut cx, |store, cx| {
            crate::app_store::seed_mixed_working_sessions(store);
            let core = store.test_core.clone();
            let events: Arc<dyn runner_daemon::session::manager::SessionEvents> =
                Arc::new(core.session_events());
            for id in ["busy-shell", "direct-agent"] {
                core.sessions
                    .prepare_unlisted_terminal(id, (80, 24), &core.db, &events)
                    .unwrap();
                store.bridge.attach(id).unwrap();
            }
            let mut layout = PaneLayout::single(Some("busy-shell"), &[]);
            let target_pane = layout.split("p1", SplitOrientation::Row).unwrap();
            layout.assign_session(&target_pane, "direct-agent").unwrap();
            assert!(layout.focus_pane("p1"));
            let conn = core.db.get().unwrap();
            runner_daemon::repo::node::create_tab(&conn, None, "", 0, &layout.serialize().unwrap())
                .unwrap();
            store.replace_nodes(runner_daemon::repo::node::list(&conn).unwrap(), cx);
            target_pane
        });
        let host = cx.add_window(|window, cx| {
            let mut root = NativeRoot::new(
                "file-drop-test".into(),
                temp.path().join("logs"),
                None,
                None,
                store.clone(),
                window,
                cx,
            );
            root.apply_tab_rows(cx);
            assert!(root.tabs.activate_session("busy-shell"));
            root.route = AppRoute::Chat;
            root.sync_active_chat_detail(cx);
            root.ensure_active_tab_attached(window, cx).unwrap();
            root
        });
        let mut visual = VisualTestContext::from_window(host.into(), &cx);
        visual.simulate_resize(size(px(1440.), px(900.)));
        visual.run_until_parked();
        host.update(&mut cx, |root, _, _| {
            assert_eq!(root.tabs.active().unwrap().focused_pane_id, "p1");
        })
        .unwrap();

        let target = visual
            .debug_bounds("TERMINAL_FILE_DROP direct-agent")
            .expect("an interactive terminal is a drop target");
        visual.simulate_event(FileDropEvent::Entered {
            position: target.center(),
            paths: ExternalPaths(["/tmp/two words.png"].into_iter().map(Into::into).collect()),
        });
        visual.simulate_event(FileDropEvent::Submit {
            position: target.center(),
        });
        visual.run_until_parked();

        host.update(&mut cx, |root, window, _| {
            assert_eq!(root.tabs.active().unwrap().focused_pane_id, target_pane);
            assert!(root.attached["direct-agent"]
                .terminal_focus
                .is_focused(window));
            // No process backs the test session, so the daemon refuses the
            // write; the refusal names the terminal the paths were sent to.
            assert_eq!(
                root.chat_error,
                Some("session not found: direct-agent".into())
            );
        })
        .unwrap();
    }
}
