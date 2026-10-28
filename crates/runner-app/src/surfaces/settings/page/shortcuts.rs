use super::*;

pub(super) const TAB_SHORTCUT_TITLE: &str = "ui.go_to_numbered_tab";

pub(super) const TAB_SHORTCUT_DESCRIPTION: &str =
    "ui.open_a_visible_sidebar_tab_or_mission_by_its_position";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ShortcutRow {
    Entry(&'static keymap::KeymapEntry),
    TabSelection,
}

impl ShortcutRow {
    pub(super) fn id(self) -> &'static str {
        match self {
            Self::Entry(entry) => entry.id,
            Self::TabSelection => TAB_SHORTCUT_ID,
        }
    }

    pub(super) fn title(self) -> &'static str {
        match self {
            Self::Entry(entry) => entry.title(),
            Self::TabSelection => crate::i18n::text(TAB_SHORTCUT_TITLE),
        }
    }

    pub(super) fn description(self) -> &'static str {
        match self {
            Self::Entry(entry) => entry.description(),
            Self::TabSelection => crate::i18n::text(TAB_SHORTCUT_DESCRIPTION),
        }
    }

    pub(super) fn fixed(self) -> bool {
        match self {
            Self::Entry(entry) => entry.fixed,
            Self::TabSelection => true,
        }
    }

    pub(super) fn entry_ids(self) -> Vec<&'static str> {
        match self {
            Self::Entry(entry) => vec![entry.id],
            Self::TabSelection => keymap::tab_selection_entries()
                .map(|entry| entry.id)
                .collect(),
        }
    }

    pub(super) fn overridden(self, overrides: &keymap::KeymapOverrides) -> bool {
        self.entry_ids()
            .into_iter()
            .any(|id| keymap::is_overridden(id, overrides))
    }

    pub(super) fn binding_label(self, overrides: &keymap::KeymapOverrides) -> String {
        let label = |id: &str| {
            keymap::effective_binding(id, overrides).map(|combo| keymap::format_combo(&combo))
        };
        let ids = self.entry_ids();
        match (label(ids[0]), label(ids[ids.len() - 1])) {
            (Some(first), Some(last)) if ids.len() > 1 => format!("{first}–{last}"),
            (Some(first), _) => first,
            _ => crate::i18n::text("ui.unassigned").into(),
        }
    }

    pub(super) fn matches(self, query: &str, overrides: &keymap::KeymapOverrides) -> bool {
        let matches = |value: &str| value.to_lowercase().contains(query);
        matches(self.title())
            || matches(self.description())
            || matches(&self.binding_label(overrides))
            || (self == Self::TabSelection
                && keymap::tab_selection_entries().any(|entry| {
                    matches(entry.title())
                        || matches(entry.description())
                        || keymap::effective_binding(entry.id, overrides)
                            .is_some_and(|combo| matches(&keymap::format_combo(&combo)))
                }))
    }
}

pub(super) fn shortcut_rows(query: &str, overrides: &keymap::KeymapOverrides) -> Vec<ShortcutRow> {
    let query = query.trim().to_lowercase();
    let mut rows = Vec::new();
    for entry in keymap::entries() {
        let row = if keymap::is_tab_selection_entry(entry) {
            ShortcutRow::TabSelection
        } else {
            ShortcutRow::Entry(entry)
        };
        if rows.contains(&row) || (!query.is_empty() && !row.matches(&query, overrides)) {
            continue;
        }
        rows.push(row);
    }
    rows
}

impl NativeRoot {
    pub(super) fn render_shortcuts_settings(&self, cx: &mut Context<Self>) -> AnyElement {
        let overrides = self.settings(cx).keymap_overrides.clone();
        let rows = shortcut_rows(&self.settings_page.shortcut_query, &overrides)
            .into_iter()
            .map(|row| self.render_shortcut_row(row, &overrides, cx))
            .collect::<Vec<_>>();
        let has_overrides = keymap::entries()
            .iter()
            .any(|entry| keymap::is_overridden(entry.id, &overrides));
        let reset_root = cx.entity();
        let reset = Button::new(
            "reset-keymap",
            crate::i18n::text("ui.reset_all_to_defaults"),
        )
        .icon("rotate-ccw.svg")
        .size(ButtonSize::Sm)
        .disabled(!has_overrides)
        .on_press(move |window, cx| {
            reset_root.update(cx, |this, root_cx| {
                this.reset_shortcut_overrides(window, root_cx)
            });
        });
        let content = if rows.is_empty() {
            div()
                .text_size(theme::text_ui())
                .text_color(theme::faint())
                .child(crate::i18n::tf(
                    "format.no_shortcuts_match",
                    &[(
                        "query",
                        (self.settings_page.shortcut_query.trim()).to_string(),
                    )],
                ))
                .into_any_element()
        } else {
            SettingsCard::new(rows).into_any_element()
        };

        div()
            .flex()
            .flex_col()
            .gap_5()
            .child(
                PaneHeader::new(
                    crate::i18n::text("ui.keyboard_shortcuts"),
                    crate::i18n::text(crate::platform_ui::SHORTCUT_REQUIREMENTS_KEY),
                )
                .action(reset),
            )
            .child(
                div()
                    .h(rems(36. / 16.))
                    .w(rems(280. / 16.))
                    .flex()
                    .items_center()
                    .gap_2()
                    .rounded(rems(6. / 16.))
                    .border_1()
                    .border_color(theme::border())
                    .bg(theme::panel())
                    .px(rems(10. / 16.))
                    .child(
                        svg()
                            .path("search.svg")
                            .size(rems(14. / 16.))
                            .flex_none()
                            .text_color(theme::faint()),
                    )
                    .child(
                        div()
                            .min_w(px(0.))
                            .flex_1()
                            .child(self.settings_page.shortcut_search.clone()),
                    ),
            )
            .child(content)
            .into_any_element()
    }

    pub(super) fn render_shortcut_row(
        &self,
        row: ShortcutRow,
        overrides: &keymap::KeymapOverrides,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let id = row.id();
        let overridden = row.overridden(overrides);
        let recording = self.settings_page.shortcut_recording == Some(id);
        let conflict = self
            .settings_page
            .shortcut_conflict
            .as_ref()
            .filter(|conflict| conflict.id == id)
            .map(|conflict| conflict.message.resolve().to_string());
        let label = shortcut_row_label(row.title(), row.description(), conflict);

        if recording {
            let key_root = cx.entity();
            let outside_root = key_root.clone();
            return div()
                .min_h(rems(58. / 16.))
                .px_4()
                .py_3()
                .flex()
                .items_center()
                .gap_3()
                .child(label)
                .child(
                    div()
                        .id(SharedString::from(format!("shortcut-recorder-{}", id)))
                        .track_focus(&self.settings_page.shortcut_recording_focus)
                        .h_8()
                        .min_w(px(0.))
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(rems(6. / 16.))
                        .border_1()
                        .border_color(theme::border_strong())
                        .bg(theme::bg())
                        .text_size(theme::text_ui())
                        .text_color(theme::muted())
                        .child(crate::i18n::text("ui.press_keys"))
                        .on_key_down(move |event: &KeyDownEvent, window, cx| {
                            key_root.update(cx, |this, root_cx| {
                                this.record_shortcut_key(id, event, window, root_cx)
                            });
                        })
                        .on_mouse_down_out(move |_, window, cx| {
                            outside_root.update(cx, |this, root_cx| {
                                this.finish_shortcut_recording(window, root_cx)
                            });
                        }),
                )
                .child(div().w_6().flex_none())
                .into_any_element();
        }

        let binding_label = row.binding_label(overrides);
        let restore = overridden.then(|| {
            let restore_root = cx.entity();
            IconButton::new(format!("restore-shortcut-{id}"), "rotate-ccw.svg")
                .size(IconButtonSize::Sm)
                .tooltip(crate::i18n::text("ui.restore_default"))
                .on_press(move |window, cx| {
                    restore_root.update(cx, |this, root_cx| {
                        this.restore_shortcut_default(row, window, root_cx)
                    });
                })
        });
        let edit = if row.fixed() {
            div().w_6().flex_none().into_any_element()
        } else {
            let edit_root = cx.entity();
            IconButton::new(format!("edit-shortcut-{id}"), "pencil.svg")
                .size(IconButtonSize::Sm)
                .tooltip(crate::i18n::text("ui.edit_shortcut"))
                .on_press(move |window, cx| {
                    edit_root.update(cx, |this, root_cx| {
                        this.start_shortcut_recording(id, window, root_cx)
                    });
                })
                .into_any_element()
        };
        let chip = if row.fixed() {
            shortcut_chip(binding_label, false).into_any_element()
        } else {
            let chip_root = cx.entity();
            shortcut_chip(binding_label, true)
                .id(SharedString::from(format!("binding-shortcut-{id}")))
                .on_click(move |_, window, cx| {
                    chip_root.update(cx, |this, root_cx| {
                        this.start_shortcut_recording(id, window, root_cx)
                    });
                })
                .into_any_element()
        };
        let controls = div()
            .min_w(px(0.))
            .flex_1()
            .flex()
            .items_center()
            .justify_between()
            .gap(rems(6. / 16.))
            .child(chip)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(rems(6. / 16.))
                    .children(restore)
                    .child(edit),
            );
        let unbind_root = cx.entity();
        let unbind = IconButton::new(format!("unbind-shortcut-{id}"), "trash.svg")
            .size(IconButtonSize::Sm)
            .tooltip(crate::i18n::text("ui.unbind_shortcut"))
            .on_press(move |window, cx| {
                unbind_root.update(cx, |this, root_cx| {
                    this.set_shortcut_override(&row.entry_ids(), None, window, root_cx)
                });
            });

        div()
            .min_h(rems(58. / 16.))
            .px_4()
            .py_3()
            .flex()
            .items_center()
            .gap_3()
            .child(label)
            .child(controls)
            .child(unbind)
            .into_any_element()
    }

    pub(super) fn rebuild_key_bindings(&self, suspended: bool, cx: &mut Context<Self>) {
        let overrides = self.settings(cx).keymap_overrides.clone();
        keymap::install_bindings(cx, &overrides, suspended);
        cx.set_menus(crate::app_menus());
    }

    pub(super) fn start_shortcut_recording(
        &mut self,
        id: &'static str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if keymap::entry(id).is_none_or(|entry| entry.fixed) {
            return;
        }
        self.settings_page.shortcut_recording = Some(id);
        self.settings_page.shortcut_conflict = None;
        self.rebuild_key_bindings(true, cx);
        self.settings_page
            .shortcut_recording_focus
            .focus(window, cx);
        cx.notify();
    }

    pub(crate) fn finish_shortcut_recording(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.settings_page.shortcut_recording.take().is_none() {
            return;
        }
        self.settings_page.shortcut_conflict = None;
        self.rebuild_key_bindings(false, cx);
        self.settings_page.focus.focus(window, cx);
        cx.notify();
    }

    pub(super) fn record_shortcut_key(
        &mut self,
        id: &'static str,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.stop_propagation();
        if event.keystroke.key == "escape" {
            self.finish_shortcut_recording(window, cx);
            return;
        }
        let Some(combo) = keymap::combo_from_keystroke(&event.keystroke) else {
            return;
        };
        if let Some(conflict) =
            keymap::find_conflict(&combo, id, &self.settings(cx).keymap_overrides)
        {
            self.settings_page.shortcut_conflict = Some(ShortcutConflict {
                id,
                message: crate::i18n::message_with(
                    "format.already_used_by",
                    &[("command", i18n::message(conflict.title_key))],
                ),
            });
            cx.notify();
            return;
        }
        self.set_shortcut_override(&[id], Some(combo), window, cx);
    }

    pub(super) fn set_shortcut_override(
        &mut self,
        ids: &[&'static str],
        value: Option<keymap::KeyCombo>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if ids
            .iter()
            .any(|id| keymap::entry(id).is_none_or(|entry| entry.fixed && value.is_some()))
        {
            return;
        }
        self.update_app_settings(cx, true, |settings| {
            let mut changed = false;
            for id in ids {
                if settings.keymap_overrides.get(*id) != Some(&value) {
                    settings
                        .keymap_overrides
                        .insert((*id).into(), value.clone());
                    changed = true;
                }
            }
            changed
        });
        self.settings_page.shortcut_recording = None;
        self.settings_page.shortcut_conflict = None;
        self.rebuild_key_bindings(false, cx);
        self.settings_page.focus.focus(window, cx);
        cx.notify();
    }

    pub(super) fn restore_shortcut_default(
        &mut self,
        row: ShortcutRow,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let mut overrides = self.settings(cx).keymap_overrides.clone();
        let mut changed = false;
        for id in row.entry_ids() {
            match keymap::clear_override(id, &mut overrides) {
                Err(conflict) => {
                    self.settings_page.shortcut_conflict = Some(ShortcutConflict {
                        id: row.id(),
                        message: crate::i18n::message_with(
                            "format.already_used_by",
                            &[("command", i18n::message(conflict.title_key))],
                        ),
                    });
                    cx.notify();
                    return;
                }
                Ok(cleared) => changed |= cleared,
            }
        }
        if !changed {
            return;
        }
        self.update_app_settings(cx, true, |settings| {
            settings.keymap_overrides = overrides;
            true
        });
        self.settings_page.shortcut_conflict = None;
        self.rebuild_key_bindings(false, cx);
        self.settings_page.focus.focus(window, cx);
        cx.notify();
    }

    pub(super) fn reset_shortcut_overrides(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.settings(cx).keymap_overrides.is_empty() {
            return;
        }
        self.update_app_settings(cx, true, |settings| {
            settings.keymap_overrides.clear();
            true
        });
        self.settings_page.shortcut_recording = None;
        self.settings_page.shortcut_conflict = None;
        self.rebuild_key_bindings(false, cx);
        self.settings_page.focus.focus(window, cx);
        cx.notify();
    }
}

pub(super) fn shortcut_row_label(
    title: &'static str,
    description: &'static str,
    conflict: Option<String>,
) -> gpui::Div {
    div()
        .min_w(px(0.))
        .flex_1()
        .flex()
        .flex_col()
        .gap(rems(2. / 16.))
        .child(
            div()
                .text_size(theme::text_body())
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme::text())
                .child(title),
        )
        .child(
            div()
                .text_size(theme::text_meta())
                .text_color(theme::muted())
                .child(description),
        )
        .children(conflict.map(|message| {
            div()
                .text_size(theme::text_meta())
                .text_color(theme::danger())
                .child(message)
        }))
}

pub(super) fn shortcut_chip(label: String, editable: bool) -> gpui::Div {
    div()
        .px_2()
        .py_1()
        .rounded(rems(4. / 16.))
        .border_1()
        .border_color(theme::border())
        .bg(theme::raised())
        .text_size(theme::text_meta())
        .line_height(rems(14. / 16.))
        .text_color(theme::muted())
        .when(editable, |chip| {
            chip.cursor_pointer().hover(|chip| {
                chip.border_color(theme::border_strong())
                    .text_color(theme::text())
            })
        })
        .child(label)
}
