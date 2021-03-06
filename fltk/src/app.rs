pub use crate::enums::*;
use crate::prelude::*;
use crate::window::*;
use fltk_sys::fl::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::{
    ffi::{CStr, CString},
    mem,
    os::raw,
};

pub type WidgetPtr = *mut fltk_sys::widget::Fl_Widget;

/// The fonts associated with the application
pub(crate) static mut FONTS: Vec<String> = Vec::new();

static mut LOADED_FONT: Option<&str> = None;

/// Runs the event loop
pub fn run() -> Result<(), FltkError> {
    unsafe {
        match Fl_run() {
            0 => Ok(()),
            _ => Err(FltkError::Internal(FltkErrorKind::FailedToRun)),
        }
    }
}

/// Locks the main UI thread
pub fn lock() -> Result<(), FltkError> {
    unsafe {
        match Fl_lock() {
            0 => Ok(()),
            _ => Err(FltkError::Internal(FltkErrorKind::FailedToLock)),
        }
    }
}

/// Set the app scheme
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Scheme {
    /// Base fltk scheming
    Base,
    /// inspired by the Aqua user interface on Mac OS X
    Plastic,
    /// inspired by the GTK+ theme
    Gtk,
    /// inspired by the Clearlooks Glossy scheme
    Gleam,
}

/// sets the scheme of the application
pub fn set_scheme(scheme: Scheme) {
    let name_str = match scheme {
        Scheme::Base => "base",
        Scheme::Gtk => "gtk+",
        Scheme::Gleam => "gleam",
        Scheme::Plastic => "plastic",
    };
    let name_str = CString::safe_new(name_str).unwrap();
    unsafe { Fl_set_scheme(name_str.as_ptr()) }
}

/// Gets the scheme of the application
pub fn scheme() -> Scheme {
    unsafe {
        use Scheme::*;
        match Fl_scheme() {
            0 => Base,
            1 => Gtk,
            2 => Gleam,
            3 => Plastic,
            _ => unreachable!(),
        }
    }
}

/// Alias Scheme to AppScheme
pub type AppScheme = Scheme;

/// Unlocks the main UI thread
#[allow(dead_code)]
pub fn unlock() {
    unsafe {
        Fl_unlock();
    }
}

/// Awakens the main UI thread with a callback
pub fn awake(cb: Box<dyn FnMut()>) {
    unsafe {
        unsafe extern "C" fn shim(data: *mut raw::c_void) {
            let a: *mut Box<dyn FnMut()> = data as *mut Box<dyn FnMut()>;
            let f: &mut (dyn FnMut()) = &mut **a;
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f()));
        }
        let a: *mut Box<dyn FnMut()> = Box::into_raw(Box::new(cb));
        let data: *mut raw::c_void = a as *mut raw::c_void;
        let callback: Fl_Awake_Handler = Some(shim);
        Fl_awake(callback, data);
    }
}

/// Basic Application struct, used to instatiate, set the scheme and run the event loop
#[derive(Debug, Copy, Clone)]
pub struct App {}

impl App {
    /// Instantiates an App type
    pub fn default() -> App {
        register_images();
        init_all();
        unsafe {
            FONTS = vec![
                "Helvetica".to_owned(),
                "HelveticaBold".to_owned(),
                "HelveticaItalic".to_owned(),
                "HelveticaBoldItalic".to_owned(),
                "Courier".to_owned(),
                "CourierBold".to_owned(),
                "CourierItalic".to_owned(),
                "CourierBoldItalic".to_owned(),
                "Times".to_owned(),
                "TimesBold".to_owned(),
                "TimesItalic".to_owned(),
                "TimesBoldItalic".to_owned(),
                "Symbol".to_owned(),
                "Screen".to_owned(),
                "ScreenBold".to_owned(),
                "Zapfdingbats".to_owned(),
            ];
        }
        App {}
    }

    /// Sets the scheme of the application
    pub fn set_scheme(&mut self, scheme: Scheme) {
        set_scheme(scheme);
    }

    /// Sets the scheme of the application
    pub fn with_scheme(self, scheme: Scheme) -> App {
        set_scheme(scheme);
        self
    }

    /// Gets the scheme of the application
    pub fn scheme(&self) -> Scheme {
        scheme()
    }

    /// Runs the event loop
    pub fn run(&self) -> Result<(), FltkError> {
        lock()?;
        run()
    }

    /// Wait for incoming messages
    pub fn wait(&self) -> Result<bool, FltkError> {
        lock()?;
        Ok(wait())
    }

    /// Loads system fonts
    pub fn load_system_fonts(self) -> Self {
        unsafe {
            FONTS = get_font_names();
        }
        self
    }

    /// Loads a font from a path.
    /// On success, returns a String with the ttf Font Family name. The font's index is always 16.
    /// As such only one font can be loaded at a time.
    /// The font name can be used with Font::by_name, and index with Font::by_index.
    /// # Examples
    /// ```
    /// use fltk::*;
    /// let app = app::App::default();
    /// let font = app.load_font(&std::path::Path::new("font.ttf")).unwrap();
    /// let mut frame = frame::Frame::new(0, 0, 400, 100, "Hello");
    /// frame.set_label_font(Font::by_name(&font));
    /// ```
    pub fn load_font(&self, path: &std::path::Path) -> Result<String, FltkError> {
        if !path.exists() {
            return Err::<String, FltkError>(FltkError::Internal(FltkErrorKind::ResourceNotFound));
        }
        if let Some(p) = path.to_str() {
            let name = load_font(p)?;
            Ok(name)
        } else {
            Err(FltkError::Internal(FltkErrorKind::ResourceNotFound))
        }
    }

    /// Set the visual of the application
    pub fn set_visual(&self, mode: Mode) -> Result<(), FltkError> {
        set_visual(mode)
    }

    /// Awakens the main UI thread with a callback
    pub fn awake(&self, cb: Box<dyn FnMut()>) {
        unsafe {
            unsafe extern "C" fn shim(data: *mut raw::c_void) {
                let a: *mut Box<dyn FnMut()> = data as *mut Box<dyn FnMut()>;
                let f: &mut (dyn FnMut()) = &mut **a;
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f()));
            }
            let a: *mut Box<dyn FnMut()> = Box::into_raw(Box::new(cb));
            let data: *mut raw::c_void = a as *mut raw::c_void;
            let callback: Fl_Awake_Handler = Some(shim);
            Fl_awake(callback, data);
        }
    }

    /// Returns the apps windows.
    pub fn windows(&self) -> Option<Vec<Window>> {
        let mut v: Vec<Window> = vec![];
        let first = first_window();
        first.as_ref()?;
        let first = first?;
        v.push(first.clone());
        let mut win = first;
        while let Some(wind) = next_window(&win) {
            v.push(wind.clone());
            win = wind;
        }
        Some(v)
    }

    /// Redraws the app
    pub fn redraw(&self) {
        redraw()
    }

    /// Set the app as damaged to reschedule a redraw in the next event loop cycle
    pub fn set_damage(&self, flag: bool) {
        set_damage(flag)
    }

    /// Returns whether an app element is damaged
    pub fn damage(&self) -> bool {
        damage()
    }

    /// Quit the application
    pub fn quit(&self) {
        quit()
    }
}

/// Returns the latest captured event
pub fn event() -> Event {
    unsafe {
        let x = Fl_event();
        let x: Event = mem::transmute(x);
        x
    }
}

/// Returns the presed key
pub fn event_key() -> Key {
    unsafe {
        let x = Fl_event_key();
        mem::transmute(x)
    }
}

/// Returns a textual representation of the latest event
pub fn event_text() -> String {
    unsafe {
        let text = Fl_event_text();
        if text.is_null() {
            String::from("")
        } else {
            CStr::from_ptr(text as *mut raw::c_char)
                .to_string_lossy()
                .to_string()
        }
    }
}

/// Returns the captured button event
pub fn event_button() -> i32 {
    unsafe { Fl_event_button() }
}

/// Returns the number of clicks
pub fn event_clicks() -> bool {
    unsafe {
        match Fl_event_clicks() {
            0 => false,
            _ => true,
        }
    }
}

/// Gets the x coordinate of the mouse in the window
pub fn event_x() -> i32 {
    unsafe {
        Fl_event_x()
    }
}

/// Gets the y coordinate of the mouse in the window
pub fn event_y() -> i32 {
    unsafe {
        Fl_event_y()
    }
}

/// Gets the x coordinate of the mouse in the screen
pub fn event_x_root() -> i32 {
    unsafe {
        Fl_event_x_root()
    }
}

/// Gets the y coordinate of the mouse in the screen
pub fn event_y_root() -> i32 {
    unsafe {
        Fl_event_y_root()
    }
}

/// Gets the difference in x axis of the mouse coordinates from the screen to the window
pub fn event_dx() -> i32 {
    unsafe {
        Fl_event_dx()
    }
}

/// Gets the difference in y axis of the mouse coordinates from the screen to the window
pub fn event_dy() -> i32 {
    unsafe {
        Fl_event_dy()
    }
}

/// Gets the mouse coordinates relative to the screen
pub fn get_mouse() -> (i32, i32) {
    unsafe {
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        Fl_get_mouse(&mut x, &mut y);
        (x, y)
    }
}

/// Returns the x and y coordinates of the captured event
pub fn event_coords() -> (i32, i32) {
    unsafe { (Fl_event_x(), Fl_event_y()) }
}

/// Determines whether an event was a click
pub fn event_is_click() -> bool {
    unsafe {
        match Fl_event_is_click() {
            0 => false,
            _ => true,
        }
    }
}

/// Returns the duration of an event
pub fn event_length() -> u32 {
    unsafe { Fl_event_length() as u32 }
}

/// Returns the state of the event
pub fn event_state() -> Shortcut {
    unsafe { mem::transmute(Fl_event_state()) }
}

/// Returns a pair of the width and height of the screen
pub fn screen_size() -> (f64, f64) {
    unsafe { ((Fl_screen_w() as f64 / 0.96), (Fl_screen_h() as f64 / 0.96)) }
}

/// Used for widgets implementing the InputExt, pastes content from the clipboard
pub fn paste<T>(widget: &T)
where
    T: WidgetExt + InputExt,
{
    assert!(!widget.was_deleted());
    unsafe {
        Fl_paste(widget.as_widget_ptr() as *mut fltk_sys::fl::Fl_Widget, 1);
    }
}

/// Sets the callback of a widget
pub fn set_callback<W>(widget: &mut W, cb: Box<dyn FnMut()>)
where
    W: WidgetExt,
{
    assert!(!widget.was_deleted());
    unsafe {
        unsafe extern "C" fn shim(_wid: *mut fltk_sys::widget::Fl_Widget, data: *mut raw::c_void) {
            let a: *mut Box<dyn FnMut()> = data as *mut Box<dyn FnMut()>;
            let f: &mut (dyn FnMut()) = &mut **a;
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f()));
        }
        widget.unset_callback();
        let a: *mut Box<dyn FnMut()> = Box::into_raw(Box::new(cb));
        let data: *mut raw::c_void = a as *mut raw::c_void;
        let callback: fltk_sys::widget::Fl_Callback = Some(shim);
        fltk_sys::widget::Fl_Widget_callback_with_captures(widget.as_widget_ptr(), callback, data);
    }
}

/// Set a widget callback using a C style API, when boxing is not desired
/// # Safety
/// The function involves dereferencing externally provided raw pointers
pub unsafe fn set_raw_callback<W>(widget: &mut W, data: *mut raw::c_void, cb: Option<fn(WidgetPtr, *mut raw::c_void)>)
where
    W: WidgetExt,
{
    assert!(!widget.was_deleted());
    let cb: Option<unsafe extern "C" fn(WidgetPtr, *mut raw::c_void)> = mem::transmute(cb);
    fltk_sys::widget::Fl_Widget_callback_with_captures(widget.as_widget_ptr(), cb, data);
}

/// Initializes loaded fonts of a certain pattern ```name```
pub fn set_fonts(name: &str) -> u8 {
    let name = CString::safe_new(name).unwrap();
    unsafe { Fl_set_fonts(name.as_ptr() as *mut raw::c_char) as u8 }
}

/// Gets the name of a font through its index
pub fn font_name(idx: usize) -> Option<String> {
    unsafe {
        Some(FONTS[idx].clone())
    }
}

/// Returns a list of available fonts to the application
pub fn get_font_names() -> Vec<String> {
    let mut vec: Vec<String> = vec![];
    let cnt = set_fonts("*") as usize;
    for i in 0..cnt {
        let temp = unsafe {
            CStr::from_ptr(Fl_get_font(i as i32))
                .to_string_lossy().to_string()
        };
        vec.push(temp);
    }
    vec
}

/// Finds the index of a font through its name
pub fn font_index(name: &str) -> Option<usize> {
    unsafe {
        FONTS.iter().position(|i| i == name)
    }
}

/// Gets the number of loaded fonts
pub fn font_count() -> usize {
    unsafe {
        FONTS.len()
    }
}

/// Gets a Vector<String> of loaded fonts
pub fn fonts() -> Vec<String> {
    unsafe { FONTS.clone() }
}

/// Adds a custom handler for unhandled events
pub fn add_handler(cb: fn(Event) -> bool) {
    unsafe {
        let callback: Option<unsafe extern "C" fn(ev: raw::c_int) -> raw::c_int> =
            Some(mem::transmute(move |ev| {
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| cb(ev) as i32));
            }));
        Fl_add_handler(callback);
    }
}

/// Starts waiting for events
pub fn wait() -> bool {
    unsafe {
        match Fl_wait() {
            0 => false,
            _ => true,
        }
    }
}

/// Waits a maximum of `dur` seconds or until "something happens".
pub fn wait_for(dur: f64) -> Result<(), FltkError> {
    unsafe {
        if Fl_wait_for(dur) >= 0.0 {
            Ok(())
        } else {
            Err(FltkError::Unknown(String::from("An unknown error occured!")))
        }
    }
}

/// Sends a custom message
fn awake_msg<T>(msg: T) {
    unsafe { Fl_awake_msg(Box::into_raw(Box::from(msg)) as *mut raw::c_void) }
}

/// Receives a custom message
fn thread_msg<T>() -> Option<T> {
    unsafe {
        let msg = Fl_thread_msg();
        if msg.is_null() {
            None
        } else {
            let msg = Box::from_raw(msg as *const _ as *mut T);
            Some(*msg)
        }
    }
}

#[repr(C)]
struct Message<T: Copy + Send + Sync> {
    hash: u64,
    sz: usize,
    msg: T,
}

/// Creates a sender struct
#[derive(Debug, Clone, Copy)]
pub struct Sender<T: Copy + Send + Sync> {
    data: std::marker::PhantomData<T>,
    hash: u64,
    sz: usize,
}

impl<T: Copy + Send + Sync> Sender<T> {
    /// Sends a message
    pub fn send(&self, val: T) {
        let msg = Message {
            hash: self.hash,
            sz: self.sz,
            msg: val,
        };
        awake_msg(msg)
    }
}

/// Creates a receiver struct
#[derive(Debug, Clone, Copy)]
pub struct Receiver<T: Copy + Send + Sync> {
    data: std::marker::PhantomData<T>,
    hash: u64,
    sz: usize,
}

impl<T: Copy + Send + Sync> Receiver<T> {
    /// Receives a message
    pub fn recv(&self) -> Option<T> {
        let data: Option<Message<T>> = thread_msg();
        if let Some(data) = data {
            if data.sz == self.sz && data.hash == self.hash {
                Some(data.msg)
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Creates a channel returning a Sender and Receiver structs
// The implementation could really use generic statics
pub fn channel<T: Copy + Send + Sync>() -> (Sender<T>, Receiver<T>) {
    let msg_sz = std::mem::size_of::<T>();
    let type_name = std::any::type_name::<T>();
    let mut hasher = DefaultHasher::new();
    type_name.hash(&mut hasher);
    let type_hash = hasher.finish();

    let s = Sender {
        data: std::marker::PhantomData,
        hash: type_hash,
        sz: msg_sz,
    };
    let r = Receiver {
        data: std::marker::PhantomData,
        hash: type_hash,
        sz: msg_sz,
    };
    (s, r)
}

/// Returns the first window of the application
pub fn first_window() -> Option<Window> {
    unsafe {
        let x = Fl_first_window();
        if x.is_null() {
            None
        } else {
            let x = Window::from_widget_ptr(x as *mut fltk_sys::widget::Fl_Widget);
            Some(x)
        }
    }
}

/// Returns the next window in order
pub fn next_window<W: WindowExt>(w: &W) -> Option<Window> {
    unsafe {
        let x = Fl_next_window(w.as_widget_ptr() as *const raw::c_void);
        if x.is_null() {
            None
        } else {
            let x = Window::from_widget_ptr(x as *mut fltk_sys::widget::Fl_Widget);
            Some(x)
        }
    }
}

/// Quit the app
pub fn quit() {
    unsafe {
        if let Some(loaded_font) = LOADED_FONT {
            // Shouldn't fail
            unload_font(loaded_font).unwrap_or(());
        }
    }
    let mut v: Vec<Window> = vec![];
    let first = first_window();
    if first.is_none() {
        return;
    }
    let first = first.unwrap();
    v.push(first.clone());
    let mut win = first;
    while let Some(wind) = next_window(&win) {
        v.push(wind.clone());
        win = wind;
    }
    for mut i in v {
        if i.shown() {
            i.hide();
        }
    }
}

/// Adds a one-shot timeout callback. The timeout duration `tm` is indicated in seconds
pub fn add_timeout(tm: f64, cb: Box<dyn FnMut()>) {
    unsafe {
        unsafe extern "C" fn shim(data: *mut raw::c_void) {
            let a: *mut Box<dyn FnMut()> = data as *mut Box<dyn FnMut()>;
            let f: &mut (dyn FnMut()) = &mut **a;
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f()));
        }
        let a: *mut Box<dyn FnMut()> = Box::into_raw(Box::new(cb));
        let data: *mut raw::c_void = a as *mut raw::c_void;
        let callback: Option<unsafe extern "C" fn(arg1: *mut raw::c_void)> = Some(shim);
        fltk_sys::fl::Fl_add_timeout(tm, callback, data);
    }
}

/// Repeats a timeout callback from the expiration of the previous timeout
/// You may only call this method inside a timeout callback.
/// The timeout duration `tm` is indicated in seconds
pub fn repeat_timeout(tm: f64, cb: Box<dyn FnMut()>) {
    unsafe {
        unsafe extern "C" fn shim(data: *mut raw::c_void) {
            let a: *mut Box<dyn FnMut()> = data as *mut Box<dyn FnMut()>;
            let f: &mut (dyn FnMut()) = &mut **a;
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f()));
        }
        let a: *mut Box<dyn FnMut()> = Box::into_raw(Box::new(cb));
        let data: *mut raw::c_void = a as *mut raw::c_void;
        let callback: Option<unsafe extern "C" fn(arg1: *mut raw::c_void)> = Some(shim);
        fltk_sys::fl::Fl_repeat_timeout(tm, callback, data);
    }
}

/// Removes a timeout callback
pub fn remove_timeout(cb: Box<dyn FnMut()>) {
    unsafe {
        unsafe extern "C" fn shim(data: *mut raw::c_void) {
            let a: *mut Box<dyn FnMut()> = data as *mut Box<dyn FnMut()>;
            let f: &mut (dyn FnMut()) = &mut **a;
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f()));
        }
        let a: *mut Box<dyn FnMut()> = Box::into_raw(Box::new(cb));
        let data: *mut raw::c_void = a as *mut raw::c_void;
        let callback: Option<unsafe extern "C" fn(arg1: *mut raw::c_void)> = Some(shim);
        fltk_sys::fl::Fl_remove_timeout(callback, data);
    }
}

/// Returns whether a quit signal was sent
pub fn should_program_quit() -> bool {
    unsafe {
        match Fl_should_program_quit() {
            0 => false,
            _ => true,
        }
    }
}

/// Determines whether a program should quit
pub fn program_should_quit(flag: bool) {
    unsafe { Fl_program_should_quit(flag as i32) }
}

/// Returns whether an event occured within a widget
pub fn event_inside_widget<Wid: WidgetExt>(wid: &Wid) -> bool {
    assert!(!wid.was_deleted());
    let x = wid.x();
    let y = wid.y();
    let w = wid.width();
    let h = wid.height();
    unsafe {
        match Fl_event_inside(x, y, w, h) {
            0 => false,
            _ => true,
        }
    }
}

/// Returns whether an event occured within a region
pub fn event_inside(x: i32, y: i32, w: i32, h: i32) -> bool {
    unsafe {
        match Fl_event_inside(x, y, w, h) {
            0 => false,
            _ => true,
        }
    }
}

/// Gets the widget that is below the mouse cursor
pub fn belowmouse<Wid: WidgetExt>() -> Option<impl WidgetExt> {
    unsafe {
        let x = Fl_belowmouse() as *mut fltk_sys::fl::Fl_Widget;
        if x.is_null() {
            None
        } else {
            Some(crate::widget::Widget::from_widget_ptr(
                x as *mut fltk_sys::widget::Fl_Widget,
            ))
        }
    }
}

/// Deletes widgets and their children.
pub fn delete_widget<Wid: WidgetExt>(wid: &mut Wid) {
    assert!(!wid.was_deleted());
    unsafe {
        Fl_delete_widget(wid.as_widget_ptr() as *mut fltk_sys::fl::Fl_Widget);
        wid.cleanup();
    }
}

/// Deletes widgets and their children recursively deleting their user data
/// # Safety
/// Deletes user_data and any captured objects in the callback
pub unsafe fn unsafe_delete_widget<Wid: WidgetExt>(wid: &mut Wid) {
    assert!(!wid.was_deleted());
    let _u = wid.user_data();
    Fl_delete_widget(wid.as_widget_ptr() as *mut fltk_sys::fl::Fl_Widget);
    wid.cleanup();
}

/// Registers all images supported by SharedImage
pub fn register_images() {
    unsafe { fltk_sys::image::Fl_register_images() }
}

/// Inits all styles available to FLTK
pub fn init_all() {
    unsafe { fltk_sys::fl::Fl_init_all() }
}

/// Redraws everything
pub fn redraw() {
    unsafe { Fl_redraw() }
}

/// Returns whether the event is a shift press
pub fn is_event_shift() -> bool {
    unsafe { Fl_event_shift() != 0 }
}

/// Returns whether the event is a control key press
pub fn is_event_ctrl() -> bool {
    unsafe { Fl_event_ctrl() != 0 }
}

/// Returns whether the event is a command key press
pub fn is_event_command() -> bool {
    unsafe { Fl_event_command() != 0 }
}

/// Returns whether the event is a alt key press
pub fn is_event_alt() -> bool {
    unsafe { Fl_event_alt() != 0 }
}

/// Sets the damage to true or false, illiciting a redraw by the application
pub fn set_damage(flag: bool) {
    unsafe { Fl_set_damage(flag as i32) }
}

/// Returns whether any of the widgets were damaged
pub fn damage() -> bool {
    unsafe { Fl_damage() != 0 }
}

/// Sets the visual mode of the application
pub fn set_visual(mode: Mode) -> Result<(), FltkError> {
    unsafe {
        match Fl_visual(mode as i32) {
            0 => Err(FltkError::Internal(FltkErrorKind::FailedOperation)),
            _ => Ok(()),
        }
    }
}

/// Makes FLTK use its own colormap. This may make FLTK display better
pub fn own_colormap() {
    unsafe { Fl_own_colormap() }
}

/// Gets the widget which was pushed
pub fn pushed() -> Option<crate::widget::Widget> {
    unsafe {
        let ptr = Fl_pushed();
        if ptr.is_null() {
            None
        } else {
            Some(crate::widget::Widget::from_raw(
                ptr as *mut fltk_sys::widget::Fl_Widget,
            ))
        }
    }
}

/// Gets the widget which has focus
pub fn focus() -> Option<crate::widget::Widget> {
    unsafe {
        let ptr = Fl_focus();
        if ptr.is_null() {
            None
        } else {
            Some(crate::widget::Widget::from_raw(
                ptr as *mut fltk_sys::widget::Fl_Widget,
            ))
        }
    }
}

/// Sets the widget which has focus
pub fn set_focus<W: WidgetExt>(wid: &W) {
    unsafe { Fl_set_focus(wid.as_widget_ptr() as *mut raw::c_void) }
}

/// Delays the current thread by millis. Because std::thread::sleep isn't accurate on windows!
/// Caution: It's a busy wait!
pub fn delay(millis: u128) {
    let now = std::time::Instant::now();
    loop {
        let after = std::time::Instant::now();
        if after.duration_since(now).as_millis() > millis {
            break;
        }
    }
}

/// Gets FLTK version
pub fn version() -> f64 {
    unsafe { Fl_version() }
}

/// Gets FLTK API version
pub fn api_version() -> i32 {
    unsafe { Fl_api_version() }
}

/// Gets FLTK ABI version
pub fn abi_version() -> i32 {
    unsafe { Fl_abi_version() }
}

/// Gets FLTK crate version
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// The current graphics context of the app, fl_gc
/// *mut c_void to HDC on Windows, CGContextRef on MacOS, _XGC on X11
pub type GraphicsContext = *mut raw::c_void;

/// Get the graphics context, fl_gc
pub fn graphics_context() -> GraphicsContext {
    unsafe {
        let ctx = fltk_sys::window::Fl_gc();
        assert!(!ctx.is_null());
        ctx
    }
}

/// The display global variable, fl_display
/// _XDisplay on X11, HINSTANCE on Windows. 
pub type Display = *mut raw::c_void;

/// Gets the display global variable, fl_display
/// _XDisplay on X11, HINSTANCE on Windows.
pub fn display() -> Display {
    unsafe {
        let disp = fltk_sys::window::Fl_display();
        assert!(!disp.is_null());
        disp
    }
}

/// Initiate dnd action
pub fn dnd() {
    unsafe {
        Fl_dnd();
    }
}

/// Load a font from a file
fn load_font(path: &str) -> Result<String, FltkError> {
    unsafe {
        let path = CString::new(path)?;
        if let Some(load_font) = LOADED_FONT {
            unload_font(load_font)?;
        }
        let ptr = Fl_load_font(path.as_ptr());
        if ptr.is_null() {
            Err::<String, FltkError>(FltkError::Internal(FltkErrorKind::FailedOperation))
        } else {
            let name = CString::from_raw(ptr as *mut _).to_string_lossy().to_string();
            if FONTS.len() < 17 {
                FONTS.push(name.clone());
            } else {
                FONTS[16] = name.clone();
            }
            Ok(name)
        }
    }
}

/// Unload a loaded font
fn unload_font(path: &str) -> Result<(), FltkError> {
    unsafe {
        let check = std::path::PathBuf::from(path);
        if !check.exists() {
            return Err::<(), FltkError>(FltkError::Internal(FltkErrorKind::ResourceNotFound));
        }
        let path = CString::new(path)?;
        Fl_unload_font(path.as_ptr());
        Ok(())
    }
}
