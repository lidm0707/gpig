use gpui::{
    AnyElement, AppContext, Context, Entity, EventEmitter, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Render, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::actions::{OpenFile, Quit};
use crate::garph::{CommitSelected, Garph};
use crate::menu::{DropdownEvent, MenuBar};
use crate::title::{QuitClicked, TitleBar};

pub struct Dock;
pub struct Pane;
pub struct Workspace {
    dock: Option<Entity<Garph>>,
    title_bar: Entity<TitleBar>,
    menu_bar: Entity<MenuBar>,
    selected_commit: Option<CommitSelected>,
    // pane: Vec<Entity<AnyElement>>,
}

impl Workspace {
    pub fn new(dock: Option<Entity<Garph>>, cx: &mut Context<Self>) -> Self {
        let dock_clone = dock.clone();

        let menu_bar = cx.new(|_| MenuBar::new());
        let title_bar = cx.new(|_| TitleBar::new("Dark Pig Git"));

        Self {
            dock: dock_clone,
            title_bar,
            menu_bar,
            selected_commit: None,
        }
    }

    fn on_commit_selected(
        &mut self,
        _garph: Entity<Garph>,
        event: &CommitSelected,
        cx: &mut Context<Self>,
    ) {
        self.set_selected_commit(Some(event.clone()), cx);
    }

    fn on_dropdown_changed(
        &mut self,
        _menu_bar: Entity<MenuBar>,
        _event: &DropdownEvent,
        cx: &mut Context<Self>,
    ) {
        cx.notify();
    }

    fn on_quit_clicked(
        &mut self,
        _title_bar: Entity<TitleBar>,
        _event: &QuitClicked,
        cx: &mut Context<Self>,
    ) {
        println!("test");
        cx.dispatch_action(&Quit);
    }

    pub fn set_title(&mut self, title: &str, cx: &mut Context<Self>) {
        let title = title.to_string();
        self.title_bar
            .update(cx, |title_bar, _| title_bar.set_title(title));
    }

    pub fn set_selected_commit(&mut self, commit: Option<CommitSelected>, cx: &mut Context<Self>) {
        self.selected_commit = commit;
        cx.notify();
    }

    // pub fn add_pane(&mut self, pane: Entity<AnyElement>) {
    //     self.pane.push(pane);
    // }

    // pub fn remove_pane(&mut self, index: usize) {
    //     self.pane.remove(index);
    // }

    // pub fn remove_all_panes(&mut self) {
    //     self.pane.clear();
    // }
}

impl EventEmitter<CommitSelected> for Workspace {}

impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(dock) = &self.dock {
            cx.subscribe(dock, Self::on_commit_selected).detach();
        }

        cx.subscribe(&self.menu_bar, Self::on_dropdown_changed)
            .detach();
        cx.subscribe(&self.title_bar, Self::on_quit_clicked)
            .detach();

        let dock = self.dock.clone().unwrap();
        let title_bar = self.title_bar.clone();
        let menu_bar = self.menu_bar.clone();
        let selected_commit = self.selected_commit.clone();
        // let path_repo = window.use_state(cx, |_, cx| cx.new(|_| "".to_string()));
        // let repo = git2::Repository::open(&path_repo.read(cx).read(cx)).unwrap();
        // let garph = Garph::new(repo);
        let pane_content = if let Some(commit) = selected_commit {
            let timestamp = chrono::DateTime::from_timestamp(commit.timestamp.seconds(), 0)
                .unwrap_or_default()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();

            let message_lines: Vec<String> =
                commit.message.lines().take(5).map(String::from).collect();
            let message_display = message_lines.join("\n");
            let has_more = commit.message.lines().count() > 5;

            div()
                .p_4()
                .flex()
                .flex_col()
                .gap_4()
                .child(
                    div()
                        .text_2xl()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(gpui::rgb(0x000000))
                        .child("Selected Commit"),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(gpui::rgb(0x333333))
                                .child("Author"),
                        )
                        .child(div().text_color(gpui::rgb(0x000000)).child(commit.author)),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(gpui::rgb(0x333333))
                                .child("Date"),
                        )
                        .child(div().text_color(gpui::rgb(0x000000)).child(timestamp)),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(gpui::rgb(0x333333))
                                .child("Message"),
                        )
                        .children({
                            let mut children: Vec<AnyElement> = vec![
                                div()
                                    .text_color(gpui::rgb(0x000000))
                                    .child(message_display)
                                    .into_any_element(),
                            ];
                            if has_more {
                                children.push(
                                    div()
                                        .text_color(gpui::rgb(0x666666))
                                        .child("...")
                                        .into_any_element(),
                                );
                            }
                            children
                        }),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(gpui::rgb(0x333333))
                                .child("Parents"),
                        )
                        .children(if commit.parents.is_empty() {
                            vec![
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(div().text_color(gpui::rgb(0x666666)).child("None"))
                                    .into_any_element(),
                            ]
                        } else {
                            commit
                                .parents
                                .iter()
                                .map(|oid| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(
                                            div()
                                                .text_color(gpui::rgb(0x000000))
                                                .child(oid.to_string()),
                                        )
                                        .into_any_element()
                                })
                                .collect()
                        }),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(gpui::rgb(0x333333))
                                .child("Commit Hash"),
                        )
                        .child(
                            div()
                                .text_color(gpui::rgb(0x000000))
                                .font_family("monospace")
                                .child(commit.oid.to_string()),
                        ),
                )
        } else {
            div()
                .p_4()
                .text_color(gpui::rgb(0x666666))
                .child("Click on a commit to view its details")
        };

        div()
            .size_full()
            .relative()
            .flex()
            .flex_col()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _event, _window, cx| {
                    if this.menu_bar.read(cx).is_dropdown_open() {
                        this.menu_bar.update(cx, |menu_bar, cx| {
                            menu_bar.close_dropdown(cx);
                        });
                        cx.notify();
                    }
                }),
            )
            .child(title_bar)
            .child(menu_bar)
            .child(
                div()
                    .flex_1()
                    .flex()
                    .relative()
                    .child(div().w(gpui::px(300.0)).h_full().child(dock))
                    .child(div().flex_1().bg(gpui::white()).child(pane_content)),
            )
            .when(self.menu_bar.read(cx).is_dropdown_open(), |this| {
                this.child(
                    div()
                        .id("file_menu_dropdown")
                        .text_color(gpui::white())
                        .absolute()
                        .top(px(36.0))
                        .left(px(0.0))
                        .bg(gpui::rgb(0x1a1a1a))
                        .border_1()
                        .border_color(gpui::rgb(0x333333))
                        .shadow_lg()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|_this, _event, _window, cx| {
                                cx.stop_propagation();
                            }),
                        )
                        .child(
                            div()
                                .id("menu_item_new")
                                .text_color(gpui::white())
                                .px(px(16.0))
                                .py(px(8.0))
                                .child("New")
                                .hover(|style| style.bg(gpui::rgb(0x333333)))
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _event, _window, cx| {
                                        this.menu_bar.update(cx, |menu_bar, cx| {
                                            menu_bar.close_dropdown(cx);
                                        });
                                        cx.notify();
                                        cx.stop_propagation();
                                    }),
                                ),
                        )
                        .child(
                            div()
                                .id("menu_item_open")
                                .text_color(gpui::white())
                                .px(px(16.0))
                                .py(px(8.0))
                                .child("Open")
                                .hover(|style| style.bg(gpui::rgb(0x333333)))
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _event, window, cx| {
                                        println!("Open menu item clicked!");
                                        this.menu_bar.update(cx, |menu_bar, cx| {
                                            menu_bar.close_dropdown(cx);
                                        });
                                        cx.stop_propagation();
                                        // cx.dispatch_action(&OpenFile);
                                        // cx.dispatch_action(&OpenFile);
                                        window.dispatch_action(Box::new(OpenFile), cx);
                                    }),
                                ),
                        )
                        .child(
                            div()
                                .id("menu_item_save")
                                .text_color(gpui::white())
                                .px(px(16.0))
                                .py(px(8.0))
                                .child("Save")
                                .hover(|style| style.bg(gpui::rgb(0x333333)))
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _event, _window, cx| {
                                        this.menu_bar.update(cx, |menu_bar, cx| {
                                            menu_bar.close_dropdown(cx);
                                        });
                                        cx.notify();
                                        cx.stop_propagation();
                                    }),
                                ),
                        )
                        .child(
                            div()
                                .id("menu_item_exit")
                                .text_color(gpui::white())
                                .px(px(16.0))
                                .py(px(8.0))
                                .child("Exit")
                                .hover(|style| style.bg(gpui::rgb(0x333333)))
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _event, _window, cx| {
                                        this.menu_bar.update(cx, |menu_bar, cx| {
                                            menu_bar.close_dropdown(cx);
                                        });
                                        cx.notify();
                                        cx.stop_propagation();
                                    }),
                                ),
                        ),
                )
            })
    }
}
