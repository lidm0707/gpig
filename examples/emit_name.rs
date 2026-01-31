use std::error::Error;

use gpui::{
    App, AppContext, Application, Context, Entity, EventEmitter, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Render, Styled, Window, WindowOptions, div,
};

// Event for name transfer
pub struct NameTransferEvent {
    name: String,
}

// Player struct - just state, no Render implementation
pub struct Player {
    name: String,
}

impl EventEmitter<NameTransferEvent> for Player {}

impl Player {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

// Workspace manages the UI and player entities
pub struct Workspace {
    player1: Entity<Player>,
    player2: Entity<Player>,
    display_name: String,
}

impl Workspace {
    pub fn new(player1: Entity<Player>, player2: Entity<Player>, cx: &mut Context<Self>) -> Self {
        // Subscribe to name transfer events from player1
        cx.subscribe(&player1, Self::on_name_transfer).detach();

        Self {
            player1,
            player2,
            display_name: String::new(),
        }
    }

    fn on_name_transfer(
        &mut self,
        _player1: Entity<Player>,
        event: &NameTransferEvent,
        cx: &mut Context<Self>,
    ) {
        // Update player2's name with the transferred name
        self.player2.update(cx, |player2, _| {
            player2.set_name(event.name.clone());
        });
        cx.notify();
    }

    fn set_name(&mut self, name: String, cx: &mut Context<Self>) {
        // Set the display name and save to player1
        self.display_name = name.clone();
        self.player1.update(cx, |player1, _| {
            player1.set_name(name);
        });
        cx.notify();
    }

    fn transfer_name(&mut self, cx: &mut Context<Self>) {
        // Get player1's name and emit transfer event
        let name = self.player1.read(cx).get_name().to_string();
        self.player1.update(cx, |_player1, cx| {
            cx.emit(NameTransferEvent { name });
        });
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let player1_name = self.player1.read(cx).get_name().to_string();
        let player2_name = self.player2.read(cx).get_name().to_string();

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .size_full()
            .gap_4()
            .p_8()
            .bg(gpui::rgb(0x1a1a1a))
            .child(
                div()
                    .text_2xl()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(gpui::rgb(0xffffff))
                    .child("Player Name Transfer"),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_8()
                    .w_full()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .flex_1()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(gpui::rgb(0xffffff))
                                    .child("Player 1:"),
                            )
                            .child(
                                div()
                                    .p_4()
                                    .bg(gpui::rgb(0x2a2a2a))
                                    .border_1()
                                    .border_color(gpui::rgb(0x444444))
                                    .rounded_md()
                                    .text_color(gpui::rgb(0xffffff))
                                    .child(player1_name),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .flex_1()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(gpui::rgb(0xffffff))
                                    .child("Player 2:"),
                            )
                            .child(
                                div()
                                    .p_4()
                                    .bg(gpui::rgb(0x2a2a2a))
                                    .border_1()
                                    .border_color(gpui::rgb(0x444444))
                                    .rounded_md()
                                    .text_color(gpui::rgb(0xffffff))
                                    .child(player2_name),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .w_full()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(gpui::rgb(0xffffff))
                            .child("Set Player 1 Name:"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_2()
                            .child(
                                div()
                                    .p_4()
                                    .flex_1()
                                    .bg(gpui::rgb(0x2a2a2a))
                                    .border_1()
                                    .border_color(gpui::rgb(0x444444))
                                    .rounded_md()
                                    .text_color(gpui::rgb(0xffffff))
                                    .child(if self.display_name.is_empty() {
                                        "Select a name below".to_string()
                                    } else {
                                        self.display_name.clone()
                                    }),
                            )
                            .child(
                                div()
                                    .p_4()
                                    .bg(gpui::rgb(0x007acc))
                                    .text_color(gpui::rgb(0xffffff))
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .rounded_md()
                                    .child("Set 'Alice'")
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _event, _window, cx| {
                                            this.set_name("Alice".to_string(), cx);
                                        }),
                                    ),
                            )
                            .child(
                                div()
                                    .p_4()
                                    .bg(gpui::rgb(0x007acc))
                                    .text_color(gpui::rgb(0xffffff))
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .rounded_md()
                                    .child("Set 'Bob'")
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _event, _window, cx| {
                                            this.set_name("Bob".to_string(), cx);
                                        }),
                                    ),
                            )
                            .child(
                                div()
                                    .p_4()
                                    .bg(gpui::rgb(0x007acc))
                                    .text_color(gpui::rgb(0xffffff))
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .rounded_md()
                                    .child("Set 'Charlie'")
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _event, _window, cx| {
                                            this.set_name("Charlie".to_string(), cx);
                                        }),
                                    ),
                            ),
                    ),
            )
            .child(
                div()
                    .p_4()
                    .px_8()
                    .bg(gpui::rgb(0x28a745))
                    .text_color(gpui::rgb(0xffffff))
                    .font_weight(gpui::FontWeight::BOLD)
                    .rounded_md()
                    .child("Transfer to Player 2")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _event, _window, cx| {
                            this.transfer_name(cx);
                        }),
                    ),
            )
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    Application::new().run(|cx: &mut App| {
        cx.open_window(
            WindowOptions {
                ..Default::default()
            },
            |_window, cx| {
                // Create player entities
                let player1 = cx.new(|_| Player::new("".to_string()));
                let player2 = cx.new(|_| Player::new("".to_string()));

                // Create workspace with players
                cx.new(|cx| Workspace::new(player1, player2, cx))
            },
        )
        .unwrap();
    });

    Ok(())
}
