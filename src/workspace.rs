use gpui::prelude::*;
use gpui::{
    AnyElement, AppContext, Context, Entity, EventEmitter, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Render, Styled, Window, div, px,
};

use crate::actions::{OpenFile, Quit};
use crate::garph::{ChangedFile, CommitSelected, Garph};
use crate::menu::{DropdownEvent, MenuBar};
use crate::title::{QuitClicked, TitleBar};

pub struct Dock;
pub struct Pane;
pub struct Workspace {
    dock: Option<Entity<Garph>>,
    title_bar: Entity<TitleBar>,
    menu_bar: Entity<MenuBar>,
    selected_commit: Option<CommitSelected>,
    changed_files: Vec<ChangedFile>,
    selected_file: Option<usize>,
    file_diff: Option<String>,
    active_pane: ActivePane,
    loading_diff: bool,
    current_commit_oid: Option<git2::Oid>,
    // pane: Vec<Entity<AnyElement>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActivePane {
    Dock,
    Content,
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
            changed_files: Vec::new(),
            selected_file: None,
            file_diff: None,
            active_pane: ActivePane::Content,
            loading_diff: false,
            current_commit_oid: None,
        }
    }

    fn on_commit_selected(
        &mut self,
        garph: Entity<Garph>,
        event: &CommitSelected,
        cx: &mut Context<Self>,
    ) {
        let event_clone = event.clone();

        self.set_selected_commit(Some(event_clone.clone()), cx);

        // Immediately load changed files when commit is selected
        self.load_changed_files(&garph, &event_clone, cx);
    }

    fn load_changed_files(
        &mut self,
        garph: &Entity<Garph>,
        commit: &CommitSelected,
        cx: &mut Context<Self>,
    ) {
        let files = garph.update(cx, |garph, _cx| {
            match garph.get_changed_files(&commit.oid) {
                Ok(files) => files,
                Err(e) => {
                    eprintln!("Failed to get changed files: {}", e);
                    Vec::new()
                }
            }
        });

        self.changed_files = files;
        self.selected_file = None;
        self.file_diff = None;
        self.current_commit_oid = Some(commit.oid);
        cx.notify();
    }

    fn on_file_selected(
        &mut self,
        file_index: usize,
        garph: Entity<Garph>,
        cx: &mut Context<Self>,
    ) {
        // Safety check to prevent out of bounds
        if file_index >= self.changed_files.len() {
            eprintln!(
                "Invalid file index: {} (total files: {})",
                file_index,
                self.changed_files.len()
            );
            return;
        }

        self.selected_file = Some(file_index);
        self.loading_diff = true;
        cx.notify();

        let file = self.changed_files[file_index].clone();

        // Get commit OID - if none available, show error
        let commit_oid = match self.current_commit_oid {
            Some(oid) => oid,
            None => {
                self.file_diff = Some("No commit selected".to_string());
                self.loading_diff = false;
                cx.notify();
                return;
            }
        };

        let diff_content = garph.update(cx, |garph, _cx| {
            match garph.compute_file_diff(&commit_oid, &file.path) {
                Ok(diff) => diff,
                Err(e) => format!("Failed to compute diff: {}", e),
            }
        });

        self.file_diff = Some(diff_content);
        self.loading_diff = false;
        cx.notify();
    }

    fn on_back_to_file_list(&mut self, cx: &mut Context<Self>) {
        self.selected_file = None;
        self.file_diff = None;
        self.loading_diff = false;
        cx.notify();
    }

    fn render_file_list(&self, dock: &Entity<Garph>, cx: &mut Context<Self>) -> AnyElement {
        if self.changed_files.is_empty() {
            div()
                .flex()
                .items_center()
                .justify_center()
                .size_full()
                .bg(gpui::rgb(0x1E1E1E))
                .text_color(gpui::rgb(0x888888))
                .child("No files changed in this commit")
                .into_any()
        } else {
            let dock_for_file = dock.clone();
            div()
                .size_full()
                .flex()
                .flex_col()
                .bg(gpui::rgb(0x1E1E1E))
                .child(
                    div()
                        .w_full()
                        .px(px(12.0))
                        .py(px(8.0))
                        .border_b_1()
                        .border_color(gpui::rgb(0x333333))
                        .bg(gpui::rgb(0x252525))
                        .text_color(gpui::white())
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_size(px(14.0))
                        .child(format!("Changed Files ({})", self.changed_files.len())),
                )
                .child(
                    div()
                        .id("changed-files-list")
                        .flex_1()
                        .bg(gpui::rgb(0x1E1E1E))
                        .flex()
                        .flex_col()
                        .overflow_y_scroll()
                        .children(self.changed_files.iter().enumerate().map(|(index, file)| {
                            let dock_for_file_clone = dock_for_file.clone();
                            let file_path = file.path.clone();
                            let status = file.status;

                            let status_color = match status {
                                git2::Delta::Added => gpui::rgb(0x2ECC71),
                                git2::Delta::Deleted => gpui::rgb(0xE74C3C),
                                git2::Delta::Modified => gpui::rgb(0xF39C12),
                                git2::Delta::Renamed => gpui::rgb(0x3498DB),
                                git2::Delta::Copied => gpui::rgb(0x9B59B6),
                                _ => gpui::rgb(0x888888),
                            };

                            let status_text = match status {
                                git2::Delta::Added => "A",
                                git2::Delta::Deleted => "D",
                                git2::Delta::Modified => "M",
                                git2::Delta::Renamed => "R",
                                git2::Delta::Copied => "C",
                                _ => "?",
                            };

                            div()
                                .w_full()
                                .flex()
                                .flex_row()
                                .items_center()
                                .px(px(12.0))
                                .py(px(8.0))
                                .border_b_1()
                                .border_color(gpui::rgb(0x2A2A2A))
                                .hover(|style| style.bg(gpui::rgb(0x2A2A2A)))
                                .cursor_pointer()
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _event, _window, cx| {
                                        this.on_file_selected(
                                            index,
                                            dock_for_file_clone.clone(),
                                            cx,
                                        );
                                    }),
                                )
                                .child(
                                    div()
                                        .w(px(30.0))
                                        .text_color(status_color)
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_size(px(12.0))
                                        .child(status_text),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .text_color(gpui::rgb(0xCCCCCC))
                                        .text_size(px(13.0))
                                        .font_family("monospace")
                                        .overflow_hidden()
                                        .whitespace_nowrap()
                                        .max_w(px(400.0))
                                        .child(file_path),
                                )
                                .into_any()
                        })),
                )
                .into_any()
        }
    }

    fn render_file_diff(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.loading_diff {
            div()
                .flex()
                .items_center()
                .justify_center()
                .size_full()
                .bg(gpui::rgb(0x1E1E1E))
                .flex_col()
                .gap_4()
                .child(
                    div()
                        .text_color(gpui::rgb(0x888888))
                        .text_size(px(14.0))
                        .child("Loading diff..."),
                )
                .child(
                    div()
                        .text_color(gpui::rgb(0x666666))
                        .text_size(px(12.0))
                        .child("Computing file differences"),
                )
                .into_any()
        } else if let Some(file_index) = self.selected_file {
            // Safety check to prevent out of bounds
            if file_index >= self.changed_files.len() {
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size_full()
                    .bg(gpui::rgb(0x1E1E1E))
                    .text_color(gpui::rgb(0xE74C3C))
                    .child("Error: Invalid file selection")
                    .into_any()
            } else {
                let file = &self.changed_files[file_index];
                let title = format!("Diff: {}", file.path);
                let diff_content = self
                    .file_diff
                    .as_ref()
                    .cloned()
                    .unwrap_or_else(|| "No diff available".to_string());

                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .bg(gpui::rgb(0x1E1E1E))
                    // Header with back button
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .px(px(12.0))
                            .py(px(8.0))
                            .border_b_1()
                            .border_color(gpui::rgb(0x333333))
                            .bg(gpui::rgb(0x252525))
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_color(gpui::rgb(0x888888))
                                            .text_size(px(16.0))
                                            .px(px(8.0))
                                            .py(px(4.0))
                                            .cursor_pointer()
                                            .hover(|style| style.bg(gpui::rgb(0x444444)))
                                            .rounded(px(4.0))
                                            .child("←")
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(|this, _event, _window, cx| {
                                                    this.on_back_to_file_list(cx);
                                                }),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .text_color(gpui::white())
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_size(px(14.0))
                                            .child(title),
                                    ),
                            )
                            .child(
                                div()
                                    .text_color(gpui::rgb(0x888888))
                                    .text_size(px(16.0))
                                    .px(px(8.0))
                                    .py(px(4.0))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(gpui::rgb(0x444444)))
                                    .rounded(px(4.0))
                                    .child("✕")
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _event, _window, cx| {
                                            this.on_back_to_file_list(cx);
                                        }),
                                    ),
                            ),
                    )
                    // Diff content
                    .child(
                        div()
                            .flex_1()
                            .id("file_diff_content")
                            .bg(gpui::rgb(0x1E1E1E))
                            .flex()
                            .flex_col()
                            .overflow_y_scroll()
                            .px(px(12.0))
                            .py(px(8.0))
                            .child(
                                div()
                                    .text_color(gpui::rgb(0xCCCCCC))
                                    .text_size(px(12.0))
                                    .font_family("monospace")
                                    .child(diff_content),
                            ),
                    )
                    .into_any()
            }
        } else {
            div()
                .flex()
                .items_center()
                .justify_center()
                .size_full()
                .bg(gpui::rgb(0x1E1E1E))
                .text_color(gpui::rgb(0x888888))
                .child("Select a file to view diff")
                .into_any()
        }
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

        // let path_repo = window.use_state(cx, |_, cx| cx.new(|_| "".to_string()));
        // let repo = git2::Repository::open(&path_repo.read(cx).read(cx)).unwrap();
        // let garph = Garph::new(repo);

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
                    .child(
                        div()
                            .w(gpui::px(300.0))
                            .h_full()
                            .border_r_1()
                            .border_color(if self.active_pane == ActivePane::Dock {
                                gpui::rgb(0x4A90D9)
                            } else {
                                gpui::rgb(0xE5E5E5)
                            })
                            .bg(if self.active_pane == ActivePane::Dock {
                                gpui::rgb(0xF0F8FF)
                            } else {
                                gpui::rgb(0x282828)
                            })
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _event, _window, cx| {
                                    this.active_pane = ActivePane::Dock;
                                    cx.notify();
                                }),
                            )
                            .child(dock.clone()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .bg(gpui::rgb(0x1E1E1E))
                            .border_l_1()
                            .border_color(if self.active_pane == ActivePane::Content {
                                gpui::rgb(0x4A90D9)
                            } else {
                                gpui::rgb(0xE5E5E5)
                            })
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _event, _window, cx| {
                                    this.active_pane = ActivePane::Content;
                                    cx.notify();
                                }),
                            )
                            .child(if self.selected_file.is_some() {
                                self.render_file_diff(cx)
                            } else {
                                self.render_file_list(&dock, cx)
                            }),
                    ),
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
