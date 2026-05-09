use std::cell::RefCell;
use std::rc::Rc;

use git2::Repository;
use gpui::prelude::*;
use gpui::{
    AnyElement, Context, EventEmitter, InteractiveElement, IntoElement, MouseButton, ParentElement,
    Render, SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};

#[derive(Clone, Debug)]
pub struct BranchInfo {
    pub name: String,
    pub is_head: bool,
}

#[derive(Clone, Debug)]
pub struct BranchCheckedOut {
    pub name: String,
}

pub struct BranchPanel {
    repo: Rc<RefCell<Option<Repository>>>,
    branches: Vec<BranchInfo>,
}

impl EventEmitter<BranchCheckedOut> for BranchPanel {}

impl BranchPanel {
    pub fn new(repo: Rc<RefCell<Option<Repository>>>) -> Self {
        let mut panel = Self {
            repo,
            branches: Vec::new(),
        };
        panel.reload();
        panel
    }

    pub fn reload(&mut self) {
        self.branches.clear();
        let repo = self.repo.borrow();
        let Some(repo) = repo.as_ref() else {
            return;
        };

        let head_name = repo
            .head()
            .ok()
            .and_then(|r| r.shorthand().map(|s| s.to_string()));

        let Ok(branches) = repo.branches(Some(git2::BranchType::Local)) else {
            return;
        };

        for branch in branches.flatten() {
            let (b, _) = branch;
            let name = b.name().ok().flatten().unwrap_or("").to_string();
            let is_head = head_name.as_ref() == Some(&name);
            self.branches.push(BranchInfo { name, is_head });
        }
    }

    fn do_checkout(&self, name: &str) -> Result<(), git2::Error> {
        let repo = self.repo.borrow();
        let Some(repo) = repo.as_ref() else {
            return Err(git2::Error::from_str("No repository loaded"));
        };
        let refname = format!("refs/heads/{}", name);
        repo.set_head(&refname)?;
        repo.checkout_head(None)?;
        Ok(())
    }

    pub fn checkout(&mut self, name: &str, cx: &mut Context<Self>) {
        if let Err(e) = self.do_checkout(name) {
            eprintln!("checkout failed: {}", e);
            return;
        }
        self.reload();
        cx.emit(BranchCheckedOut {
            name: name.to_string(),
        });
        cx.notify();
    }

    pub fn current_branch(&self) -> Option<&str> {
        self.branches
            .iter()
            .find(|b| b.is_head)
            .map(|b| b.name.as_str())
    }

    pub fn branches(&self) -> &[BranchInfo] {
        &self.branches
    }
}

impl Render for BranchPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_panel(cx)
    }
}

impl BranchPanel {
    fn render_panel(&self, cx: &mut Context<Self>) -> AnyElement {
        let has_repo = self.repo.borrow().is_some();

        if !has_repo {
            return div()
                .size_full()
                .flex()
                .flex_col()
                .bg(gpui::rgb(0x1E1E1E))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size_full()
                        .text_color(gpui::rgb(0x666666))
                        .text_size(px(12.0))
                        .child("No repo"),
                )
                .into_any();
        }

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(gpui::rgb(0x1E1E1E))
            .child(
                div()
                    .w_full()
                    .px(px(10.0))
                    .py(px(6.0))
                    .border_b_1()
                    .border_color(gpui::rgb(0x333333))
                    .bg(gpui::rgb(0x252525))
                    .text_color(gpui::rgb(0xCCCCCC))
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_size(px(12.0))
                    .child("Branches"),
            )
            .child(
                div()
                    .id("branch_list")
                    .flex_1()
                    .overflow_y_scroll()
                    .children(self.branches.iter().map(|b| {
                        let name = b.name.clone();
                        let is_head = b.is_head;

                        let bg = if is_head {
                            gpui::rgb(0x2A3A2A)
                        } else {
                            gpui::rgb(0x1E1E1E)
                        };
                        let text_color = if is_head {
                            gpui::rgb(0x4AE04A)
                        } else {
                            gpui::rgb(0xCCCCCC)
                        };

                        div()
                            .id(SharedString::from(name.clone()))
                            .w_full()
                            .px(px(10.0))
                            .py(px(4.0))
                            .bg(bg)
                            .hover(|s| s.bg(gpui::rgb(0x2A2A2A)))
                            .cursor_pointer()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_color(text_color)
                                    .text_size(px(12.0))
                                    .font_family("monospace")
                                    .child(if is_head {
                                        format!("* {}", name)
                                    } else {
                                        format!("  {}", name)
                                    }),
                            )
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _event, _window, cx| {
                                    if !is_head {
                                        this.checkout(&name, cx);
                                    }
                                }),
                            )
                    })),
            )
            .into_any()
    }
}
