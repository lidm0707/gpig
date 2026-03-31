# Workspace Module Documentation

The workspace module is the core of Zed's UI layer, managing everything project-related within a window. Each workspace window contains one or more `Workspace` instances (managed by `MultiWorkspace`), each handling projects, panes, docks, and user interactions.

## Overview

### What is a Workspace?

A `Workspace` represents a window's project context, containing:
- **Panels**: Splitable containers for items (files, buffers, etc.)
- **Docks**: Left, right, and bottom panels for auxiliary features
- **Status Bar**: Bottom bar showing workspace status
- **Modal Layer**: Overlay for dialogs and prompts
- **Toast Layer**: Notification system

```rust
/// Collects everything project-related for a certain window opened.
/// In some way, is a counterpart of a window, as the [`WindowHandle`] could be downcast into `Workspace`.
///
/// A `Workspace` usually consists of 1 or more projects, a central pane group, 3 docks and a status bar.
/// The `Workspace` owns everybody's state and serves as a default, "global context",
/// that can be used to register a global action to be triggered from any place in the window.
pub struct Workspace {
    pub(crate) weak_self: WeakEntity<Self>,
    pub center: PaneGroup,
    pub left_dock: Entity<Dock>,
    pub bottom_dock: Entity<Dock>,
    pub right_dock: Entity<Dock>,
    pub panes: Vec<Entity<Pane>>,
    pub active_pane: Entity<Pane>,
    pub status_bar: Entity<StatusBar>,
    pub modal_layer: Entity<ModalLayer>,
    pub toast_layer: Entity<ToastLayer>,
    pub project: Entity<Project>,
    // ... additional fields
}
```

## Core Components

### Pane Management

Panels are splitable containers that hold items (files, buffers, etc.). They support:
- **Splitting**: Create new panels by splitting existing ones
- **Navigation**: Navigate back/forward through item history
- **Activation**: Track which item is active and recently accessed
- **Zooming**: Expand to fill available workspace

#### Splitting Panes

```rust
impl Workspace {
    pub fn split_pane(
        &mut self,
        pane_to_split: Entity<Pane>,
        split_direction: SplitDirection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<Pane> {
        let new_pane = self.add_pane(window, cx);
        self.center.split(&pane_to_split, &new_pane, split_direction, cx);
        cx.notify();
        new_pane
    }
}
```

#### Split with Clone

Creates a new pane and clones the active item into it:

```rust
impl Workspace {
    pub fn split_and_clone(
        &mut self,
        pane: Entity<Pane>,
        direction: SplitDirection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Task<Option<Entity<Pane>>> {
        let Some(item) = pane.read(cx).active_item() else {
            return Task::ready(None);
        };
        if !item.can_split(cx) {
            return Task::ready(None);
        }
        let task = item.clone_on_split(self.database_id(), window, cx);
        cx.spawn_in(window, async move |this, cx| {
            let Some(clone) = task.await else {
                return None;
            };
            let new_pane = this.add_pane(window, cx);
            let nav_history = pane.read(cx).fork_nav_history();
            new_pane.update(cx, |new_pane, cx| {
                new_pane.set_nav_history(nav_history, cx);
                new_pane.add_item(clone, true, true, None, window, cx)
            });
            this.center.split(&pane, &new_pane, direction, cx);
            cx.notify();
            new_pane
        })
    }
}
```

### Dock Panels

Three dock positions contain panels for auxiliary features:

```rust
pub enum DockPosition {
    Left,
    Right,
    Bottom,
}
```

Each dock can:
- **Toggle**: Open/close with keyboard shortcuts
- **Resize**: Adjust panel sizes
- **Focus**: Transfer focus to/from center panes
- **Panel Management**: Activate, move, close panels

#### Panel Sizing

Panels support two sizing modes:

1. **Fixed Size**: Specific pixel width/height
2. **Flexible Size**: Proportional sharing of available space

```rust
pub struct PanelSizeState {
    pub size: Option<Pixels>,      // Fixed size in pixels
    pub flex: Option<f32>,        // Proportional ratio
}
```

Flexible panels share workspace width/height proportionally:

```rust
fn calculate_flexible_size(
    workspace_width: Pixels,
    dock_flex: f32,
    opposite_dock_flex: Option<f32>,
) -> Pixels {
    let available = if let Some(opposite_flex) = opposite_dock_flex {
        let opposite_width = opposite_dock_fixed_size;
        (workspace_width - opposite_width).max(RESIZE_HANDLE_SIZE)
    } else {
        workspace_width
    };
    
    let total_flex = dock_flex + 1.0 + opposite_dock_flex.unwrap_or(0.0);
    (dock_flex / total_flex * available).max(RESIZE_HANDLE_SIZE)
}
```

#### Toggle All Docks

```rust
impl Workspace {
    pub fn toggle_all_docks(
        &mut self,
        _: &ToggleAllDocks,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let open_dock_positions = self.get_open_dock_positions(cx);

        if !open_dock_positions.is_empty() {
            // Close all and remember positions
            self.close_all_docks(window, cx);
        } else if !self.last_open_dock_positions.is_empty() {
            // Restore last configuration
            self.restore_last_open_docks(window, cx);
        }
    }
}
```

### Item Management

Items are the core content (files, buffers, search results, etc.) that live in panes.

#### Item Lifecycle

```rust
pub trait ItemHandle: Send + Sync {
    fn item_id(&self) -> EntityId;
    fn project_path(&self, cx: &App) -> Option<ProjectPath>;
    fn is_dirty(&self, cx: &App) -> bool;
    fn can_split(&self, cx: &App) -> bool;
    // ... more methods
}
```

Items flow through these stages:

1. **Opening**: `open_path()` or `open_abs_path()`
2. **Adding to Pane**: `pane.add_item()`
3. **Activation**: `pane.activate_item()`
4. **Saving**: `Pane::save_item()`
5. **Closing**: `pane.close_item()`

#### Project Items

Items associated with project paths are tracked differently:

```rust
pub trait ProjectItem {
    type Item: ?Sized;
    
    fn try_open(
        project: &Entity<Project>,
        path: &ProjectPath,
        cx: &mut App,
    ) -> Option<Task<Result<Entity<Self>>>>;
    
    fn entry_id(&self, cx: &App) -> Option<ProjectEntryId>;
    fn project_path(&self, cx: &App) -> Option<ProjectPath>;
    // ... more methods
}
```

Project items are registered via:

```rust
pub fn register_project_item<I: ProjectItem>(cx: &mut App) {
    cx.default_global::<ProjectItemRegistry>().register::<I>();
}
```

## Key Features

### Serialization & Persistence

The workspace automatically saves and restores its state.

#### What Gets Serialized

```rust
struct SerializedWorkspace {
    id: WorkspaceId,
    location: SerializedWorkspaceLocation,
    paths: PathList,
    center_group: SerializedPaneGroup,
    window_bounds: Option<SerializedWindowBounds>,
    display: Option<Uuid>,
    docks: DockStructure,
    centered_layout: bool,
    session_id: Option<String>,
    breakpoints: Vec<proto::SourceBreakpoint>,
    window_id: Option<u64>,
    user_toolchains: HashMap<Arc<str>, Vec<Toolchain>>,
}
```

#### Throttled Serialization

Serialization is throttled to 200ms to avoid excessive database writes:

```rust
pub const SERIALIZATION_THROTTLE_TIME: Duration = Duration::from_millis(200);

impl Workspace {
    fn serialize_workspace(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self._schedule_serialize_workspace.is_none() {
            self._schedule_serialize_workspace = Some(cx.spawn_in(window, async move |this, cx| {
                cx.background_executor().timer(SERIALIZATION_THROTTLE_TIME).await;
                this.update_in(cx, |this, window, cx| {
                    this._serialize_workspace_task = Some(this.serialize_workspace_internal(window, cx));
                    this._schedule_serialize_workspace.take();
                }).log_err();
            }));
        }
    }
}
```

#### Flush for Exit

Before quitting, the workspace bypasses throttling to ensure state is saved:

```rust
impl Workspace {
    pub fn flush_serialization(&mut self, window: &mut Window, cx: &mut App) -> Task<()> {
        self._schedule_serialize_workspace.take();
        self._serialize_workspace_task.take();
        
        let bounds_task = self.save_window_bounds(window, cx);
        let serialize_task = self.serialize_workspace_internal(window, cx);
        cx.spawn(async move |_| {
            bounds_task.await;
            serialize_task.await;
        })
    }
}
```

### Collaboration & Following

The workspace supports following remote collaborators.

#### Follower State

```rust
pub struct FollowerState {
    center_pane: Entity<Pane>,
    dock_pane: Option<Entity<Pane>>,
    active_view_id: Option<ViewId>,
    items_by_leader_view_id: HashMap<ViewId, FollowerView>,
}

struct FollowerView {
    view: Box<dyn FollowableItemHandle>,
    location: Option<proto::PanelId>,
}
```

#### Following Workflow

```rust
impl Workspace {
    pub fn follow(
        &mut self,
        leader_id: impl Into<CollaboratorId>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let leader_id = leader_id.into();
        let pane = self.active_pane().clone();

        // Clean up any existing following state
        self.unfollow(leader_id, window, cx);
        self.unfollow_in_pane(&pane, window, cx);

        // Create new follower state
        self.follower_states.insert(
            leader_id,
            FollowerState {
                center_pane: pane.clone(),
                dock_pane: None,
                active_view_id: None,
                items_by_leader_view_id: Default::default(),
            },
        );
    }
}
```

#### View Synchronization

When the leader changes their active view, followers receive updates:

```rust
impl Workspace {
    async fn process_leader_update(
        this: &WeakEntity<Self>,
        leader_id: PeerId,
        update: proto::UpdateFollowers,
        cx: &mut AsyncWindowContext,
    ) -> Result<()> {
        match update.variant.context("invalid update")? {
            proto::update_followers::Variant::CreateView(view) => {
                let view_id = ViewId::from_proto(view.id.clone().context("invalid view id")?)?;
                if let Some(should_add_view) = this.update(cx, |this, _| {
                    let state = this.follower_states.get_mut(&leader_id.into())?;
                    Ok(!state.items_by_leader_view_id.contains_key(&view_id))
                })?? && should_add_view {
                    Self::add_view_from_leader(this.clone(), leader_id, &view, cx).await?
                }
            }
            proto::update_followers::Variant::UpdateActiveView(update) => {
                // Update active view and navigate to it
            }
            proto::update_followers::Variant::UpdateView(update) => {
                // Apply updates to existing view
            }
        }
        Ok(())
    }
}
```

### Navigation History

Panels maintain navigation history for back/forward movement.

#### Navigation History Structure

```rust
struct NavHistory {
    entries: Vec<NavigationEntry>,
    index: usize,  // Current position in history
}

struct NavigationEntry {
    item: WeakEntity<dyn ItemHandle>,
    timestamp: usize,
    data: Option<Box<dyn std::any::Any>>,
}
```

#### Back/Forward Navigation

```rust
impl Workspace {
    pub fn go_back(
        &mut self,
        pane: WeakEntity<Pane>,
        window: &mut Window,
        cx: &mut Context<Workspace>,
    ) -> Task<Result<()>> {
        self.navigate_history(pane, NavigationMode::GoingBack, window, cx)
    }
    
    fn navigate_history_impl(
        &mut self,
        pane: WeakEntity<Pane>,
        mode: NavigationMode,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Task<Result<()>> {
        // Pop entry from history
        // If item still exists, activate it
        // If item doesn't exist, reload it from saved path
    }
}
```

#### Deduplication

Navigation history deduplicates entries for the same item:

```rust
/// When navigating back and forth between items (e.g., A -> B -> A -> B -> A -> B -> C),
/// navigation history deduplicates by keeping only the most recent visit to each item,
/// resulting in [A, B, C] instead of [A, B, A, B, A, B, C].
```

### Zooming

Zooming expands a pane or panel to fill the available workspace space.

#### Zoom States

```rust
impl Workspace {
    pub fn zoomed_item(&self) -> Option<&AnyWeakView> {
        self.zoomed.as_ref()
    }
}
```

- **None**: No zooming active
- **Some(view)**: A pane or panel is zoomed and filling workspace

#### Pane Zoom

```rust
impl Workspace {
    fn handle_pane_focused(
        &mut self,
        pane: Entity<Pane>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.active_pane != pane {
            self.set_active_pane(&pane, window, cx);
        }
        
        if pane.read(cx).is_zoomed() {
            self.zoomed = Some(pane.downgrade().into());
        } else {
            self.zoomed = None;
        }
        self.zoomed_position = None;
        cx.emit(Event::ZoomChanged);
    }
}
```

#### Panel Zoom

```rust
impl Workspace {
    fn render_dock(
        &self,
        position: DockPosition,
        dock: &Entity<Dock>,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<Div> {
        // Hide dock if another dock/pane is zoomed
        if self.zoomed_position == Some(position) {
            return None;
        }
        // ... render dock
    }
}
```

When a panel is zoomed:
- Other docks are hidden
- Center pane is hidden (or the zoomed panel overlays it)
- Focus remains on the zoomed panel

### Window Management

Multiple windows can exist with different workspaces.

#### Window Lifecycle

```rust
impl Workspace {
    pub fn activate_next_window(&mut self, cx: &mut Context<Self>) {
        let Some(current_window_id) = cx.active_window().map(|a| a.window_id()) else {
            return;
        };
        let windows = cx.windows();
        let next_window = SystemWindowTabController::get_next_tab_group_window(cx, current_window_id);
        
        if let Some(window) = next_window {
            window.update(cx, |_, window, _| window.activate_window()).ok();
        }
    }
}
```

#### Window Bounds Persistence

Window position and size are persisted per workspace:

```rust
impl Workspace {
    fn save_window_bounds(&self, window: &mut Window, cx: &mut App) -> Task<()> {
        let Some(display) = window.display(cx) else {
            return Task::ready(());
        };
        let window_bounds = window.inner_window_bounds();
        let database_id = self.database_id;
        let db = WorkspaceDb::global(cx);
        
        cx.background_executor().spawn(async move {
            if let Some(database_id) = database_id {
                db.set_window_open_status(
                    database_id,
                    SerializedWindowBounds(window_bounds),
                    display_uuid,
                ).await.log_err();
            }
        })
    }
}
```

### Actions System

Actions are keyboard shortcuts and commands that can be dispatched.

#### Defining Actions

```rust
actions!(workspace, [
    /// Activates the next pane in the workspace.
    ActivateNextPane,
    /// Activates the previous pane in the workspace.
    ActivatePreviousPane,
    /// Toggles the left dock.
    ToggleLeftDock,
    /// Toggles the right dock.
    ToggleRightDock,
    /// Saves the current file with the specified options.
    Save,
    // ... many more actions
]);
```

#### Registering Action Handlers

```rust
impl Workspace {
    pub fn register_action<A: Action>(
        &mut self,
        callback: impl Fn(&mut Self, &A, &mut Window, &mut Context<Self>) + 'static,
    ) -> &mut Self {
        let callback = Arc::new(callback);
        self.workspace_actions.push(Box::new(move |div, _, _, cx| {
            let callback = callback.clone();
            div.on_action(cx.listener(move |workspace, event, window, cx| {
                (callback)(workspace, event, window, cx)
            }))
        }));
        self
    }
}
```

#### Dispatching Actions

```rust
impl Workspace {
    fn save_all(&mut self, action: &SaveAll, window: &mut Window, cx: &mut Context<Self>) {
        self.save_all_internal(
            action.save_intent.unwrap_or(SaveIntent::SaveAll),
            window,
            cx,
        ).detach_and_log_err(cx);
    }
}
```

## Data Structures

### Workspace State

```rust
pub struct Workspace {
    // Core components
    pub(crate) weak_self: WeakEntity<Self>,
    pub center: PaneGroup,
    pub left_dock: Entity<Dock>,
    pub bottom_dock: Entity<Dock>,
    pub right_dock: Entity<Dock>,
    pub panes: Vec<Entity<Pane>>,
    pub active_pane: Entity<Pane>,
    pub status_bar: Entity<StatusBar>,
    pub modal_layer: Entity<ModalLayer>,
    pub toast_layer: Entity<ToastLayer>,
    
    // Project
    pub project: Entity<Project>,
    
    // Collaboration
    pub follower_states: HashMap<CollaboratorId, FollowerState>,
    pub last_leaders_by_pane: HashMap<WeakEntity<Pane>, CollaboratorId>,
    
    // State
    pub database_id: Option<WorkspaceId>,
    pub session_id: Option<String>,
    pub centered_layout: bool,
    pub window_edited: bool,
    
    // Items
    pub panes_by_item: HashMap<EntityId, WeakEntity<Pane>>,
    pub dirty_items: HashMap<EntityId, Subscription>,
    
    // ... more fields
}
```

### Pane Group Structure

```rust
enum Member {
    Axis(PaneAxis),
    Pane(Entity<Pane>),
}

struct PaneAxis {
    axis: Axis,
    members: Vec<Member>,
    flexes: Arc<Mutex<Vec<f32>>>,
}
```

### Dock State

```rust
pub struct DockStructure {
    pub left: DockData,
    pub right: DockData,
    pub bottom: DockData,
}

struct DockData {
    pub visible: bool,
    pub active_panel: Option<String>,
    pub zoom: bool,
}
```

## Key Traits

### ItemHandle

```rust
pub trait ItemHandle: Send + Sync {
    fn item_id(&self) -> EntityId;
    fn project_path(&self, cx: &App) -> Option<ProjectPath>;
    fn is_dirty(&self, cx: &App) -> bool;
    fn can_split(&self, cx: &App) -> bool;
    fn is_singleton(&self, cx: &App) -> bool;
    fn tab_content_text(&self, detail: usize, cx: &App) -> SharedString;
    fn tab_content_icon(&self, cx: &App) -> Option<IconName>;
    fn clone_on_split(
        &self,
        workspace_id: Option<WorkspaceId>,
        window: &mut Window,
        cx: &mut App,
    ) -> Task<Option<Box<dyn ItemHandle>>>;
    // ... more methods
}
```

### FollowableItem

```rust
pub trait FollowableItem: ItemHandle {
    fn remote_id(&self, client: &Client, window: &Window, cx: &App) -> Option<proto::ViewId>;
    fn to_state_proto(&self, window: &Window, cx: &App) -> Option<proto::view::Variant>;
    fn apply_update_proto(
        &self,
        project: &Entity<Project>,
        update: proto::update_view::Variant,
        window: &mut Window,
        cx: &mut App,
    ) -> Task<Result<()>>;
}
```

### Panel

```rust
pub trait Panel: Render + Focusable + Sized {
    fn persistent_name(&self) -> &'static str;
    fn position(&self, window: &Window, cx: &App) -> DockPosition;
    fn set_position(&mut self, position: DockPosition, window: &Window, cx: &mut App);
    fn has_flexible_size(&self, window: &Window, cx: &App) -> bool;
    fn default_size(&self, window: &Window, cx: &App) -> Pixels;
}
```

## Usage Examples

### Opening a File

```rust
workspace.update_in(cx, |workspace, window, cx| {
    let project_path = ProjectPath {
        worktree_id,
        path: rel_path("src/main.rs"),
    };
    workspace.open_path(project_path, None, true, window, cx)
        .detach();
});
```

### Splitting a Pane

```rust
workspace.update_in(cx, |workspace, window, cx| {
    let active_pane = workspace.active_pane().clone();
    workspace.split_pane(
        active_pane,
        SplitDirection::Right,
        window,
        cx,
    );
});
```

### Toggling a Dock

```rust
workspace.update_in(cx, |workspace, window, cx| {
    workspace.toggle_dock(DockPosition::Right, window, cx);
});
```

### Saving All Files

```rust
workspace.dispatch_action(&SaveAll::default());
```

## Persistence

### Workspace Database

The workspace uses `WorkspaceDb` for persistence:

```rust
pub struct WorkspaceDb {
    // Store workspaces, their layouts, and items
}

impl WorkspaceDb {
    pub fn save_workspace(&self, workspace: SerializedWorkspace) {
        // Save to database
    }
    
    pub fn workspace_for_roots(&self, paths: &[PathBuf]) -> Option<SerializedWorkspace> {
        // Find matching workspace
    }
}
```

### KVP Store

Key-value pairs for settings and panel states:

```rust
let kvp = db::kvp::KeyValueStore::global(cx);
let scope = kvp.scoped(dock::PANEL_SIZE_STATE_KEY);
scope.write(
    format!("{workspace_id}:{panel_key}"),
    serde_json::to_string(&size_state)?
).await;
```

## Testing

The module has extensive test coverage including:

- Tab disambiguation
- Tracking active path
- Window closing with prompts
- Autosave behavior
- Pane navigation
- Navigation history deduplication
- Toggle docks and panels
- Panel sizing (fixed and flexible)
- Moving items between panes
- Multi-buffer item handling
- Close on disk deletion
- And many more...

See the `tests` module in `workspace.rs` for complete test examples.