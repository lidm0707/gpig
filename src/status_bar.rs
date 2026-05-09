use std::cell::RefCell;
use std::rc::Rc;

use git2::Repository;
use gpui::prelude::*;
use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px};

pub struct StatusBar {
    repo: Rc<RefCell<Option<Repository>>>,
    branch_name: String,
    commit_count: usize,
    dirty_count: usize,
}

impl StatusBar {
    pub fn new(repo: Rc<RefCell<Option<Repository>>>) -> Self {
        let mut bar = Self {
            repo,
            branch_name: String::new(),
            commit_count: 0,
            dirty_count: 0,
        };
        bar.refresh();
        bar
    }

    pub fn refresh(&mut self) {
        let repo = self.repo.borrow();
        let Some(repo) = repo.as_ref() else {
            self.branch_name.clear();
            self.commit_count = 0;
            self.dirty_count = 0;
            return;
        };

        self.branch_name = repo
            .head()
            .ok()
            .and_then(|r| r.shorthand().map(|s| s.to_string()))
            .unwrap_or_default();

        self.commit_count = repo
            .revwalk()
            .and_then(|mut rw| {
                rw.push_head()?;
                Ok(rw.count())
            })
            .unwrap_or(0);

        self.dirty_count = repo.statuses(None).map(|s| s.iter().count()).unwrap_or(0);
    }

    pub fn set_dirty_count(&mut self, count: usize) {
        self.dirty_count = count;
    }
}

impl Render for StatusBar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let branch = if self.branch_name.is_empty() {
            "no repo".to_string()
        } else {
            self.branch_name.clone()
        };

        div()
            .w_full()
            .h(px(24.0))
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px(px(12.0))
            .bg(gpui::rgb(0x1A1A2E))
            .border_t_1()
            .border_color(gpui::rgb(0x333333))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .text_color(gpui::rgb(0x4AE04A))
                            .text_size(px(11.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .font_family("monospace")
                            .child(format!(" {}", branch)),
                    )
                    .child(
                        div()
                            .text_color(gpui::rgb(0x888888))
                            .text_size(px(11.0))
                            .font_family("monospace")
                            .child(format!("{} commits", self.commit_count)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .when(self.dirty_count > 0, |el| {
                        el.child(
                            div()
                                .text_color(gpui::rgb(0xF39C12))
                                .text_size(px(11.0))
                                .font_family("monospace")
                                .child(format!("{} dirty", self.dirty_count)),
                        )
                    })
                    .when(
                        self.dirty_count == 0 && !self.branch_name.is_empty(),
                        |el| {
                            el.child(
                                div()
                                    .text_color(gpui::rgb(0x4AE04A))
                                    .text_size(px(11.0))
                                    .font_family("monospace")
                                    .child("clean"),
                            )
                        },
                    ),
            )
    }
}
