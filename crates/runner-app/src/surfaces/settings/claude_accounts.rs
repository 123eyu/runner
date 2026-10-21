use crate::{app_store::AppStore, theme};
use gpui::{
    div, prelude::*, px, rems, svg, AnyElement, Context, Entity, FontWeight, KeyDownEvent,
    PathPromptOptions, Render, SharedString, Subscription, WeakEntity, Window,
};
use runner_app::ui::menu::{MenuItem, PopoverMenu};
use runner_app::ui::{
    Badge, BrowseField, Button, ButtonSize, ButtonVariant, ConfirmDialog, TextField, Tone,
};
use runner_core::protocol::{ClaudeAccount, ClaudeAccountAction, ClaudeAccounts};
use std::{collections::HashMap, path::PathBuf, rc::Rc};

pub(crate) fn account_error(error: &str) -> String {
    let key = match error {
        "account name is required" => "accounts.error_name",
        "account folder must be absolute" => "accounts.error_folder",
        "this folder already has an account" => "accounts.error_duplicate",
        "account not found" | "active account missing" => "accounts.error_missing",
        "the main, active and original native accounts cannot be removed" => {
            "accounts.cannot_remove"
        }
        "sign in before using this account" => "accounts.error_sign_in",
        "Claude Code is not installed" => "accounts.error_install",
        "main account change is incomplete; retry Make main in Settings" => {
            "accounts.finish_main_change"
        }
        _ => return error.to_owned(),
    };
    crate::i18n::text(key).to_owned()
}

type RevealFolder = Rc<dyn Fn(&std::path::Path, &mut gpui::App)>;

pub(crate) struct AccountsSection {
    store: Entity<AppStore>,
    name: Entity<TextField>,
    folder: Entity<TextField>,
    adding: bool,
    use_main_settings: bool,
    share_confirmation: Option<String>,
    overlay: Entity<AccountSharingOverlay>,
    renaming: Option<String>,
    saving: bool,
    error: Option<String>,
    menus: HashMap<String, Entity<PopoverMenu>>,
    reveal_folder: RevealFolder,
    _subscriptions: Vec<Subscription>,
}

impl AccountsSection {
    pub(crate) fn new(store: Entity<AppStore>, cx: &mut Context<Self>) -> Self {
        let name = cx.new(|cx| {
            TextField::new(
                cx.focus_handle(),
                "",
                crate::i18n::message("accounts.name"),
                false,
            )
        });
        let folder = cx.new(|cx| {
            TextField::new(
                cx.focus_handle(),
                "",
                crate::i18n::message("accounts.auto_folder"),
                true,
            )
        });
        let folder_for_name = folder.clone();
        let subscriptions = vec![
            cx.observe(&store, |_, _, cx| cx.notify()),
            cx.observe(&name, move |_, name, cx| {
                let folder_name =
                    runner_core::protocol::claude_account_folder_name(name.read(cx).text());
                folder_for_name.update(cx, |folder, cx| {
                    folder.set_placeholder(
                        crate::i18n::tf(
                            "accounts.auto_path",
                            &[("path", format!("~/{folder_name}"))],
                        ),
                        cx,
                    )
                });
                cx.notify();
            }),
        ];
        let pane = cx.entity();
        let overlay = cx.new(|overlay_cx| AccountSharingOverlay::new(pane, overlay_cx));
        Self {
            use_main_settings: true,
            share_confirmation: None,
            overlay,
            store,
            name,
            folder,
            adding: false,
            renaming: None,
            saving: false,
            error: None,
            menus: HashMap::new(),
            reveal_folder: Rc::new(|path, cx| cx.reveal_path(path)),
            _subscriptions: subscriptions,
        }
    }

    pub(crate) fn is_composing(&self, cx: &gpui::App) -> bool {
        self.name.read(cx).is_composing() || self.folder.read(cx).is_composing()
    }

    pub(crate) fn refresh(&mut self, cx: &mut Context<Self>) {
        self.perform(ClaudeAccountAction::Refresh, cx);
    }

    fn perform(&mut self, action: ClaudeAccountAction, cx: &mut Context<Self>) {
        if self.saving {
            return;
        }
        self.saving = true;
        let client = self.store.read(cx).client.clone();
        let task = cx.background_spawn(async move {
            client
                .claude_account_action(action)
                .map_err(|error| error.to_string())
        });
        cx.spawn(async move |weak, cx| {
            let result = task.await;
            let _ = weak.update(cx, |this, cx| {
                this.saving = false;
                match result {
                    Ok(accounts) => {
                        this.store.update(cx, |store, cx| {
                            store.claude_accounts = accounts;
                            store.refresh(crate::app_store::StoreRefreshKind::Activity, cx);
                        });
                        this.share_confirmation = None;
                        this.adding = false;
                        this.renaming = None;
                        this.error = None;
                    }
                    Err(error) => this.error = Some(account_error(&error)),
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    fn browse(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let selected = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some(crate::i18n::text("accounts.folder").into()),
        });
        cx.spawn(async move |weak, cx| {
            if let Ok(Ok(Some(paths))) = selected.await {
                if let Some(path) = paths.first() {
                    let _ = weak.update(cx, |this, cx| {
                        this.folder
                            .update(cx, |field, cx| field.set_text(path.to_string_lossy(), cx))
                    });
                }
            }
        })
        .detach();
    }

    fn menu_items(account: &ClaudeAccount, accounts: &ClaudeAccounts) -> Vec<MenuItem> {
        let changing_main = accounts.pending_main_id.is_some();
        let protected = account.native
            || account.id == accounts.main_id
            || account.id == accounts.active_id
            || changing_main;
        let remove = MenuItem::new(crate::i18n::message("ui.remove"))
            .icon("trash.svg")
            .separator_before(true)
            .destructive(true)
            .disabled(protected);
        let remove = if protected {
            remove.tooltip(crate::i18n::message(if changing_main {
                "accounts.finish_main_change"
            } else {
                "accounts.cannot_remove"
            }))
        } else {
            remove
        };
        let mut items = vec![
            MenuItem::new(crate::i18n::message("accounts.make_main"))
                .icon("star.svg")
                .disabled(
                    account.id == accounts.main_id
                        || accounts
                            .pending_main_id
                            .as_ref()
                            .is_some_and(|id| id != &account.id),
                ),
            MenuItem::new(crate::i18n::message("ui.rename")).icon("pencil.svg"),
            MenuItem::new(crate::i18n::message(if cfg!(windows) {
                "ui.reveal_in_explorer"
            } else {
                "ui.reveal_in_finder"
            }))
            .icon("folder.svg"),
            remove,
        ];
        if accounts.sharing.contains_key(&account.id) {
            items.insert(
                1,
                MenuItem::new(crate::i18n::tf(
                    "accounts.stop_sharing",
                    &[("main", main_name(accounts))],
                ))
                .icon("link.svg")
                .disabled(changing_main),
            );
        }
        items
    }

    pub(crate) fn confirm_overlay(&self) -> Entity<AccountSharingOverlay> {
        self.overlay.clone()
    }

    fn account_menu(
        &mut self,
        account: &ClaudeAccount,
        accounts: &ClaudeAccounts,
        cx: &mut Context<Self>,
    ) -> Entity<PopoverMenu> {
        let items = Self::menu_items(account, accounts);
        let linked = accounts.sharing.contains_key(&account.id);
        let owner = cx.weak_entity();
        let account = account.clone();
        let id = account.id.clone();
        let handler: runner_app::ui::menu::MenuHandler = Rc::new(
            move |index: usize, window: &mut Window, cx: &mut gpui::App| {
                let _ = owner.update(cx, |this, cx| {
                    if linked && index == 1 {
                        this.perform(
                            ClaudeAccountAction::StopSharing {
                                id: account.id.clone(),
                            },
                            cx,
                        );
                        return;
                    }
                    let index = if linked && index > 1 {
                        index - 1
                    } else {
                        index
                    };
                    match index {
                        0 => this.perform(
                            ClaudeAccountAction::MakeMain {
                                id: account.id.clone(),
                            },
                            cx,
                        ),
                        1 => {
                            this.renaming = Some(account.id.clone());
                            this.adding = false;
                            let name = this
                                .store
                                .read(cx)
                                .claude_accounts
                                .accounts
                                .iter()
                                .find(|current| current.id == account.id)
                                .map(|account| account.name.clone())
                                .unwrap_or_else(|| account.name.clone());
                            this.name.update(cx, |field, cx| field.set_text(name, cx));
                            this.name.read(cx).focus_handle().focus(window, cx);
                            cx.notify();
                        }
                        2 => (this.reveal_folder)(&PathBuf::from(&account.folder), cx),
                        3 => this.perform(
                            ClaudeAccountAction::Remove {
                                id: account.id.clone(),
                            },
                            cx,
                        ),
                        _ => {}
                    }
                });
            },
        );
        if let Some(menu) = self.menus.get(&id) {
            menu.update(cx, |menu, cx| {
                menu.set_items_and_handler(items, handler, cx)
            });
            return menu.clone();
        }
        let menu = cx.new(|cx| {
            PopoverMenu::new(
                SharedString::from(format!("account-menu-{id}")),
                cx.focus_handle(),
                items,
                handler,
                cx,
            )
            .min_width(px(280.))
            .single_line_labels()
            .align_right()
        });
        self.menus.insert(id, menu.clone());
        menu
    }

    fn row(
        &mut self,
        account: &ClaudeAccount,
        accounts: &ClaudeAccounts,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let known = accounts.statuses.contains_key(&account.id);
        let status = accounts
            .statuses
            .get(&account.id)
            .cloned()
            .unwrap_or_default();
        let menu = self.account_menu(account, accounts, cx);
        let owner = cx.entity();
        let id = account.id.clone();
        let signing_in = status.signing_in;
        let signed_in = status.logged_in;
        let identity = [status.email, status.organization, status.plan]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" · ");
        let sharing = accounts.sharing.get(&account.id);
        let main = main_name(accounts);
        let owner_share = cx.entity();
        let share_id = id.clone();
        div()
            .debug_selector({
                let id = id.clone();
                move || format!("CLAUDE_ACCOUNT_{id}")
            })
            .px_5()
            .py_4()
            .border_b_1()
            .border_color(theme::border())
            .flex()
            .items_center()
            .gap_3()
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .text_size(theme::text_body())
                            .child(account.name.clone())
                            .children((account.id == accounts.main_id).then(|| {
                                let id = account.id.clone();
                                div()
                                    .debug_selector(move || format!("CLAUDE_ACCOUNT_MAIN_{id}"))
                                    .child(Badge::new(
                                        crate::i18n::text("accounts.main"),
                                        Tone::Muted,
                                    ))
                            }))
                            .children((account.id == accounts.active_id).then(|| {
                                let id = account.id.clone();
                                div()
                                    .debug_selector(move || format!("CLAUDE_ACCOUNT_ACTIVE_{id}"))
                                    .child(Badge::new(
                                        crate::i18n::text("accounts.active"),
                                        Tone::Accent,
                                    ))
                            })),
                    )
                    .children((!identity.is_empty()).then(|| {
                        div()
                            .text_size(theme::text_ui())
                            .text_color(theme::muted())
                            .child(identity)
                    }))
                    .child(
                        div()
                            .text_size(theme::text_meta())
                            .text_color(theme::faint())
                            .flex()
                            .items_center()
                            .gap_2()
                            .flex_wrap()
                            .child(account.folder.clone())
                            .children(
                                (sharing.is_none() && account.id != accounts.main_id).then(|| {
                                    div().child(crate::i18n::text("accounts.own_settings"))
                                }),
                            )
                            .children(sharing.map(|sharing| {
                                let mut label = if sharing.using_main {
                                    crate::i18n::tf(
                                        "accounts.using_settings",
                                        &[("main", main.clone())],
                                    )
                                } else {
                                    crate::i18n::text("accounts.sharing_pending").to_owned()
                                };
                                if sharing.using_main {
                                    for (key, count) in [
                                        ("accounts.shared_skills", sharing.shared_skills),
                                        ("accounts.shared_plugins", sharing.shared_plugins),
                                    ] {
                                        if count > 0 {
                                            label.push_str(" · ");
                                            label.push_str(&crate::i18n::tn(key, count));
                                        }
                                    }
                                }
                                for item in &sharing.own_items {
                                    label.push_str(" · ");
                                    label.push_str(&match item.as_str() {
                                        "skills" => {
                                            crate::i18n::text("accounts.local_skills").to_owned()
                                        }
                                        "plugins" => {
                                            crate::i18n::text("accounts.local_plugins").to_owned()
                                        }
                                        _ => crate::i18n::tf(
                                            "accounts.own_item",
                                            &[("item", item.clone())],
                                        ),
                                    });
                                }
                                div().child(label)
                            }))
                            .children((account.id != accounts.main_id && sharing.is_none()).then(
                                || {
                                    Button::new(
                                        SharedString::from(format!("account-share-{share_id}")),
                                        crate::i18n::tf("accounts.use_settings", &[("main", main)]),
                                    )
                                    .icon("link.svg")
                                    .variant(ButtonVariant::Ghost)
                                    .size(ButtonSize::Sm)
                                    .disabled(self.saving || accounts.pending_main_id.is_some())
                                    .on_press(
                                        move |window, cx| {
                                            owner_share.update(cx, |this, cx| {
                                                this.share_confirmation = Some(share_id.clone());
                                                this.name.read(cx).focus_handle().focus(window, cx);
                                                cx.notify();
                                            })
                                        },
                                    )
                                },
                            )),
                    ),
            )
            .child(
                div()
                    .text_size(theme::text_ui())
                    .text_color(if signed_in {
                        theme::accent()
                    } else {
                        theme::muted()
                    })
                    .child(crate::i18n::text(if !known {
                        "ui.checking"
                    } else if signing_in {
                        "accounts.signing_in"
                    } else if signed_in {
                        "accounts.signed_in"
                    } else if status.error {
                        "accounts.status_unavailable"
                    } else {
                        "accounts.not_signed_in"
                    })),
            )
            .children(if signing_in {
                let url = status.sign_in_url;
                Some(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            Button::new(
                                SharedString::from(format!("account-open-{id}")),
                                crate::i18n::text("accounts.open_sign_in"),
                            )
                            .size(ButtonSize::Sm)
                            .disabled(url.is_none())
                            .on_press(move |_, cx| {
                                if let Some(url) = &url {
                                    cx.open_url(url);
                                }
                            }),
                        )
                        .child(
                            Button::new(
                                SharedString::from(format!("account-cancel-{id}")),
                                crate::i18n::text("ui.cancel"),
                            )
                            .size(ButtonSize::Sm)
                            .variant(ButtonVariant::Ghost)
                            .on_press(move |_, cx| {
                                owner.update(cx, |this, cx| {
                                    this.perform(ClaudeAccountAction::Cancel { id: id.clone() }, cx)
                                })
                            }),
                        ),
                )
            } else if account.id != accounts.active_id || !signed_in {
                Some(
                    div().child(
                        Button::new(
                            SharedString::from(format!("account-use-{id}")),
                            crate::i18n::text(if signed_in {
                                "accounts.use"
                            } else {
                                "accounts.log_in"
                            }),
                        )
                        .size(ButtonSize::Sm)
                        .disabled(self.saving)
                        .on_press(move |_, cx| {
                            owner.update(cx, |this, cx| {
                                this.perform(
                                    if signed_in {
                                        ClaudeAccountAction::Use { id: id.clone() }
                                    } else {
                                        ClaudeAccountAction::Login { id: id.clone() }
                                    },
                                    cx,
                                )
                            })
                        }),
                    ),
                )
            } else {
                None
            })
            .child(menu)
            .into_any_element()
    }

    fn form(&self, cx: &mut Context<Self>) -> AnyElement {
        let owner = cx.entity();
        let cancel = owner.clone();
        let browse = owner.clone();
        let rename = self.renaming.clone();
        let checkbox_owner = cx.entity();
        let use_main = self.use_main_settings;
        let main = main_name(&self.store.read(cx).claude_accounts);
        div()
            .p_5()
            .bg(theme::raised())
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .flex()
                    .gap_3()
                    .child(
                        div()
                            .w(rems(210. / 16.))
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(crate::i18n::text("accounts.name"))
                            .child(self.name.clone()),
                    )
                    .children(rename.is_none().then(|| {
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(crate::i18n::text("accounts.folder"))
                            .child(
                                BrowseField::new(
                                    self.folder.clone(),
                                    self.saving,
                                    Rc::new(move |window, cx| {
                                        browse.update(cx, |this, cx| this.browse(window, cx))
                                    }),
                                )
                                .browse_id("claude-account-browse"),
                            )
                    })),
            )
            .children(rename.is_none().then(|| {
                div()
                    .id("account-use-main-settings")
                    .debug_selector(|| "CLAUDE_ACCOUNT_SHARE_CHECKBOX".into())
                    .tab_index(0)
                    .tab_stop(!self.saving)
                    .flex()
                    .items_center()
                    .gap_2()
                    .cursor_pointer()
                    .child(
                        div()
                            .size(rems(1.))
                            .rounded_sm()
                            .border_1()
                            .border_color(theme::border_strong())
                            .bg(if use_main {
                                theme::accent()
                            } else {
                                theme::panel()
                            })
                            .children(use_main.then(|| {
                                svg()
                                    .path("check.svg")
                                    .size(rems(14. / 16.))
                                    .text_color(theme::panel())
                            })),
                    )
                    .child(crate::i18n::tf("accounts.use_settings", &[("main", main)]))
                    .on_click(move |_, _, cx| {
                        checkbox_owner.update(cx, |this, cx| {
                            if !this.saving {
                                this.use_main_settings = !this.use_main_settings;
                                cx.notify();
                            }
                        })
                    })
                    .on_key_down(|event: &KeyDownEvent, _, cx| {
                        if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                            cx.stop_propagation();
                        }
                    })
            }))
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap_2()
                    .child(
                        Button::new("account-form-cancel", crate::i18n::text("ui.cancel"))
                            .variant(ButtonVariant::Ghost)
                            .on_press(move |_, cx| {
                                cancel.update(cx, |this, cx| {
                                    this.adding = false;
                                    this.renaming = None;
                                    cx.notify();
                                })
                            }),
                    )
                    .child(
                        Button::new(
                            "account-form-save",
                            crate::i18n::text(if rename.is_some() {
                                "ui.save"
                            } else {
                                "accounts.add_and_login"
                            }),
                        )
                        .variant(ButtonVariant::Primary)
                        .disabled(self.saving)
                        .on_press(move |_, cx| {
                            owner.update(cx, |this, cx| {
                                let name = this.name.read(cx).text().to_owned();
                                let folder = this.folder.read(cx).text().trim().to_owned();
                                this.perform(
                                    if let Some(id) = &rename {
                                        ClaudeAccountAction::Rename {
                                            id: id.clone(),
                                            name,
                                        }
                                    } else {
                                        ClaudeAccountAction::AddAndLogin {
                                            name,
                                            folder: (!folder.is_empty()).then_some(folder),
                                            use_main_settings: this.use_main_settings,
                                        }
                                    },
                                    cx,
                                );
                            })
                        }),
                    ),
            )
            .into_any_element()
    }
}

impl Render for AccountsSection {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let accounts = self.store.read(cx).claude_accounts.clone();
        self.menus
            .retain(|id, _| accounts.accounts.iter().any(|account| &account.id == id));
        let owner = cx.entity();
        let retry_owner = owner.clone();
        let rows = accounts
            .accounts
            .iter()
            .map(|account| self.row(account, &accounts, cx))
            .collect::<Vec<_>>();
        div()
            .debug_selector(|| "CLAUDE_ACCOUNTS".into())
            .w_full()
            .min_w_0()
            .flex()
            .flex_col()
            .gap_3()
            .text_size(theme::text_ui())
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(theme::text_body())
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(crate::i18n::text("accounts.title")),
                    )
                    .child(
                        Button::new("claude-add-account", crate::i18n::text("accounts.add"))
                            .icon("plus.svg")
                            .size(ButtonSize::Sm)
                            .disabled(
                                self.adding || self.saving || accounts.pending_main_id.is_some(),
                            )
                            .on_press(move |window, cx| {
                                owner.update(cx, |this, cx| {
                                    this.adding = true;
                                    this.use_main_settings = true;
                                    this.renaming = None;
                                    this.name.update(cx, |field, cx| field.set_text("", cx));
                                    this.folder.update(cx, |field, cx| field.set_text("", cx));
                                    this.name.read(cx).focus_handle().focus(window, cx);
                                    cx.notify();
                                })
                            }),
                    ),
            )
            .child(
                div()
                    .text_color(theme::muted())
                    .child(crate::i18n::text("accounts.description")),
            )
            .children(accounts.pending_main_id.as_ref().map(|id| {
                let id = id.clone();
                let target = accounts
                    .accounts
                    .iter()
                    .find(|account| account.id == id)
                    .map(|account| account.name.clone())
                    .unwrap_or_default();
                div()
                    .debug_selector(|| "CLAUDE_MAIN_CHANGE_PENDING".into())
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .flex_1()
                            .text_color(theme::warning())
                            .child(crate::i18n::tf(
                                "accounts.main_change_pending",
                                &[("account", target)],
                            )),
                    )
                    .child(
                        Button::new("claude-retry-main-change", crate::i18n::text("ui.retry"))
                            .size(ButtonSize::Sm)
                            .disabled(self.saving)
                            .on_press(move |_, cx| {
                                retry_owner.update(cx, |this, cx| {
                                    this.perform(
                                        ClaudeAccountAction::MakeMain { id: id.clone() },
                                        cx,
                                    )
                                })
                            }),
                    )
            }))
            .child(
                div()
                    .rounded_lg()
                    .border_1()
                    .border_color(theme::border_strong())
                    .bg(theme::panel())
                    .overflow_hidden()
                    .children(rows)
                    .children((self.adding || self.renaming.is_some()).then(|| self.form(cx))),
            )
            .children(
                self.error
                    .as_ref()
                    .map(|error| div().text_color(theme::danger()).child(error.clone())),
            )
            .child(
                div()
                    .text_color(theme::faint())
                    .child(crate::i18n::text("accounts.keep_folder")),
            )
    }
}

fn main_name(accounts: &ClaudeAccounts) -> String {
    accounts
        .accounts
        .iter()
        .find(|account| account.id == accounts.main_id)
        .map(|account| account.name.clone())
        .unwrap_or_default()
}

pub(crate) struct AccountSharingOverlay {
    pane: WeakEntity<AccountsSection>,
    _subscription: Subscription,
}

impl AccountSharingOverlay {
    fn new(pane: Entity<AccountsSection>, cx: &mut Context<Self>) -> Self {
        let subscription = cx.observe(&pane, |_, _, cx| cx.notify());
        Self {
            pane: pane.downgrade(),
            _subscription: subscription,
        }
    }
}

impl Render for AccountSharingOverlay {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(pane) = self.pane.upgrade() else {
            return div().into_any_element();
        };
        let state = pane.read(cx);
        let Some(id) = state.share_confirmation.clone() else {
            return div().into_any_element();
        };
        let accounts = &state.store.read(cx).claude_accounts;
        let Some(account) = accounts.accounts.iter().find(|account| account.id == id) else {
            return div().into_any_element();
        };
        let parameters = [
            ("main", main_name(accounts)),
            ("account", account.name.clone()),
        ];
        let cancel = pane.clone();
        ConfirmDialog::new(
            crate::i18n::tf("accounts.share_title", &parameters),
            crate::i18n::tf("accounts.share_body", &parameters),
            crate::i18n::tf("accounts.use_settings", &parameters),
            crate::i18n::text("ui.saving"),
            state.saving,
            Rc::new(move |_, cx| {
                pane.update(cx, |this, cx| {
                    this.perform(ClaudeAccountAction::Share { id: id.clone() }, cx)
                })
            }),
            Rc::new(move |_, cx| {
                cancel.update(cx, |this, cx| {
                    this.share_confirmation = None;
                    cx.notify();
                })
            }),
        )
        .icon("link.svg")
        .variant(ButtonVariant::Primary)
        .focus_handle(state.name.read(cx).focus_handle())
        .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{px, Focusable, VisualTestContext};

    fn accounts() -> ClaudeAccounts {
        ClaudeAccounts {
            accounts: vec![
                ClaudeAccount {
                    id: "native".into(),
                    name: "Personal".into(),
                    folder: "/tmp/native".into(),
                    native: true,
                },
                ClaudeAccount {
                    id: "work".into(),
                    name: "Work".into(),
                    folder: "/tmp/work".into(),
                    native: false,
                },
                ClaudeAccount {
                    id: "other".into(),
                    name: "Other".into(),
                    folder: "/tmp/other".into(),
                    native: false,
                },
            ],
            main_id: "native".into(),
            active_id: "work".into(),
            sharing: Default::default(),
            pending_main_id: None,
            statuses: Default::default(),
        }
    }

    #[test]
    fn account_menu_protects_main_active_and_native_rows() {
        let mut accounts = accounts();
        for account in &accounts.accounts {
            let items = AccountsSection::menu_items(account, &accounts);
            assert!(items.iter().all(|item| item.description.is_none()));
            assert_eq!(items[3].tooltip.is_some(), items[3].disabled);
        }
        assert!(AccountsSection::menu_items(&accounts.accounts[0], &accounts)[3].disabled);
        assert!(AccountsSection::menu_items(&accounts.accounts[1], &accounts)[3].disabled);
        assert!(!AccountsSection::menu_items(&accounts.accounts[2], &accounts)[3].disabled);
        accounts.main_id = "other".into();
        assert!(AccountsSection::menu_items(&accounts.accounts[0], &accounts)[3].disabled);
    }

    #[test]
    fn active_badge_is_independent_of_main_badge() {
        let mut cx = gpui::TestAppContext::single();
        let temp = tempfile::tempdir().unwrap();
        let store = crate::app_store::test_lifecycle_store(&mut cx, temp.path());
        store.update(&mut cx, |store, _| store.claude_accounts = accounts());
        let pane = cx.add_window(|window, cx| {
            window.resize(gpui::size(px(1000.), px(700.)));
            AccountsSection::new(store.clone(), cx)
        });
        let mut visual = VisualTestContext::from_window(pane.into(), &cx);
        visual.run_until_parked();
        assert!(visual.debug_bounds("CLAUDE_ACCOUNT_MAIN_native").is_some());
        assert!(visual.debug_bounds("CLAUDE_ACCOUNT_ACTIVE_work").is_some());
        assert!(visual
            .debug_bounds("CLAUDE_ACCOUNT_ACTIVE_native")
            .is_none());
        store.update(&mut visual, |store, cx| {
            store.claude_accounts.active_id = "native".into();
            cx.notify();
        });
        visual.run_until_parked();
        let main = visual.debug_bounds("CLAUDE_ACCOUNT_MAIN_native").unwrap();
        let active = visual.debug_bounds("CLAUDE_ACCOUNT_ACTIVE_native").unwrap();
        assert!(active.left() >= main.right());
        assert!(visual.debug_bounds("CLAUDE_ACCOUNT_ACTIVE_work").is_none());
        store.update(&mut visual, |store, cx| {
            store.claude_accounts.main_id = "other".into();
            cx.notify();
        });
        visual.run_until_parked();
        assert!(visual.debug_bounds("CLAUDE_ACCOUNT_MAIN_other").is_some());
        assert!(visual
            .debug_bounds("CLAUDE_ACCOUNT_ACTIVE_native")
            .is_some());
        assert!(visual.debug_bounds("CLAUDE_ACCOUNT_MAIN_native").is_none());
    }

    #[test]
    fn account_rows_and_add_form_render_without_running_authentication() {
        let mut cx = gpui::TestAppContext::single();
        let temp = tempfile::tempdir().unwrap();
        let store = crate::app_store::test_lifecycle_store(&mut cx, temp.path());
        store.update(&mut cx, |store, _| store.claude_accounts = accounts());
        let pane = cx.add_window(|window, cx| {
            window.resize(gpui::size(px(1000.), px(700.)));
            AccountsSection::new(store, cx)
        });
        cx.run_until_parked();
        let mut visual = VisualTestContext::from_window(pane.into(), &cx);
        assert!(visual.debug_bounds("CLAUDE_ACCOUNTS").is_some());
        for id in [
            "CLAUDE_ACCOUNT_native",
            "CLAUDE_ACCOUNT_work",
            "CLAUDE_ACCOUNT_other",
        ] {
            assert!(visual.debug_bounds(id).is_some());
        }
        pane.update(&mut visual, |pane, _, cx| {
            pane.adding = true;
            pane.name
                .update(cx, |field, cx| field.set_text("Work Team", cx));
            cx.notify();
        })
        .unwrap();
        visual.run_until_parked();
        pane.read_with(&visual, |pane, cx| {
            assert_eq!(pane.name.read(cx).text(), "Work Team");
            assert_eq!(pane.folder.read(cx).text(), "");
            assert!(!pane.saving);
            assert!(pane.use_main_settings);
        })
        .unwrap();
        let checkbox = visual
            .debug_bounds("CLAUDE_ACCOUNT_SHARE_CHECKBOX")
            .unwrap();
        visual.simulate_click(checkbox.center(), gpui::Modifiers::default());
        visual.run_until_parked();
        pane.read_with(&visual, |pane, _| assert!(!pane.use_main_settings))
            .unwrap();
        pane.update(&mut visual, |pane, _, cx| {
            pane.adding = false;
            pane.store.update(cx, |store, cx| {
                store.claude_accounts.pending_main_id = Some("other".into());
                cx.notify();
            });
            cx.notify();
        })
        .unwrap();
        visual.run_until_parked();
        assert!(visual.debug_bounds("CLAUDE_MAIN_CHANGE_PENDING").is_some());
        pane.read_with(&visual, |pane, cx| {
            let accounts = &pane.store.read(cx).claude_accounts;
            assert!(AccountsSection::menu_items(&accounts.accounts[1], accounts)[0].disabled);
            assert!(!AccountsSection::menu_items(&accounts.accounts[2], accounts)[0].disabled);
            assert!(
                AccountsSection::menu_items(&accounts.accounts[2], accounts)
                    .last()
                    .unwrap()
                    .disabled
            );
        })
        .unwrap();
    }

    #[test]
    fn sharing_menu_is_present_only_for_linked_accounts() {
        let mut accounts = accounts();
        let before = AccountsSection::menu_items(&accounts.accounts[1], &accounts);
        accounts.sharing.insert(
            "work".into(),
            runner_core::protocol::ClaudeAccountSharing {
                using_main: true,
                own_items: vec!["settings.json".into()],
                ..Default::default()
            },
        );
        let after = AccountsSection::menu_items(&accounts.accounts[1], &accounts);
        assert_eq!(after.len(), before.len() + 1);
        assert_eq!(
            after[1].label,
            crate::i18n::tf("accounts.stop_sharing", &[("main", "Personal".into())])
        );
        assert!(after.last().unwrap().disabled);
    }

    struct AccountsMenuHost(Entity<AccountsSection>);
    impl Render for AccountsMenuHost {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w(px(1000.)).h(px(700.)).child(self.0.clone())
        }
    }

    #[test]
    fn cached_menu_updates_actions_after_sharing_changes() {
        let mut cx = gpui::TestAppContext::single();
        let temp = tempfile::tempdir().unwrap();
        let store = crate::app_store::test_lifecycle_store(&mut cx, temp.path());
        let client = store.read_with(&cx, |store, _| store.client.clone());
        let snapshot = client
            .claude_account_action(ClaudeAccountAction::Add {
                name: "Menu fixture".into(),
                folder: Some(temp.path().join("account").to_string_lossy().into_owned()),
                use_main_settings: false,
            })
            .unwrap();
        let id = snapshot.accounts.last().unwrap().id.clone();
        let folder = PathBuf::from(&snapshot.accounts.last().unwrap().folder);
        store.update(&mut cx, |store, _| store.claude_accounts = snapshot);
        let revealed = Rc::new(std::cell::RefCell::new(Vec::<PathBuf>::new()));
        let revealed_paths = revealed.clone();
        let host = cx.add_window(|window, cx| {
            window.resize(gpui::size(px(1000.), px(700.)));
            AccountsMenuHost(cx.new(|cx| {
                let mut pane = AccountsSection::new(store.clone(), cx);
                pane.reveal_folder =
                    Rc::new(move |path, _| revealed_paths.borrow_mut().push(path.to_path_buf()));
                pane
            }))
        });
        let pane = host.read_with(&cx, |host, _| host.0.clone()).unwrap();
        cx.run_until_parked();
        let mut visual = VisualTestContext::from_window(host.into(), &cx);
        let menu = pane.read_with(&visual, |pane, _| pane.menus[&id].clone());
        store.update(&mut visual, |store, cx| {
            store.claude_accounts.sharing.insert(
                id.clone(),
                runner_core::protocol::ClaudeAccountSharing {
                    using_main: true,
                    own_items: vec![],
                    ..Default::default()
                },
            );
            cx.notify();
        });
        visual.run_until_parked();
        assert_eq!(
            menu.entity_id(),
            pane.read_with(&visual, |pane, _| pane.menus[&id].entity_id())
        );
        let focus = menu.read_with(&visual, |menu, cx| menu.focus_handle(cx));
        let open = |visual: &mut VisualTestContext| {
            visual.update(|window, cx| focus.focus(window, cx));
            visual.simulate_keystrokes("down");
            visual.run_until_parked();
        };
        let select = |visual: &mut VisualTestContext, index| {
            open(visual);
            for _ in 0..index {
                visual.simulate_keystrokes("down");
            }
            visual.simulate_keystrokes("enter");
            visual.run_until_parked();
        };
        select(&mut visual, 3);
        assert_eq!(*revealed.borrow(), vec![folder]);
        pane.read_with(&visual, |pane, cx| {
            assert!(pane
                .store
                .read(cx)
                .claude_accounts
                .accounts
                .iter()
                .any(|account| account.id == id));
            assert!(!pane.saving);
            assert!(pane.renaming.is_none());
        });
        select(&mut visual, 1);
        pane.read_with(&visual, |pane, cx| {
            assert!(pane.renaming.is_none());
            assert!(!pane.saving);
            assert!(!pane
                .store
                .read(cx)
                .claude_accounts
                .sharing
                .contains_key(&id));
            assert!(pane
                .store
                .read(cx)
                .claude_accounts
                .accounts
                .iter()
                .any(|account| account.id == id));
        });
        assert_eq!(
            menu.entity_id(),
            pane.read_with(&visual, |pane, _| pane.menus[&id].entity_id())
        );
        select(&mut visual, 1);
        pane.read_with(&visual, |pane, _| {
            assert_eq!(pane.renaming.as_deref(), Some(id.as_str()))
        });
    }
}
