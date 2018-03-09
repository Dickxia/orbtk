extern crate orbfont;

use orbclient::{self, Mode, Renderer, WindowFlag};
use orbclient::color::Color;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::sync::Arc;

use super::{Content, Event, Point, Rect, Widget, Node};
use theme::Theme;
use traits::Resize;

pub use orbclient::Window as InnerWindow;

pub struct WindowRenderer<'a> {
    inner: &'a mut InnerWindow,
    font: &'a Option<orbfont::Font>,
}

impl<'a> WindowRenderer<'a> {
    pub fn new(inner: &'a mut InnerWindow, font: &'a Option<orbfont::Font>) -> WindowRenderer<'a> {
        WindowRenderer {
            inner: inner,
            font: font,
        }
    }
}

impl<'a> Renderer for WindowRenderer<'a> {
    fn width(&self) -> u32 {
        self.inner.width()
    }

    fn height(&self) -> u32 {
        self.inner.height()
    }

    fn data(&self) -> &[Color] {
        self.inner.data()
    }

    fn data_mut(&mut self) -> &mut [Color] {
        self.inner.data_mut()
    }

    fn sync(&mut self) -> bool {
        self.inner.sync()
    }

    fn mode(&self) -> &Cell<Mode> {
        &self.inner.mode()
    }

    fn char(&mut self, x: i32, y: i32, c: char, color: Color) {
        if let Some(ref font) = *self.font {
            let mut buf = [0; 4];
            font.render(&c.encode_utf8(&mut buf), 16.0)
                .draw(self.inner, x, y, color)
        } else {
            self.inner.char(x, y, c, color);
        }
    }
}

impl<'a> Drop for WindowRenderer<'a> {
    fn drop(&mut self) {
        self.inner.sync();
    }
}

pub struct Window {
    inner: RefCell<InnerWindow>,
    font: Option<orbfont::Font>,
    //    pub widgets: RefCell<Vec<Arc<Widget>>>,
    pub running: Cell<bool>,
    pub theme: Theme,
    resize_callback: RefCell<Option<Arc<Fn(&Window, u32, u32)>>>,
    _mouse_point: Point,
    _mouse_left: bool,
    _mouse_middle: bool,
    _mouse_right: bool,
    events: VecDeque<Event>,
    redraw: bool,
}

impl Resize for Window {
    fn emit_resize(&self, width: u32, height: u32) {
        if let Some(ref resize_callback) = *self.resize_callback.borrow() {
            resize_callback(self, width, height);
        }
    }

    fn on_resize<T: Fn(&Self, u32, u32) + 'static>(&self, func: T) -> &Self {
        *self.resize_callback.borrow_mut() = Some(Arc::new(func));
        self
    }
}

impl Window {
    pub fn new(rect: Rect, title: &str) -> Self {
        Window::new_flags(rect, title, &[])
    }

    pub fn new_flags(rect: Rect, title: &str, flags: &[WindowFlag]) -> Self {
        Window::from_inner(
            InnerWindow::new_flags(rect.x, rect.y, rect.width, rect.height, title, flags).unwrap(),
        )
    }

    pub fn from_inner(inner: InnerWindow) -> Self {
        let mut events = VecDeque::new();
        events.push_back(Event::Init);
        Window {
            inner: RefCell::new(inner),
            font: orbfont::Font::find(None, None, None).ok(),
            //            widgets: RefCell::new(Vec::new()),
            running: Cell::new(true),
            theme: Theme::new(),
            resize_callback: RefCell::new(None),
            _mouse_point: Point::new(0, 0),
            _mouse_left: false,
            _mouse_right: false,
            _mouse_middle: false,
            events: events,
            redraw: true,
            //            focus_manager: FocusManager::new(),
        }
    }

    pub fn into_inner(self) -> InnerWindow {
        self.inner.into_inner()
    }

    pub fn x(&self) -> i32 {
        let inner = self.inner.borrow();
        (*inner).x()
    }

    pub fn y(&self) -> i32 {
        let inner = self.inner.borrow();
        (*inner).y()
    }

    pub fn width(&self) -> u32 {
        let inner = self.inner.borrow();
        (*inner).width()
    }

    pub fn height(&self) -> u32 {
        let inner = self.inner.borrow();
        (*inner).height()
    }

    pub fn title(&self) -> String {
        let inner = self.inner.borrow();
        (*inner).title()
    }

    pub fn set_pos(&self, x: i32, y: i32) {
        let mut inner = self.inner.borrow_mut();
        (*inner).set_pos(x, y);
    }

    pub fn set_size(&self, width: u32, height: u32) {
        let mut inner = self.inner.borrow_mut();
        (*inner).set_size(width, height);
    }

    pub fn set_title(&self, title: &str) {
        let mut inner = self.inner.borrow_mut();
        (*inner).set_title(title);
    }

    pub fn close(&self) {
        self.running.set(false);
    }

    //    pub fn add<T: Widget>(&self, widget: &Arc<T>) -> usize {
    //        let mut widgets = self.widgets.borrow_mut();
    //        let id = widgets.len();
    //        widgets.push(widget.clone());
    //
    //        if id == 0 {
    //            self.focus_manager.request_focus(&widgets[id]);
    //        }
    //
    //        id
    //    }

    pub fn draw(&self) {
        let mut inner = self.inner.borrow_mut();
        inner.set(self.theme.color("background", &"window".into()));

        let mut _renderer = WindowRenderer::new(&mut *inner, &self.font);
        //        for widget in self.widgets.borrow().iter() {
        //            self.draw_widget(&mut renderer, self.focus_manager.focused(&widget), widget);
        //        }
    }

    //    fn draw_widget(&self, renderer: &mut Renderer, focused: bool, widget: &Arc<Widget>) {
    //        widget.update();
    //        widget.draw(renderer, focused, &self.theme);
    //
    //        for child in widget.children().borrow().iter() {
    //            self.draw_widget(renderer, self.focus_manager.focused(&child), child);
    //        }
    //    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub fn step(&mut self) {
        self.drain_orbital_events();
        self.drain_events();
    }

    pub fn drain_events(&mut self) {
        while let Some(event) = self.events.pop_front() {
            match event {
                Event::Resize { width, height } => {
                    self.emit_resize(width, height);
                }
                _ => (),
            }

            //            for widget in self.widgets.borrow().iter() {
            //                self.redraw = self.drain_event(event, self.redraw, widget);
            //            }
        }
    }

    //    fn drain_event(&self, event: Event, redraw: bool, widget: &Arc<Widget>) -> bool {
    //        let mut redraw = redraw;
    //        //let mut children_redraw = false;
    //
    //        if widget.event(event, self.focus_manager.focused(&widget), &mut redraw) {
    //            if !self.focus_manager.focused(&widget) {
    //                self.focus_manager.request_focus(&widget);
    //                redraw = true;
    //            }
    //        }
    //
    //        redraw
    //
    //        // for child in &*widget.children().borrow_mut() {
    //        //     children_redraw = self.drain_event(event, redraw, child);
    //        // }
    //
    //        // redraw || children_redraw
    //    }

    pub fn drain_orbital_events(&mut self) {
        for orbital_event in self.inner.borrow_mut().events() {
            match orbital_event.to_option() {
                
                orbclient::EventOption::Resize(resize_event) => {
                    self.redraw = true;
                    self.events.push_back(Event::Resize {
                        width: resize_event.width,
                        height: resize_event.height,
                    });
                }
                orbclient::EventOption::Quit(_quit_event) => {
                    self.running.set(false);
                }
                _ => (),
            };
        }
    }

    pub fn exec(&mut self) {
        'event: while self.running.get() {
            self.drain_events();
            self.draw_if_needed();
            self.drain_orbital_events();
        }
    }

    pub fn needs_redraw(&mut self) {
        self.redraw = true;
    }

    pub fn draw_if_needed(&mut self) {
        if self.redraw {
            self.draw();
            self.redraw = false;
        }
    }
}



fn build_tree(root: &Option<Arc<Node>>, widget: &Arc<Widget>) -> Option<Arc<Node>> {
    let node = {
        if let Some(ref root) = *root {
            Some(Node::new(widget, root))
        } else {
            Some(Node::new_root(widget))
        }
    };

    if let Some(ref root) = *root {
        if let Some(ref node) = node {
            root.children().borrow_mut().push(node.clone())
        }
    }

    match widget.build() {
        Content::Zero => return None,
        Content::Single(child) => {
            build_tree(&node, &child);
        }
        Content::Multi(children) => for child in children {
            build_tree(&node, &child);
        },
    }

    node
}

pub struct Application<'a> {
    rect: Rect,
    title: &'a str,
    font: Option<orbfont::Font>,
    theme: Option<Theme>,
    flags: Option<&'a [WindowFlag]>,
    root: Option<Arc<Widget>>,
    tree: Option<Arc<Node>>,
}

impl<'a> Application<'a> {
    pub fn new(rect: Rect, title: &'a str) -> Self {
        Application {
            rect: rect,
            title: title,
            font: orbfont::Font::find(None, None, None).ok(),
            theme: None,
            flags: None,
            root: None,
            tree: None,
        }
    }

    pub fn font(mut self, font: orbfont::Font) -> Self {
        self.font = Some(font);
        self
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = Some(theme);
        self
    }

    pub fn flags(mut self, flags: &'a [WindowFlag]) -> Self {
        self.flags = Some(flags);
        self
    }

    pub fn root<W: 'static + Widget>(mut self, root: &Arc<W>) -> Self {
        self.root = Some(root.clone());
        if let Some(ref root) = self.root {
            self.tree = build_tree(&None, root);
        }
        self
    }

    fn build(self) -> Window {
        let (rect, title, font) = (self.rect, self.title, self.font);

        let flags = match self.flags {
            Some(flags) => flags,
            None => &[],
        };

        let inner =
            InnerWindow::new_flags(rect.x, rect.y, rect.width, rect.height, title, flags).unwrap();

        let theme = match self.theme {
            Some(theme) => theme,
            None => Theme::new(),
        };

        let mut events = VecDeque::new();
        events.push_back(Event::Init);

        Window {
            inner: RefCell::new(inner),
            font: font,
            //            widgets: RefCell::new(Vec::new()),
            running: Cell::new(true),
            theme: theme,
            resize_callback: RefCell::new(None),
            _mouse_point: Point::new(0, 0),
            _mouse_left: false,
            _mouse_right: false,
            _mouse_middle: false,
            events: events,
            redraw: true,
            //            focus_manager: FocusManager::new(),
        }
    }

    pub fn print_tree(self) -> Self {
        if let Some(ref root) = self.tree {
            println!("Window (OrbTK)");
            self.print_node(root, "");
        } else {
            println!("Tree is empty.");
        }

        self
    }

    fn print_node(&self, root: &Arc<Node>, spacer: &str) {
        println!("{}|- {}", spacer, root.widget().element());
        let mut spacer = String::from(spacer);
        spacer.push_str("|    ");
        for child in root.children().borrow().iter() {
            self.print_node(child, &spacer);
        }
    }

    pub fn run(self) {
        self.build().exec();
    }
}
