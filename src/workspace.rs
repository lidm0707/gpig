use gpui::prelude::*;
use gpui::{
    AnyElement, Context, Entity, EventEmitter, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Render, SharedString, Styled, Window, div, px,
};

use crate::actions::Quit;
use crate::branch::{BranchCheckedOut, BranchPanel};
use crate::diff_viewer;
use crate::garph::{self, ChangedFile, CommitSelected, Garph};
use crate::menu::{DropdownEvent, MenuBar};
use crate::path_bar::{self, PathBar, RepoPathSubmitted, SearchPathCleared, SearchPathSubmitted};
use crate::repo_picker;
use crate::status_bar::StatusBar;
use crate::status_panel::StatusPanel;
use crate::title::{QuitClicked, TitleBar};
use std::sync::mpsc::{self, Receiver};

pub struct Dock;
pub struct Pane;
pub struct Workspace {
    dock: Option<Entity<Garph>>,
    title_bar: Entity<TitleBar>,
    menu_bar: Entity<MenuBar>,
    path_bar: Entity<PathBar>,
    branch_panel: Option<Entity<BranchPanel>>,
    status_panel: Option<Entity<StatusPanel>>,
    status_bar: Option<Entity<StatusBar>>,
    selected_commit: Option<CommitSelected>,
    changed_files: Vec<ChangedFile>,
    expanded_file: Option<usize>,
    file_diff: Option<String>,
    active_pane: ActivePane,
    loading_diff: bool,
    current_commit_oid: Option<git2::Oid>,
    pending_files_rx: Option<Receiver<Vec<ChangedFile>>>,
    pending_diff_rx: Option<Receiver<String>>,
    pending_paths_rx: Option<Receiver<Vec<String>>>,
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
        let path_bar = cx.new(PathBar::new);

        let branch_panel = dock.as_ref().map(|garph| {
            let repo = garph.read(cx).repo();
            cx.new(|_| BranchPanel::new(repo.clone()))
        });

        let status_panel = dock.as_ref().map(|garph| {
            let repo = garph.read(cx).repo();
            cx.new(|_| StatusPanel::new(repo.clone()))
        });

        let status_bar = dock.as_ref().map(|garph| {
            let repo = garph.read(cx).repo();
            cx.new(|_| StatusBar::new(repo))
        });

        cx.subscribe(&path_bar, Self::on_repo_path_submitted)
            .detach();
        cx.subscribe(&path_bar, Self::on_search_path_submitted)
            .detach();
        cx.subscribe(&path_bar, Self::on_search_path_cleared)
            .detach();

        if let Some(ref garph) = dock {
            cx.subscribe(garph, Self::on_repo_path_changed).detach();
        }

        Self {
            dock: dock_clone,
            title_bar,
            menu_bar,
            path_bar,
            branch_panel,
            status_panel,
            status_bar,
            selected_commit: None,
            changed_files: Vec::new(),
            expanded_file: None,
            file_diff: None,
            active_pane: ActivePane::Content,
            loading_diff: false,
            current_commit_oid: None,
            pending_files_rx: None,
            pending_diff_rx: None,
            pending_paths_rx: None,
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
        let repo_path = garph.read(cx).repo_path().map(|s| s.to_string());
        let oid = commit.oid;

        self.changed_files.clear();
        self.expanded_file = None;
        self.file_diff = None;
        self.current_commit_oid = Some(oid);
        cx.notify();

        let Some(repo_path) = repo_path else {
            return;
        };

        let (tx, rx) = mpsc::channel();
        self.pending_files_rx = Some(rx);

        std::thread::spawn(move || {
            let result = garph::get_changed_files_bg(repo_path, oid).unwrap_or_else(|e| {
                eprintln!("Failed to get changed files: {}", e);
                Vec::new()
            });
            let _ = tx.send(result);
        });
    }

    fn on_file_toggled(&mut self, file_index: usize, garph: Entity<Garph>, cx: &mut Context<Self>) {
        if file_index >= self.changed_files.len() {
            return;
        }

        if self.expanded_file == Some(file_index) {
            self.expanded_file = None;
            self.file_diff = None;
            self.loading_diff = false;
            self.pending_diff_rx = None;
            cx.notify();
            return;
        }

        self.expanded_file = Some(file_index);
        self.file_diff = None;
        self.loading_diff = true;
        cx.notify();

        let file = self.changed_files[file_index].clone();
        let commit_oid = match self.current_commit_oid {
            Some(oid) => oid,
            None => {
                self.file_diff = Some("No commit selected".to_string());
                self.loading_diff = false;
                cx.notify();
                return;
            }
        };

        let repo_path = match garph.read(cx).repo_path().map(|s| s.to_string()) {
            Some(p) => p,
            None => {
                self.file_diff = Some("No repo".to_string());
                self.loading_diff = false;
                cx.notify();
                return;
            }
        };

        let (tx, rx) = mpsc::channel();
        self.pending_diff_rx = Some(rx);

        std::thread::spawn(move || {
            let result = garph::compute_file_diff_bg(repo_path, commit_oid, file.path.clone())
                .unwrap_or_else(|e| format!("Failed to compute diff: {}", e));
            let _ = tx.send(result);
        });
    }

    fn on_branch_checked_out(
        &mut self,
        _branch_panel: Entity<BranchPanel>,
        _event: &BranchCheckedOut,
        cx: &mut Context<Self>,
    ) {
        if let Some(dock) = &self.dock {
            dock.update(cx, |garph, cx| {
                garph.mark_dirty();
                cx.notify();
            });
        }
        self.reload_status_panels(cx);
    }

    fn on_repo_path_submitted(
        &mut self,
        _path_bar: Entity<PathBar>,
        event: &RepoPathSubmitted,
        cx: &mut Context<Self>,
    ) {
        if let Some(dock) = &self.dock {
            let result = dock.update(cx, |garph, _cx| garph.update_repo(&event.path));
            if let Err(e) = result {
                self.path_bar.update(cx, |pb, _| {
                    pb.set_error(Some(format!("Failed: {}", e)));
                });
            } else {
                self.path_bar.update(cx, |pb, _| {
                    pb.set_error(None);
                });
                self.spawn_path_collection(&event.path);
            }
        }
        cx.notify();
    }

    fn on_search_path_submitted(
        &mut self,
        _path_bar: Entity<PathBar>,
        event: &SearchPathSubmitted,
        cx: &mut Context<Self>,
    ) {
        if let Some(dock) = &self.dock {
            dock.update(cx, |garph, cx| {
                garph.set_search_path(Some(event.path.clone()));
                cx.notify();
            });
        }
        cx.notify();
    }

    fn on_search_path_cleared(
        &mut self,
        _path_bar: Entity<PathBar>,
        _event: &SearchPathCleared,
        cx: &mut Context<Self>,
    ) {
        if let Some(dock) = &self.dock {
            dock.update(cx, |garph, cx| {
                garph.set_search_path(None);
                cx.notify();
            });
        }
        cx.notify();
    }

    fn on_repo_path_changed(
        &mut self,
        garph: Entity<Garph>,
        _event: &garph::RepoPathChanged,
        cx: &mut Context<Self>,
    ) {
        self.reload_status_panels(cx);
        self.path_bar.update(cx, |pb, cx| {
            pb.clear_search(cx);
        });
        if let Some(path) = garph.read(cx).repo_path().map(|s| s.to_string()) {
            self.spawn_path_collection(&path);
        }
        cx.notify();
    }

    fn spawn_path_collection(&mut self, repo_path: &str) {
        let (tx, rx) = mpsc::channel();
        self.pending_paths_rx = Some(rx);
        let repo_path = repo_path.to_string();
        std::thread::spawn(move || {
            let result = garph::collect_paths_bg(repo_path).unwrap_or_default();
            let _ = tx.send(result);
        });
    }

    fn reload_status_panels(&mut self, cx: &mut Context<Self>) {
        if let Some(sp) = &self.status_panel {
            sp.update(cx, |sp, cx| {
                sp.reload();
                cx.notify();
            });
        }
        if let Some(sb) = &self.status_bar {
            sb.update(cx, |sb, cx| {
                sb.refresh();
                cx.notify();
            });
        }
        if let Some(bp) = &self.branch_panel {
            bp.update(cx, |bp, cx| {
                bp.reload();
                cx.notify();
            });
        }
    }

    fn render_file_panel(&self, dock: &Entity<Garph>, cx: &mut Context<Self>) -> AnyElement {
        if self.changed_files.is_empty() {
            return div()
                .flex()
                .items_center()
                .justify_center()
                .size_full()
                .bg(gpui::rgb(0x1E1E1E))
                .text_color(gpui::rgb(0x888888))
                .child("No files changed in this commit")
                .into_any();
        }

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
                        self.render_file_row(index, file, &dock_for_file, cx)
                    })),
            )
            .into_any()
    }

    fn render_file_row(
        &self,
        index: usize,
        file: &ChangedFile,
        dock: &Entity<Garph>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let is_expanded = self.expanded_file == Some(index);
        let dock_clone = dock.clone();

        let status_color = match file.status {
            git2::Delta::Added => gpui::rgb(0x2ECC71),
            git2::Delta::Deleted => gpui::rgb(0xE74C3C),
            git2::Delta::Modified => gpui::rgb(0xF39C12),
            git2::Delta::Renamed => gpui::rgb(0x3498DB),
            git2::Delta::Copied => gpui::rgb(0x9B59B6),
            _ => gpui::rgb(0x888888),
        };

        let status_text = match file.status {
            git2::Delta::Added => "A",
            git2::Delta::Deleted => "D",
            git2::Delta::Modified => "M",
            git2::Delta::Renamed => "R",
            git2::Delta::Copied => "C",
            _ => "?",
        };

        let arrow = if is_expanded { "▼" } else { "▶" };

        let row_bg = if is_expanded {
            gpui::rgb(0x252525)
        } else {
            gpui::rgb(0x1E1E1E)
        };

        let mut row = div()
            .w_full()
            .border_b_1()
            .border_color(gpui::rgb(0x2A2A2A))
            .child(
                div()
                    .id(SharedString::from(format!("file-row-{}", index)))
                    .w_full()
                    .flex()
                    .flex_row()
                    .items_center()
                    .px(px(12.0))
                    .py(px(6.0))
                    .bg(row_bg)
                    .hover(|style| style.bg(gpui::rgb(0x2A2A2A)))
                    .cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _event, _window, cx| {
                            this.on_file_toggled(index, dock_clone.clone(), cx);
                        }),
                    )
                    .child(
                        div()
                            .w(px(14.0))
                            .text_color(gpui::rgb(0x888888))
                            .text_size(px(10.0))
                            .child(arrow),
                    )
                    .child(
                        div()
                            .w(px(24.0))
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
                            .child(file.path.clone()),
                    ),
            );

        if is_expanded {
            row = row.child(self.render_inline_diff());
        }

        row.into_any()
    }

    fn render_inline_diff(&self) -> AnyElement {
        if self.loading_diff {
            return div()
                .w_full()
                .flex()
                .items_center()
                .justify_center()
                .py(px(12.0))
                .bg(gpui::rgb(0x1E1E1E))
                .text_color(gpui::rgb(0x888888))
                .text_size(px(12.0))
                .child("Loading diff...")
                .into_any();
        }

        let raw = match &self.file_diff {
            Some(d) => d.clone(),
            None => {
                return div()
                    .w_full()
                    .py(px(8.0))
                    .bg(gpui::rgb(0x1E1E1E))
                    .text_color(gpui::rgb(0x666666))
                    .text_size(px(12.0))
                    .child("No diff available")
                    .into_any();
            }
        };

        if diff_viewer::is_binary_or_error(&raw) {
            return div()
                .w_full()
                .px(px(12.0))
                .py(px(8.0))
                .bg(gpui::rgb(0x1E1E1E))
                .text_color(gpui::rgb(0x888888))
                .text_size(px(12.0))
                .font_family("monospace")
                .child(raw)
                .into_any();
        }

        let rows = diff_viewer::parse_diff(&raw);
        diff_viewer::render_side_by_side(&rows)
    }

    fn poll_pending_results(&mut self, cx: &mut Context<Self>) {
        if let Some(rx) = &self.pending_files_rx {
            if let Ok(files) = rx.try_recv() {
                self.changed_files = files;
                self.pending_files_rx = None;
                cx.notify();
            } else {
                cx.notify();
            }
        }
        if let Some(rx) = &self.pending_diff_rx {
            if let Ok(diff) = rx.try_recv() {
                self.file_diff = Some(diff);
                self.loading_diff = false;
                self.pending_diff_rx = None;
                cx.notify();
            } else {
                cx.notify();
            }
        }
        if let Some(rx) = &self.pending_paths_rx {
            if let Ok(paths) = rx.try_recv() {
                self.path_bar.update(cx, |pb, _| {
                    pb.set_suggest_paths(paths);
                });
                self.pending_paths_rx = None;
                cx.notify();
            } else {
                cx.notify();
            }
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
}

impl EventEmitter<CommitSelected> for Workspace {}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.poll_pending_results(cx);

        if let Some(dock) = &self.dock {
            cx.subscribe(dock, Self::on_commit_selected).detach();
        }

        cx.subscribe(&self.menu_bar, Self::on_dropdown_changed)
            .detach();
        cx.subscribe(&self.title_bar, Self::on_quit_clicked)
            .detach();
        if let Some(bp) = &self.branch_panel {
            cx.subscribe(bp, Self::on_branch_checked_out).detach();
        }

        let dock = self.dock.clone().unwrap();
        let title_bar = self.title_bar.clone();
        let menu_bar = self.menu_bar.clone();
        let path_bar = self.path_bar.clone();

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
                    }
                    this.path_bar.update(cx, |pb, cx| {
                        pb.close_repo_dropdown(cx);
                        pb.close_suggest();
                    });
                    cx.notify();
                }),
            )
            .child(title_bar)
            .child(menu_bar)
            .child(path_bar)
            .child(
                div()
                    .flex_1()
                    .flex()
                    .relative()
                    .child(
                        div()
                            .w(gpui::px(300.0))
                            .h_full()
                            .flex()
                            .flex_col()
                            .border_r_1()
                            .border_color(gpui::rgb(0x333333))
                            .bg(gpui::rgb(0x282828))
                            .when_some(self.branch_panel.clone(), |el, bp| {
                                el.child(
                                    div()
                                        .w_full()
                                        .h(gpui::px(140.0))
                                        .border_b_1()
                                        .border_color(gpui::rgb(0x333333))
                                        .child(bp),
                                )
                            })
                            .when_some(self.status_panel.clone(), |el, sp| {
                                el.child(
                                    div()
                                        .w_full()
                                        .h(gpui::px(140.0))
                                        .border_b_1()
                                        .border_color(gpui::rgb(0x333333))
                                        .child(sp),
                                )
                            })
                            .child(div().flex_1().child(dock.clone())),
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
                            .child(self.render_file_panel(&dock, cx)),
                    ),
            )
            .when_some(self.status_bar.clone(), |el, sb| el.child(sb))
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
                                .id("menu_item_open")
                                .text_color(gpui::white())
                                .px(px(16.0))
                                .py(px(8.0))
                                .child("Open")
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
            .when(
                self.path_bar.read(cx).repo_picker().read(cx).is_open(),
                |el| {
                    let picker = self.path_bar.read(cx).repo_picker().clone();
                    if let Some(dropdown) = repo_picker::render_dropdown(&picker, cx) {
                        el.child(dropdown)
                    } else {
                        el
                    }
                },
            )
            .when(self.path_bar.read(cx).suggest_open(), |el| {
                let pb = self.path_bar.clone();
                if let Some(dropdown) = path_bar::render_suggest_dropdown(&pb, cx) {
                    el.child(dropdown)
                } else {
                    el
                }
            })
    }
}
