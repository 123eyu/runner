use super::*;

impl NativeRoot {
    pub(crate) fn render_start_chat_modal(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let modal = self.start_chat_modal.as_ref().expect("modal is open");
        if modal.mission_role.is_some() {
            return self.render_add_role_modal(modal, window, cx);
        }
        let mode = modal.mode;
        let submitting = modal.submitting;
        let can_submit = modal.can_submit();
        let form_width = OverlayWidth::Custom(MODAL_WIDTH).body_width(window);
        let layout = CardLayout::new(form_width);

        let role_fields = self.render_role_fields(modal, layout, cx);
        let direct_fields = self.render_direct_fields(modal, layout, cx);

        let root = cx.entity();
        let browse_root = root.clone();
        let content = div()
            .debug_selector(|| "START_CHAT_FORM".into())
            .flex()
            .flex_col()
            .when(cfg!(any(windows, target_os = "linux")), |content| {
                content.w(rems(form_width / 16.))
            })
            .gap_5()
            .on_key_down(cx.listener(Self::on_start_chat_key_down))
            .children(
                modal
                    .error
                    .as_ref()
                    .filter(|_| !self.app_store.read(cx).daemon_disconnected)
                    .map(|error| {
                        runner_app::ui::notice_banner(error.resolve(), runner_app::ui::Tone::Danger)
                    }),
            )
            .child(
                div()
                    .debug_selector(|| "START_CHAT_MODES".into())
                    .flex()
                    .w_full()
                    .p(rems(2. / 16.))
                    .rounded(rems(6. / 16.))
                    .border_1()
                    .border_color(theme::border())
                    .bg(theme::bg())
                    .child(self.render_mode_button(
                        crate::i18n::text("ui.direct"),
                        ChatMode::Runtime,
                        mode,
                        submitting,
                        modal.direct_mode_focus.clone(),
                        cx,
                    ))
                    .child(self.render_mode_button(
                        crate::i18n::text("ui.role"),
                        ChatMode::Role,
                        mode,
                        submitting,
                        modal.role_mode_focus.clone(),
                        cx,
                    )),
            )
            .children(
                modal
                    .host_account
                    .as_ref()
                    .filter(|_| {
                        modal
                            .active_runtime()
                            .is_some_and(|runtime| runtime.name.key() == "claude-code")
                    })
                    .map(|name| {
                        div()
                            .text_size(theme::text_meta())
                            .text_color(theme::muted())
                            .child(crate::i18n::tf(
                                "client.host_account",
                                &[("account", name.clone())],
                            ))
                    }),
            )
            .child(match mode {
                ChatMode::Role => role_fields,
                ChatMode::Runtime => direct_fields,
            })
            .children(modal.front_page.supports(&["rename"]).then(|| {
                Field::new(
                    "start-chat-title-field",
                    crate::i18n::text("ui.chat_name"),
                    modal.title.clone(),
                )
                .focus_target(modal.title.read(cx).focus_handle())
                .emphasized(true)
                .tag(crate::i18n::text("word.optional"))
            }))
            .child(
                Field::new(
                    "start-chat-project-field",
                    crate::i18n::text("ui.project"),
                    modal.project.select(),
                )
                .focus_target(modal.project.focus_handle(cx))
                .emphasized(true)
                .map(|field| {
                    if modal.project.locked() {
                        field.subtitle(crate::i18n::text("ui.chats_in_a_pane_file_with_their_tab"))
                    } else {
                        field
                    }
                }),
            )
            .child(
                Field::new(
                    "start-chat-working-dir-field",
                    crate::i18n::text("ui.working_directory"),
                    div()
                        .relative()
                        .child(
                            WorkingDirField::new(
                                modal.cwd.clone(),
                                submitting,
                                Rc::new(move |window, cx| {
                                    browse_root.update(cx, |this, cx| {
                                        if let Some(picker) = this
                                            .start_chat_modal
                                            .as_ref()
                                            .and_then(|modal| modal.folder_picker.clone())
                                        {
                                            picker
                                                .update(cx, |picker, cx| picker.toggle(window, cx));
                                        } else {
                                            this.browse_start_chat_cwd(cx);
                                        }
                                    });
                                }),
                            )
                            .browse_focus(modal.browse_focus.clone())
                            .show_browse(
                                !modal.front_page.is_remote()
                                    || modal.front_page.supports(&["folders"]),
                            ),
                        )
                        .children(
                            modal.folder_picker.clone().map(
                                crate::surfaces::front_page::folder_picker::FolderPicker::anchor,
                            ),
                        ),
                )
                .focus_target(modal.cwd.read(cx).focus_handle())
                .emphasized(true)
                .map(|field| {
                    if let Some(error) = modal
                        .folder_picker
                        .as_ref()
                        .and_then(|picker| picker.read(cx).path_error.clone())
                    {
                        field.error(error)
                    } else if let Some(hint) =
                        modal.project.directory_hint(modal.cwd.read(cx).text())
                    {
                        hint.apply(field)
                    } else {
                        field.subtitle(working_dir_hint(mode, modal.selected_role()))
                    }
                }),
            );

        let close_root = root.clone();
        let cancel_root = root.clone();
        let submit_root = root;
        let title = div()
            .flex()
            .items_center()
            .justify_between()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(rems(2. / 16.))
                    .child(
                        div()
                            .text_size(theme::text_heading())
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme::text())
                            .child(crate::i18n::text("ui.start_a_chat")),
                    )
                    .child(
                        div()
                            .text_size(theme::text_ui())
                            .font_weight(FontWeight::NORMAL)
                            .text_color(theme::muted())
                            .child(crate::i18n::text(
                                "ui.a_one_on_one_terminal_with_an_agent_no_mission_required",
                            )),
                    ),
            )
            .child(
                IconButton::new("close-start-chat", "close.svg")
                    .focus_handle(modal.close_focus.clone())
                    .tooltip(crate::i18n::text("ui.close_start_chat"))
                    .disabled(submitting)
                    .on_press(move |window, cx| {
                        close_root.update(cx, |this, cx| this.close_start_chat_modal(window, cx));
                    }),
            );
        let footer = div()
            .flex()
            .items_center()
            .gap_2()
            .child(
                Button::new("cancel-start-chat", crate::i18n::text("ui.cancel"))
                    .shortcut("esc")
                    .focus_handle(modal.cancel_focus.clone())
                    .disabled(submitting)
                    .on_press(move |window, cx| {
                        cancel_root.update(cx, |this, cx| this.close_start_chat_modal(window, cx));
                    }),
            )
            .child(
                Button::new(
                    "submit-start-chat",
                    if submitting {
                        crate::i18n::text("ui.starting")
                    } else {
                        crate::i18n::text("ui.start_chat")
                    },
                )
                .shortcut(keymap::fixed_shortcut("cmd-enter"))
                .focus_handle(modal.submit_focus.clone())
                .variant(ButtonVariant::Primary)
                .disabled(!can_submit)
                .on_press(move |window, cx| {
                    submit_root.update(cx, |this, cx| this.submit_start_chat(window, cx));
                }),
            );
        let modal_close_root = cx.entity();
        let modal_element = Modal::new(
            title,
            content,
            Rc::new(move |window, cx| {
                modal_close_root.update(cx, |this, cx| this.close_start_chat_modal(window, cx));
            }),
        )
        .key_context("StartChat")
        .width(OverlayWidth::Custom(MODAL_WIDTH))
        .busy(submitting)
        .focus_order(if submitting {
            Vec::new()
        } else {
            start_chat_focus_order(modal, cx)
        })
        .scrollbar(modal.scroll_handle.clone(), modal.scrollbar.clone())
        .footer(footer);
        div()
            .absolute()
            .inset_0()
            .on_action(cx.listener(Self::confirm_start_chat))
            .on_action(cx.listener(|this, _: &StartChatDirect, window, cx| {
                this.switch_start_chat_mode(ChatMode::Runtime, window, cx);
            }))
            .on_action(cx.listener(|this, _: &StartChatRole, window, cx| {
                this.switch_start_chat_mode(ChatMode::Role, window, cx);
            }))
            .child(modal_element)
            .children(modal.folder_picker.clone())
            .into_any_element()
    }

    /// Puts one role control back to the role's value and hands focus to it,
    /// since its Reset goes with the override.
    pub(super) fn reset_start_chat_control(
        &mut self,
        kind: ResetKind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(modal) = self.start_chat_modal.as_mut() else {
            return;
        };
        if modal.submitting {
            return;
        }
        let focus = match kind {
            ResetKind::Runtime => {
                modal.role_runtime_override = None;
                sync_role_runtime_select(modal, cx);
                restore_baseline(modal, cx);
                modal.role_runtime_select.read(cx).focus_handle()
            }
            ResetKind::Model => {
                modal
                    .model
                    .update(cx, |input, input_cx| input.reset("", input_cx));
                modal.model.read(cx).focus_handle()
            }
            ResetKind::Effort => {
                modal.effort = modal.baseline_effort();
                sync_effort_control(modal, cx);
                modal.effort_select.read(cx).focus_handle()
            }
            ResetKind::Speed => {
                modal.speed = modal.baseline_speed();
                sync_speed_control(modal, cx);
                modal.speed_select.read(cx).focus_handle()
            }
        };
        focus.focus(window, cx);
        if kind == ResetKind::Runtime {
            if let Some(runtime) = self
                .start_chat_modal
                .as_ref()
                .and_then(|modal| modal.active_runtime())
            {
                if self.remote_client.is_local() {
                    self.request_model_catalog(runtime.name.key(), cx);
                }
            }
        }
        cx.notify();
    }

    pub(super) fn render_direct_fields(
        &self,
        modal: &StartChatModal,
        layout: CardLayout,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let settings_root = cx.entity();
        let mut card = card(layout).child(modal.runtime_select.clone());
        if let Some(runtime) = modal.active_runtime() {
            let has_effort = !runtime.efforts.is_empty();
            let has_speed = modal.codex_speed_visible();
            card = card.child(
                card_section()
                    .when(cfg!(test), |section| {
                        section.debug_selector(|| "START_CHAT_SETUP".into())
                    })
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .gap(rems(COLUMN_GAP / 16.))
                            .child(setup_field(
                                layout.model(has_effort, has_speed),
                                crate::i18n::text("ui.model"),
                                div().w_full().child(modal.model_field.clone()),
                                None,
                            ))
                            .when(has_effort, |row| {
                                row.child(setup_field(
                                    EFFORT_COLUMN_WIDTH,
                                    crate::i18n::text("ui.effort"),
                                    modal.effort_select.clone(),
                                    None,
                                ))
                            })
                            .when(has_speed, |row| {
                                row.child(render_speed_column(modal, SPEED_COLUMN_WIDTH, None))
                            }),
                    )
                    .children(speed_note(modal)),
            );
        }
        Field::new(
            "start-chat-direct-agent-field",
            crate::i18n::text("ui.agent"),
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(card)
                .when(modal.runtimes.is_empty(), |field| {
                    field.child(
                        div()
                            .flex()
                            .items_center()
                            .text_size(theme::text_meta())
                            .text_color(theme::warning())
                            .when(modal.agents_checking, |message| {
                                message.child(crate::i18n::text("ui.detecting_agents"))
                            })
                            .when(
                                !modal.agents_checking && !modal.front_page.is_remote(),
                                |message| {
                                    let root = settings_root.clone();
                                    message
                                        .child(crate::i18n::text(
                                            "ui.no_enabled_agents_launch_hint",
                                        ))
                                        .child(
                                            div()
                                                .id("start-chat-open-agent-settings")
                                                .cursor_pointer()
                                                .text_color(theme::warning())
                                                .hover(|link| link.text_color(theme::text()))
                                                .child(crate::i18n::text(
                                                    "ui.configure_one_in_settings_agents",
                                                ))
                                                .on_click(move |_, window, cx| {
                                                    root.update(cx, |this, cx| {
                                                        this.start_chat_modal = None;
                                                        this.enter_settings_route(
                                                            Some("agents"),
                                                            window,
                                                            cx,
                                                        );
                                                    });
                                                }),
                                        )
                                },
                            ),
                    )
                })
                .children(
                    modal
                        .agents_error
                        .clone()
                        .filter(|_| !self.app_store.read(cx).daemon_disconnected)
                        .map(|error| {
                            div()
                                .text_size(theme::text_meta())
                                .text_color(theme::danger())
                                .child(error)
                        }),
                ),
        )
        .focus_target(modal.runtime_select.read(cx).focus_handle())
        .emphasized(true)
        .into_any_element()
    }

    pub(super) fn render_mode_button(
        &self,
        label: &'static str,
        mode: ChatMode,
        active: ChatMode,
        disabled: bool,
        focus_handle: FocusHandle,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let root = cx.entity();
        let key_root = root.clone();
        let mut button = div()
            .debug_selector(move || match mode {
                ChatMode::Role => "START_CHAT_ROLE_MODE".into(),
                ChatMode::Runtime => "START_CHAT_DIRECT_MODE".into(),
            })
            .id(match mode {
                ChatMode::Role => "start-chat-mode-role",
                ChatMode::Runtime => "start-chat-mode-runtime",
            })
            .track_focus(&focus_handle.tab_stop(!disabled && active == mode))
            .tab_index(0)
            .tab_stop(!disabled && active == mode)
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .h(rems(30. / 16.))
            .rounded_md()
            .text_size(theme::text_ui())
            .font_weight(FontWeight::SEMIBOLD)
            .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
            .text_color(if active == mode {
                theme::text()
            } else {
                theme::muted()
            })
            .when(active == mode, |button| button.bg(theme::border()))
            .opacity(if disabled { 0.6 } else { 1. })
            .focus_visible(|button| button.text_color(theme::text()));
        if !disabled {
            button = button
                .cursor_pointer()
                .hover(|button| button.text_color(theme::text()))
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.mouse_switch_start_chat_mode(mode, window, cx);
                }))
                .on_key_down(move |event: &KeyDownEvent, window, cx| {
                    if matches!(event.keystroke.key.as_str(), "left" | "right") {
                        cx.stop_propagation();
                        let next = if mode == ChatMode::Role {
                            ChatMode::Runtime
                        } else {
                            ChatMode::Role
                        };
                        key_root.update(cx, |this, cx| {
                            this.set_start_chat_mode(next, cx);
                            if let Some(modal) = &this.start_chat_modal {
                                modal.mode_focus().focus(window, cx);
                            }
                        });
                    } else if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                        cx.stop_propagation();
                        key_root.update(cx, |this, cx| this.set_start_chat_mode(mode, cx));
                    }
                });
        }
        button
            .gap_2()
            .child(label)
            .child(
                div()
                    .text_size(theme::text_meta())
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme::faint())
                    .child(keymap::fixed_shortcut(match mode {
                        ChatMode::Runtime => "cmd-1",
                        ChatMode::Role => "cmd-2",
                    })),
            )
            .into_any_element()
    }
}

/// Column widths inside a card, from the width the modal leaves its form.
#[derive(Clone, Copy)]
pub(super) struct CardLayout {
    /// The card's own width, border included.
    pub(super) outer: f32,
    /// The width inside the border and the card's padding.
    pub(super) content: f32,
}

impl CardLayout {
    pub(super) fn new(form_width: f32) -> Self {
        Self {
            outer: form_width.floor(),
            content: form_width.floor() - 2. - 2. * CARD_PADDING,
        }
    }

    /// Runtime, Model and Effort of a role chat.
    pub(super) fn third(self) -> f32 {
        ((self.content - 2. * COLUMN_GAP) / 3.).floor()
    }

    /// Direct's Model fills what its Effort and Speed leave.
    pub(super) fn model(self, effort: bool, speed: bool) -> f32 {
        let taken = if effort {
            EFFORT_COLUMN_WIDTH + COLUMN_GAP
        } else {
            0.
        } + if speed {
            SPEED_COLUMN_WIDTH + COLUMN_GAP
        } else {
            0.
        };
        self.content - taken
    }
}

/// A role or an agent: who the chat is with on top, how it runs underneath.
pub(super) fn card(layout: CardLayout) -> Div {
    div()
        .when(cfg!(test), |card| {
            card.debug_selector(|| "START_CHAT_CARD".into())
        })
        .w(rems(layout.outer / 16.))
        .flex()
        .flex_col()
        .rounded(rems(8. / 16.))
        .border_1()
        .border_color(theme::border())
        .bg(theme::bg())
}

pub(super) fn card_section() -> Div {
    div()
        .flex()
        .flex_col()
        .gap(rems(COLUMN_GAP / 16.))
        .px(rems(CARD_PADDING / 16.))
        .py(rems(CARD_PADDING / 16.))
        .border_t_1()
        .border_color(theme::border())
}

/// A labelled column of a card, with a note under the control while it
/// overrides the role's value.
pub(super) fn setup_field(
    width: f32,
    label: &'static str,
    control: impl IntoElement,
    note: Option<AnyElement>,
) -> Div {
    div()
        .when(cfg!(test), |field| {
            field.debug_selector(move || format!("START_CHAT_FIELD {label}"))
        })
        .w(rems(width / 16.))
        .flex_none()
        .flex()
        .flex_col()
        .gap(rems(6. / 16.))
        .child(section_label(label))
        .child(control)
        .children(note)
}

/// What Reset takes of a column's width, and the gap before it.
pub(super) const RESET_WIDTH: f32 = 50.;

pub(super) const RESET_GAP: f32 = 8.;

/// The role's value under an overridden control, with its Reset.
pub(super) fn override_note(
    label: &'static str,
    width: f32,
    text: String,
    reset: gpui::Stateful<Div>,
) -> AnyElement {
    let selector = format!("START_CHAT_NOTE {label} {text}");
    div()
        .flex()
        .items_center()
        .gap(rems(RESET_GAP / 16.))
        .child(
            column_text(text, width - RESET_WIDTH - RESET_GAP)
                .when(cfg!(test), |note| {
                    note.debug_selector(move || selector.clone())
                })
                .text_size(theme::text_meta())
                .text_color(theme::faint()),
        )
        .child(reset)
        .into_any_element()
}

/// A control's Reset: restores that one value to the role's.
pub(super) fn reset_action(
    root: &Entity<NativeRoot>,
    focus: &FocusHandle,
    kind: ResetKind,
) -> gpui::Stateful<Div> {
    let root = root.clone();
    text_action(
        match kind {
            ResetKind::Runtime => "start-chat-reset-runtime",
            ResetKind::Model => "start-chat-reset-model",
            ResetKind::Effort => "start-chat-reset-effort",
            ResetKind::Speed => "start-chat-reset-speed",
        },
        focus,
        move |window, cx| {
            root.update(cx, |this, cx| {
                this.reset_start_chat_control(kind, window, cx)
            });
        },
    )
    .when(cfg!(test), move |reset| {
        reset.debug_selector(move || format!("START_CHAT_RESET {kind:?}"))
    })
    .flex_none()
    .gap(rems(6. / 16.))
    .text_size(theme::text_ui())
    .text_color(theme::muted())
    .hover(|reset| reset.text_color(theme::text()))
    .child(
        svg()
            .flex_none()
            .path("rotate-ccw.svg")
            .size(rems(11. / 16.))
            .text_color(theme::muted()),
    )
    .child(crate::i18n::text("ui.reset"))
}

pub(super) fn render_speed_column(
    modal: &StartChatModal,
    width: f32,
    note: Option<AnyElement>,
) -> Div {
    setup_field(
        width,
        crate::i18n::text("ui.speed"),
        div()
            .when(cfg!(test), |field| {
                field.debug_selector(|| "START_CHAT_SPEED_FIELD".into())
            })
            .child(modal.speed_select.clone()),
        note,
    )
}

pub(super) fn speed_note(modal: &StartChatModal) -> Option<Div> {
    (modal.effective_speed() == Some(CodexSpeed::Fast)).then(|| {
        div()
            .when(cfg!(test), |note| {
                note.debug_selector(|| "START_CHAT_SPEED_NOTE".into())
            })
            .text_size(theme::text_meta())
            .text_color(theme::faint())
            .child(crate::i18n::text("ui.fast_uses_more_credits"))
    })
}

/// Why a role's model and effort are not on show once another agent is
/// picked.
pub(super) fn agent_note(
    agent: &str,
    role_agent: &str,
    role_model: Option<&str>,
    role_effort: Option<&str>,
) -> String {
    let role_effort =
        role_effort.map(|effort| runner_app::ui::select::effort_label(effort, effort).resolve());
    let role_effort = role_effort.as_deref();
    let start = crate::i18n::tf(
        "format.starts_from_its_own_model_and_effort",
        &[("agent", (agent).to_string())],
    );
    match (role_model, role_effort) {
        (Some(model), Some(effort)) => crate::i18n::tf(
            "format.the_role_s_and_belong_to_and_don_t_carry_over",
            &[
                ("start", (start).to_string()),
                ("model", (model).to_string()),
                ("effort", (effort).to_string()),
                ("role_agent", (role_agent).to_string()),
            ],
        ),
        (Some(value), None) | (None, Some(value)) => crate::i18n::tf(
            "format.the_role_s_belongs_to_and_doesn_t_carry_over",
            &[
                ("start", (start).to_string()),
                ("inherited_setting", (value).to_string()),
                ("role_agent", (role_agent).to_string()),
            ],
        ),
        (None, None) => start,
    }
}

/// Where a blank Working directory starts.
pub(super) fn working_dir_hint(mode: ChatMode, role: Option<&RoleChoice>) -> &'static str {
    match mode {
        ChatMode::Role
            if role.is_some_and(|role| trimmed(role.working_dir.as_deref()).is_some()) =>
        {
            crate::i18n::text("ui.blank_starts_in_the_role_s_directory")
        }
        _ => crate::i18n::text("ui.blank_starts_in_your_default_directory"),
    }
}
