use super::*;

impl NativeRoot {
    pub(super) fn render_role_fields(
        &self,
        modal: &StartChatModal,
        layout: CardLayout,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut card = card(layout).child(modal.role_select.clone());
        let role = modal.selected_role();
        if let Some(role) = role {
            card = card.child(self.render_role_setup(modal, role, layout, cx));
        }
        let mut field = Field::new(
            "start-chat-role-field",
            "Role",
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(card)
                .when(modal.roles.is_empty(), |field| {
                    field.child(
                        div()
                            .text_size(theme::text_meta())
                            .text_color(theme::warning())
                            .child("No roles yet. Create one from the Roles page first."),
                    )
                }),
        )
        .focus_target(modal.role_select.read(cx).focus_handle())
        .emphasized(true);
        if role.is_some() {
            field = field.subtitle(if modal.mission_role.is_some() {
                "Starts with the role's settings. Change any of them for this agent only."
            } else {
                ROLE_HINT
            });
        }
        field.into_any_element()
    }

    /// The role's Runtime, Model and Effort as controls filled with the role's
    /// values; an override adds the amber dot and, under the control, the
    /// role's value with Reset.
    pub(super) fn render_role_setup(
        &self,
        modal: &StartChatModal,
        role: &Role,
        layout: CardLayout,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let third = layout.third();
        let root = cx.entity();
        let overrides = modal.role_overrides(cx);
        let own = modal.on_own_agent();
        let agent = modal.role_agent().unwrap_or(&role.runtime).to_owned();
        let runtime = modal.active_runtime();
        let effort_available = runtime.is_some_and(|runtime| !runtime.efforts.is_empty());
        let note = |label: &'static str, kind: ResetKind, changed: bool, value: String| {
            changed.then(|| {
                override_note(
                    label,
                    third,
                    format!("role: {value}"),
                    reset_action(&root, &modal.reset_focus[kind as usize], kind),
                )
            })
        };
        let row = || div().flex().items_start().gap(rems(COLUMN_GAP / 16.));
        let controls = row()
            .child(setup_field(
                third,
                "Runtime",
                modal.role_runtime_select.clone(),
                note(
                    "Runtime",
                    ResetKind::Runtime,
                    overrides.runtime,
                    runtime_display_name(&modal.runtimes, &role.runtime),
                ),
            ))
            .child(setup_field(
                third,
                "Model",
                div().w_full().child(modal.model_field.clone()),
                note(
                    "Model",
                    ResetKind::Model,
                    overrides.model,
                    model_placeholder(Some(role)),
                ),
            ))
            .when(effort_available, |controls| {
                controls.child(setup_field(
                    third,
                    "Effort",
                    modal.effort_select.clone(),
                    note(
                        "Effort",
                        ResetKind::Effort,
                        overrides.effort,
                        role_setting_label(Some(&modal.baseline_effort())).0,
                    ),
                ))
            });
        card_section()
            .when(cfg!(test), |section| {
                section.debug_selector(|| "START_CHAT_SETUP".into())
            })
            .child(controls)
            .when(modal.codex_speed_visible(), |section| {
                section.child(row().child(render_speed_column(
                    modal,
                    third,
                    note(
                        "Speed",
                        ResetKind::Speed,
                        overrides.speed,
                        speed_label(role.codex_speed).into(),
                    ),
                )))
            })
            .children(speed_note(modal))
            .children((!own).then(|| {
                let note = agent_note(
                    &runtime_display_name(&modal.runtimes, &agent),
                    &runtime_display_name(&modal.runtimes, &role.runtime),
                    trimmed(role.model.as_deref()),
                    trimmed(role.effort.as_deref()),
                );
                div()
                    .w(rems(layout.content / 16.))
                    .text_size(theme::text_ui())
                    .text_color(theme::faint())
                    .child(note)
            }))
            .into_any_element()
    }
}
