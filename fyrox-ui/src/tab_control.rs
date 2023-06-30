//! The Tab Control handles the visibility of several tabs, only showing a single tab that the user has selected via the
//! tab header buttons. See docs for [`TabControl`] widget for more info and usage examples.

#![warn(missing_docs)]

use crate::{
    border::BorderBuilder,
    brush::Brush,
    button::{ButtonBuilder, ButtonMessage},
    core::{color::Color, pool::Handle},
    define_constructor,
    grid::{Column, GridBuilder, Row},
    message::{MessageDirection, UiMessage},
    stack_panel::StackPanelBuilder,
    utils::make_cross,
    widget::{Widget, WidgetBuilder, WidgetMessage},
    BuildContext, Control, NodeHandleMapping, Orientation, UiNode, UserInterface,
};
use std::{
    any::{Any, TypeId},
    ops::{Deref, DerefMut},
};

/// A set of messages for [`TabControl`] widget.
#[derive(Debug, Clone, PartialEq)]
pub enum TabControlMessage {
    /// Used to change the active tab of a [`TabControl`] widget (with [`MessageDirection::ToWidget`]) or to fetch if the active
    /// tab has changed (with [`MessageDirection::FromWidget`]).
    ActiveTab(Option<usize>),
    /// Emitted by a tab, that needs to be closed (and removed). Does **not** remove the tab, its main usage is to catch the moment
    /// when the tab wants to be closed. To remove the tab use [`TabControlMessage::RemoveTab`] message.
    CloseTab(usize),
    /// Used to remove a particular tab.
    RemoveTab(usize),
    /// Adds a new tab using its definition.
    AddTab(TabDefinition),
}

impl TabControlMessage {
    define_constructor!(
        /// Creates [`TabControlMessage::ActiveTab`] message.
        TabControlMessage:ActiveTab => fn active_tab(Option<usize>), layout: false
    );
    define_constructor!(
        /// Creates [`TabControlMessage::CloseTab`] message.
        TabControlMessage:CloseTab => fn close_tab(usize), layout: false
    );
    define_constructor!(
        /// Creates [`TabControlMessage::RemoveTab`] message.
        TabControlMessage:RemoveTab => fn remove_tab(usize), layout: false
    );
    define_constructor!(
        /// Creates [`TabControlMessage::AddTab`] message.
        TabControlMessage:AddTab => fn add_tab(TabDefinition), layout: false
    );
}

/// Tab of the [`TabControl`] widget. It stores important tab data, that is widely used at runtime.
#[derive(Clone, PartialEq, Eq)]
pub struct Tab {
    /// A handle of the header button, that is used to switch tabs.
    pub header_button: Handle<UiNode>,
    /// Tab's content.
    pub content: Handle<UiNode>,
    /// A handle of a button, that is used to close the tab.
    pub close_button: Handle<UiNode>,
    /// A handle to a container widget, that holds the header.
    pub header_container: Handle<UiNode>,
}

/// The Tab Control handles the visibility of several tabs, only showing a single tab that the user has selected via the
/// tab header buttons. Each tab is defined via a Tab Definition struct which takes two widgets, one representing the tab
/// header and the other representing the tab's contents.
///
/// The following example makes a 2 tab, Tab Control containing some simple text widgets:
///
/// ```rust,no_run
/// # use fyrox_ui::{
/// #     BuildContext,
/// #     widget::WidgetBuilder,
/// #     text::TextBuilder,
/// #     tab_control::{TabControlBuilder, TabDefinition},
/// # };
/// fn create_tab_control(ctx: &mut BuildContext) {
///
///     TabControlBuilder::new(WidgetBuilder::new())
///         .with_tab(
///             TabDefinition{
///                 header: TextBuilder::new(WidgetBuilder::new())
///                             .with_text("First")
///                             .build(ctx),
///                             
///                 content: TextBuilder::new(WidgetBuilder::new())
///                             .with_text("First tab's contents!")
///                             .build(ctx),
///                 can_be_closed: true
///             }
///         )
///         .with_tab(
///             TabDefinition{
///                 header: TextBuilder::new(WidgetBuilder::new())
///                             .with_text("Second")
///                             .build(ctx),
///                             
///                 content: TextBuilder::new(WidgetBuilder::new())
///                             .with_text("Second tab's contents!")
///                             .build(ctx),
///                 can_be_closed: true
///             }
///         )
///         .build(ctx);
/// }
/// ```
///
/// As usual, we create the widget via the builder TabControlBuilder. Tabs are added via the [`TabControlBuilder::with_tab`]
/// function in the order you want them to appear, passing each call to the function a directly constructed [`TabDefinition`]
/// struct. Tab headers will appear from left to right at the top with tab contents shown directly below the tabs. As usual, if no
/// constraints are given to the base [`WidgetBuilder`] of the [`TabControlBuilder`], then the tab content area will resize to fit
/// whatever is in the current tab.
///
/// Each tab's content is made up of one widget, so to be useful you will want to use one of the container widgets to help
/// arrange additional widgets within the tab.
///
/// ## Tab Header Styling
///
/// Notice that you can put any widget into the tab header, so if you want images to denote each tab you can add an Image
/// widget to each header, and if you want an image *and* some text you can insert a stack panel with an image on top and
/// text below it.
///
/// You will also likely want to style whatever widgets you add. As can be seen when running the code example above, the
/// tab headers are scrunched when there are no margins provided to your text widgets. Simply add something like the below
/// code example and you will get a decent look:
///
/// ```rust,no_run
/// # use fyrox_ui::{
/// #     BuildContext,
/// #     widget::WidgetBuilder,
/// #     text::TextBuilder,
/// #     Thickness,
/// #     tab_control::{TabDefinition},
/// # };
/// # fn build(ctx: &mut BuildContext) {
/// # TabDefinition{
/// header: TextBuilder::new(
///             WidgetBuilder::new()
///                 .with_margin(Thickness::uniform(4.0))
///         )
///             .with_text("First")
///             .build(ctx),
/// # content: Default::default(),
/// # can_be_closed: true
/// # };
/// # }
///
/// ```
#[derive(Clone)]
pub struct TabControl {
    /// Base widget of the tab control.
    pub widget: Widget,
    /// A set of tabs used by the tab control.
    pub tabs: Vec<Tab>,
    /// Active tab of the tab control.
    pub active_tab: Option<usize>,
    /// A handle of a widget, that holds content of every tab.
    pub content_container: Handle<UiNode>,
    /// A handle of a widget, that holds headers of every tab.
    pub headers_container: Handle<UiNode>,
}

crate::define_widget_deref!(TabControl);

impl Control for TabControl {
    fn query_component(&self, type_id: TypeId) -> Option<&dyn Any> {
        if type_id == TypeId::of::<Self>() {
            Some(self)
        } else {
            None
        }
    }

    fn resolve(&mut self, node_map: &NodeHandleMapping) {
        for tab in self.tabs.iter_mut() {
            node_map.resolve(&mut tab.header_button);
            node_map.resolve(&mut tab.content);
        }
    }

    fn handle_routed_message(&mut self, ui: &mut UserInterface, message: &mut UiMessage) {
        self.widget.handle_routed_message(ui, message);

        if let Some(ButtonMessage::Click) = message.data() {
            for (tab_index, tab) in self.tabs.iter().enumerate() {
                if message.destination() == tab.header_button && tab.header_button.is_some() {
                    ui.send_message(TabControlMessage::active_tab(
                        self.handle,
                        MessageDirection::ToWidget,
                        Some(tab_index),
                    ));
                    break;
                } else if message.destination() == tab.close_button {
                    ui.send_message(TabControlMessage::close_tab(
                        self.handle,
                        MessageDirection::FromWidget,
                        tab_index,
                    ));
                }
            }
        } else if let Some(msg) = message.data::<TabControlMessage>() {
            if message.destination() == self.handle()
                && message.direction() == MessageDirection::ToWidget
            {
                match msg {
                    TabControlMessage::ActiveTab(active_tab) => {
                        if self.active_tab != *active_tab {
                            for (existing_tab_index, tab) in self.tabs.iter().enumerate() {
                                ui.send_message(WidgetMessage::visibility(
                                    tab.content,
                                    MessageDirection::ToWidget,
                                    active_tab.map_or(false, |active_tab_index| {
                                        existing_tab_index == active_tab_index
                                    }),
                                ));
                            }
                            self.active_tab = *active_tab;
                            // Notify potential listeners, that the active tab has changed.
                            ui.send_message(message.reverse());
                        }
                    }
                    TabControlMessage::CloseTab(_) => {
                        // Nothing to do.
                    }
                    TabControlMessage::RemoveTab(index) => {
                        if let Some(tab) = self.tabs.get(*index) {
                            ui.send_message(WidgetMessage::remove(
                                tab.header_container,
                                MessageDirection::ToWidget,
                            ));
                            ui.send_message(WidgetMessage::remove(
                                tab.content,
                                MessageDirection::ToWidget,
                            ));

                            self.tabs.remove(*index);

                            ui.send_message(message.reverse());
                        }
                    }
                    TabControlMessage::AddTab(definition) => {
                        let header = Header::build(definition, &mut ui.build_ctx());

                        ui.send_message(WidgetMessage::link(
                            header.container,
                            MessageDirection::ToWidget,
                            self.headers_container,
                        ));

                        ui.send_message(WidgetMessage::link(
                            definition.content,
                            MessageDirection::ToWidget,
                            self.content_container,
                        ));

                        ui.send_message(message.reverse());
                    }
                }
            }
        }
    }
}

/// Tab control builder is used to create [`TabControl`] widget instances and add them to the user interface.
pub struct TabControlBuilder {
    widget_builder: WidgetBuilder,
    tabs: Vec<TabDefinition>,
}

/// Tab definition is used to describe content of each tab for the [`TabControlBuilder`] builder.
#[derive(Debug, Clone, PartialEq)]
pub struct TabDefinition {
    /// Content of the tab-switching (header) button.
    pub header: Handle<UiNode>,
    /// Content of the tab.
    pub content: Handle<UiNode>,
    /// A flag, that defines whether the tab can be closed or not.
    pub can_be_closed: bool,
}

struct Header {
    container: Handle<UiNode>,
    button: Handle<UiNode>,
    close_button: Handle<UiNode>,
}

impl Header {
    fn build(tab_definition: &TabDefinition, ctx: &mut BuildContext) -> Self {
        let button;
        let close_button;
        let grid = GridBuilder::new(
            WidgetBuilder::new()
                .with_child({
                    button = ButtonBuilder::new(WidgetBuilder::new().on_row(0).on_column(0))
                        .with_content(tab_definition.header)
                        .build(ctx);
                    button
                })
                .with_child({
                    close_button = if tab_definition.can_be_closed {
                        ButtonBuilder::new(
                            WidgetBuilder::new().on_row(0).on_column(1).with_width(16.0),
                        )
                        .with_content(make_cross(ctx, 10.0, 2.0))
                        .build(ctx)
                    } else {
                        Handle::NONE
                    };
                    close_button
                }),
        )
        .add_row(Row::auto())
        .add_column(Column::auto())
        .add_column(Column::auto())
        .build(ctx);

        Header {
            container: grid,
            button,
            close_button,
        }
    }
}

impl TabControlBuilder {
    /// Creates new tab control builder.
    pub fn new(widget_builder: WidgetBuilder) -> Self {
        Self {
            widget_builder,
            tabs: Default::default(),
        }
    }

    /// Adds a new tab to the builder.
    pub fn with_tab(mut self, tab: TabDefinition) -> Self {
        self.tabs.push(tab);
        self
    }

    /// Finishes [`TabControl`] building and adds it to the user interface and returns its handle.
    pub fn build(self, ctx: &mut BuildContext) -> Handle<UiNode> {
        let mut content = Vec::new();
        let tab_count = self.tabs.len();
        for (i, tab) in self.tabs.iter().enumerate() {
            // Hide everything but first tab content.
            if i > 0 {
                if let Some(content) = ctx.try_get_node_mut(tab.content) {
                    content.set_visibility(false);
                }
            }
            content.push(tab.content);
        }

        let tab_headers = self
            .tabs
            .iter()
            .map(|tab_definition| Header::build(tab_definition, ctx))
            .collect::<Vec<_>>();

        let headers_container = StackPanelBuilder::new(
            WidgetBuilder::new()
                .with_children(tab_headers.iter().map(|h| h.container))
                .on_row(0),
        )
        .with_orientation(Orientation::Horizontal)
        .build(ctx);

        let content_container = GridBuilder::new(
            WidgetBuilder::new()
                .with_children(content.iter().cloned())
                .on_row(1),
        )
        .add_row(Row::stretch())
        .add_column(Column::stretch())
        .build(ctx);

        let grid = GridBuilder::new(
            WidgetBuilder::new()
                .with_child(headers_container)
                .with_child(content_container),
        )
        .add_column(Column::auto())
        .add_row(Row::auto())
        .add_row(Row::auto())
        .build(ctx);

        let tc = TabControl {
            widget: self
                .widget_builder
                .with_child(
                    BorderBuilder::new(
                        WidgetBuilder::new()
                            .with_background(Brush::Solid(Color::from_rgba(0, 0, 0, 0)))
                            .with_child(grid),
                    )
                    .build(ctx),
                )
                .build(),
            active_tab: if tab_count == 0 { None } else { Some(0) },
            tabs: tab_headers
                .iter()
                .zip(content)
                .map(|(header, content)| Tab {
                    header_button: header.button,
                    content,
                    close_button: header.close_button,
                    header_container: header.container,
                })
                .collect(),
            content_container,
            headers_container,
        };

        ctx.add_node(UiNode::new(tc))
    }
}
