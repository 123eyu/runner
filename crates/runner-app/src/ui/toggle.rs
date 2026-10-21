use std::rc::Rc;

use gpui::prelude::*;
use gpui::{
    div, px, rems, App, BoxShadow, CursorStyle, ElementId, KeyDownEvent, RenderOnce, Window,
};

use crate::theme;

pub type ToggleHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct Toggle {
    id: ElementId,
    on: bool,
    disabled: bool,
    on_change: Option<ToggleHandler>,
}

impl Toggle {
    pub fn new(id: impl Into<ElementId>, on: bool) -> Self {
        Self {
            id: id.into(),
            on,
            disabled: false,
            on_change: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Toggle {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let handler = self.on_change;
        let disabled = self.disabled;
        let on = self.on;
        let mut toggle = div()
            .id(self.id)
            .tab_index(0)
            .tab_stop(!disabled)
            .flex()
            .items_center()
            .justify_start()
            .when(on, |toggle| toggle.justify_end())
            .size_full()
            .w(rems(32. / 16.))
            .h(rems(18. / 16.))
            .p(rems(2. / 16.))
            .rounded_full()
            .bg(if on {
                theme::with_alpha(theme::accent(), 0.15)
            } else {
                theme::raised()
            })
            .opacity(if disabled { 0.5 } else { 1. })
            .cursor(if disabled {
                CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::PointingHand
            })
            .focus_visible(|style| {
                style.shadow(vec![BoxShadow {
                    color: theme::border_strong(),
                    offset: gpui::point(px(0.), px(0.)),
                    blur_radius: px(0.),
                    spread_radius: px(2.),
                    inset: false,
                }])
            })
            .child(div().size(rems(14. / 16.)).rounded_full().bg(if on {
                theme::accent()
            } else {
                theme::faint()
            }));
        if !disabled {
            if let Some(handler) = handler {
                toggle = toggle
                    .on_click(move |_, window, cx| handler(!on, window, cx))
                    .on_key_down(|event: &KeyDownEvent, _, cx| {
                        if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                            cx.stop_propagation();
                        }
                    });
            }
        }
        toggle
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{
        Context, KeyUpEvent, Keystroke, Modifiers, Render, TestAppContext, VisualTestContext,
    };

    struct ToggleHost {
        on: bool,
        changes: Vec<bool>,
        row_activations: usize,
    }

    impl Render for ToggleHost {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let host = cx.entity();
            div()
                .id("toggle-row")
                .tab_index(0)
                .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, _| {
                    if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                        this.row_activations += 1;
                    }
                }))
                .child(
                    div()
                        .debug_selector(|| "TEST_TOGGLE".into())
                        .w(rems(32. / 16.))
                        .child(Toggle::new("toggle", self.on).on_change(move |on, _, cx| {
                            host.update(cx, |this, cx| {
                                this.on = on;
                                this.changes.push(on);
                                cx.notify();
                            });
                        })),
                )
        }
    }

    #[test]
    fn keyboard_press_and_release_toggle_once_without_activating_the_row() {
        for initial in [false, true] {
            for key in ["enter", "space"] {
                let mut cx = TestAppContext::single();
                let host = cx.add_window(|_, _| ToggleHost {
                    on: initial,
                    changes: Vec::new(),
                    row_activations: 0,
                });
                cx.run_until_parked();
                let mut visual = VisualTestContext::from_window(host.into(), &cx);
                host.update(&mut visual, |_, window, cx| {
                    window.focus_next(cx);
                    window.focus_next(cx);
                })
                .unwrap();
                visual.run_until_parked();
                for expected in [!initial, initial] {
                    visual.simulate_keystrokes(key);
                    visual.simulate_event(KeyUpEvent {
                        keystroke: Keystroke::parse(key).unwrap(),
                    });
                    visual.run_until_parked();
                    host.read_with(&visual, |host, _| {
                        assert_eq!(host.on, expected, "{key} from {initial}");
                        assert_eq!(host.row_activations, 0);
                    })
                    .unwrap();
                }
                host.read_with(&visual, |host, _| {
                    assert_eq!(host.changes, [!initial, initial])
                })
                .unwrap();
                let toggle = visual.debug_bounds("TEST_TOGGLE").unwrap();
                visual.simulate_click(toggle.center(), Modifiers::default());
                visual.run_until_parked();
                host.read_with(&visual, |host, _| {
                    assert_eq!(host.changes, [!initial, initial, !initial])
                })
                .unwrap();
            }
        }
    }
}
