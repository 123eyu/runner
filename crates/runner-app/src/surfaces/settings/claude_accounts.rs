use crate::{app_store::AppStore, theme};
use gpui::{
    div, prelude::*, rems, AnyElement, Context, Entity, FontWeight, PathPromptOptions, Render,
    SharedString, Subscription, Window,
};
use runner_app::ui::menu::{MenuItem, PopoverMenu};
use runner_app::ui::{Badge, BrowseField, Button, ButtonSize, ButtonVariant, TextField, Tone};
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
        _ => return error.to_owned(),
    };
    crate::i18n::text(key).to_owned()
}

pub(crate) struct AccountsSection {
    store: Entity<AppStore>,
    name: Entity<TextField>,
    folder: Entity<TextField>,
    adding: bool,
    renaming: Option<String>,
    saving: bool,
    error: Option<String>,
    menus: HashMap<String, Entity<PopoverMenu>>,
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
        Self {
            store,
            name,
            folder,
            adding: false,
            renaming: None,
            saving: false,
            error: None,
            menus: HashMap::new(),
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
        let protected =
            account.native || account.id == accounts.main_id || account.id == accounts.active_id;
        vec![
            MenuItem::new(crate::i18n::message("accounts.make_main"))
                .icon("star.svg")
                .disabled(account.id == accounts.main_id),
            MenuItem::new(crate::i18n::message("ui.rename")).icon("pencil.svg"),
            MenuItem::new(crate::i18n::message(if cfg!(windows) {
                "ui.reveal_in_explorer"
            } else {
                "ui.reveal_in_finder"
            }))
            .icon("folder.svg"),
            MenuItem::new(crate::i18n::message("ui.remove"))
                .icon("trash-2.svg")
                .separator_before(true)
                .destructive(true)
                .disabled(protected)
                .description(if protected {
                    crate::i18n::message("accounts.cannot_remove")
                } else {
                    crate::i18n::message("accounts.keep_folder")
                }),
        ]
    }

    fn account_menu(
        &mut self,
        account: &ClaudeAccount,
        accounts: &ClaudeAccounts,
        cx: &mut Context<Self>,
    ) -> Entity<PopoverMenu> {
        let items = Self::menu_items(account, accounts);
        if let Some(menu) = self.menus.get(&account.id) {
            menu.update(cx, |menu, cx| menu.set_items(items, cx));
            return menu.clone();
        }
        let owner = cx.weak_entity();
        let account = account.clone();
        let id = account.id.clone();
        let menu = cx.new(|cx| {
            PopoverMenu::new(
                SharedString::from(format!("account-menu-{id}")),
                cx.focus_handle(),
                items,
                Rc::new(move |index, window, cx| {
                    let _ = owner.update(cx, |this, cx| match index {
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
                        2 => cx.reveal_path(&PathBuf::from(&account.folder)),
                        3 => this.perform(
                            ClaudeAccountAction::Remove {
                                id: account.id.clone(),
                            },
                            cx,
                        ),
                        _ => {}
                    });
                }),
                cx,
            )
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
                            .child(account.folder.clone()),
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
        let rows = accounts
            .accounts
            .iter()
            .map(|account| self.row(account, &accounts, cx))
            .collect::<Vec<_>>();
        div()
            .debug_selector(|| "CLAUDE_ACCOUNTS".into())
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
                            .disabled(self.adding || self.saving)
                            .on_press(move |window, cx| {
                                owner.update(cx, |this, cx| {
                                    this.adding = true;
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

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{px, VisualTestContext};

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
            statuses: Default::default(),
        }
    }

    #[test]
    fn account_menu_protects_main_active_and_native_rows() {
        let mut accounts = accounts();
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
        })
        .unwrap();
    }
}
