use crate::data::load_cfg_data;
use crate::pitch_canvas::PitchPainter;
use crate::strokes::Stroke;
use crate::Files;
use anyhow::Context;
use hotwatch::{Event, Hotwatch};
use nwg::stretch::geometry::{Rect, Size};
use nwg::stretch::style::{AlignItems, Dimension as D, FlexDirection};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use winapi::shared::windef::HWND;

fn default<T: Default>() -> T {
    Default::default()
}

pub fn render_app(files: Files) -> anyhow::Result<()> {
    nwg::init()?;
    nwg::Font::set_global_family("Segoe UI")?;
    let ui = App::build(files)?;
    nwg::dispatch_thread_events();
    Ok(())
}

struct Watching {
    watcher: Hotwatch,
    files: Files,
}

pub struct Ui {
    watching: Watching,

    window: nwg::Window,
    notice_receiver: nwg::Notice,

    list_file_input: nwg::TextInput,
    cfg_file_input: nwg::TextInput,

    list_select: nwg::ListBox<String>,
    pitch_canvas: nwg::ExternCanvas,
    radios: [nwg::RadioButton; 5],

    pitch_painter: Option<PitchPainter>,

    selected_stroke: Option<Stroke>,
    selected_timing: usize,

    pub cfg_item_offsets: Vec<i64>,
    pub cfg_contents: Vec<u8>,
    _other_controls_keepalive: Vec<Box<dyn Any>>,
}

impl Ui {
    fn change_data_files(&mut self, files: Files) -> anyhow::Result<()> {
        let mut changed = false;
        if files.list_file != self.watching.files.list_file {
            let _ = self.watching.watcher.unwatch(&self.watching.files.list_file);
            self.watching
                .watcher
                .watch(&files.list_file, watch_callback(&self.notice_receiver))
                .context("failed to watch list file")?;
            self.watching.files.list_file = files.list_file;
            changed = true;
        }
        if files.cfg_file != self.watching.files.cfg_file {
            let _ = self.watching.watcher.unwatch(&self.watching.files.cfg_file);
            self.watching
                .watcher
                .watch(&files.cfg_file, watch_callback(&self.notice_receiver))
                .context("failed to watch cfg file")?;
            self.watching.files.cfg_file = files.cfg_file;
            changed = true;
        }
        if changed {
            self.load_data_files();
        }
        Ok(())
    }

    fn load_data_files(&mut self) {
        let new_data = load_cfg_data(&self.watching.files);
        match new_data {
            Ok(new_data) => {
                self.cfg_item_offsets =
                    new_data.cfg_items.iter().map(|(_, offset)| *offset).collect();
                self.cfg_contents = new_data.cfg_contents;
                let previous_selection = self.list_select.selection();
                let new_cfg_items_len = new_data.cfg_items.len();
                self.list_select.set_collection(
                    new_data
                        .cfg_items
                        .into_iter()
                        .map(|(name, offset)| (line_number_of(&self.cfg_contents, offset), name))
                        .map(|(line_number, name)| format!("{}: {}", line_number, name))
                        .collect(),
                );
                let previous_selection_if_still_valid =
                    previous_selection.filter(|&i| i < new_cfg_items_len);
                self.list_select.set_selection(previous_selection_if_still_valid);
                self.update_selected_stroke(previous_selection_if_still_valid);
            }
            Err(e) => {
                println!("failed to load data files: {:#}", e)
            }
        }
    }

    fn update_selected_stroke(&mut self, selection_index: Option<usize>) {
        self.selected_stroke = selection_index.map(|i| {
            parse_stroke(
                &self.cfg_contents,
                self.cfg_item_offsets[i],
                self.cfg_item_offsets.get(i + 1).copied(),
            )
        });
        self.pitch_canvas.invalidate();
    }
}

pub struct App {
    pub ui: Rc<RefCell<Ui>>,
    handlers: [nwg::EventHandler; 1],
}

impl Drop for App {
    /// To make sure that everything is freed without issues, the default handler must be unbound.
    fn drop(&mut self) {
        for handler in self.handlers.iter() {
            nwg::unbind_event_handler(handler);
        }
    }
}

fn rect(points: f32) -> Rect<D> {
    Rect {
        top: D::Points(points),
        bottom: D::Points(0.0),
        start: D::Points(points),
        end: D::Points(points),
    }
}

impl App {
    fn build(files: Files) -> anyhow::Result<App> {
        let mut window = default();
        nwg::Window::builder()
            .flags(
                nwg::WindowFlags::WINDOW | nwg::WindowFlags::VISIBLE | nwg::WindowFlags::RESIZABLE,
            )
            .size((700, 450))
            .position((300, 300))
            .title("Stroke preview")
            .build(&mut window)?;

        let mut list_select = default();
        nwg::ListBox::builder()
            .collection(Vec::new())
            .size((300, 10))
            .parent(&window)
            .build(&mut list_select)?;

        // Layouts

        let mut right_frame = default();
        nwg::Frame::builder()
            .parent(&window)
            .flags(nwg::FrameFlags::VISIBLE)
            .build(&mut right_frame)?;

        // nested flexbox hack: put the nested flex inside a frame, the frame can be set as a child of another flex
        let mut radios_frame = default();
        nwg::Frame::builder()
            .flags(nwg::FrameFlags::VISIBLE)
            .parent(&right_frame)
            .build(&mut radios_frame)?;

        let radios_flex = default();
        let mut flex_builder =
            nwg::FlexboxLayout::builder().parent(&radios_frame).padding(rect(0.0));

        let mut radios = [default(), default(), default(), default(), default()];
        for (i, &(text, width)) in [
            ("Very early", 100.0f32),
            ("Early", 70.0),
            ("Ideal", 70.0),
            ("Late", 70.0),
            ("Very late", 90.0),
        ]
        .iter()
        .enumerate()
        {
            use nwg::RadioButtonFlags as Flags;
            nwg::RadioButton::builder()
                .parent(&radios_frame)
                .flags(if i == 0 { Flags::VISIBLE | Flags::GROUP } else { Flags::VISIBLE })
                .check_state(if i == 2 {
                    nwg::RadioButtonState::Checked
                } else {
                    nwg::RadioButtonState::Unchecked
                })
                .text(text)
                .build(&mut radios[i])?;
            flex_builder = flex_builder
                .child(&radios[i])
                .child_size(Size { width: D::Points(width), height: D::Points(35.0) })
        }
        flex_builder.build(&radios_flex)?;

        let mut pitch_canvas = default();
        nwg::ExternCanvas::builder().parent(Some(&right_frame)).build(&mut pitch_canvas)?;

        let mut cfg_file_frame = default();
        nwg::Frame::builder()
            .parent(&right_frame)
            .flags(nwg::FrameFlags::VISIBLE)
            .build(&mut cfg_file_frame)?;
        let mut cfg_file_label = default();
        nwg::Label::builder().parent(&cfg_file_frame).text("Cfg:").build(&mut cfg_file_label)?;
        let mut cfg_file_input = default();
        nwg::TextInput::builder()
            .parent(&cfg_file_frame)
            .text(&files.cfg_file)
            .build(&mut cfg_file_input)?;
        let cfg_file_flex = default();
        nwg::FlexboxLayout::builder()
            .parent(&cfg_file_frame)
            .padding(rect(0.0))
            .child(&cfg_file_label)
            .child_size(Size { width: D::Points(40.0), height: D::Percent(1.0) })
            .child(&cfg_file_input)
            .child_size(Size { width: D::Percent(1.0), height: D::Percent(1.0) })
            .build(&cfg_file_flex)?;

        let mut list_file_frame = default();
        nwg::Frame::builder()
            .parent(&right_frame)
            .flags(nwg::FrameFlags::VISIBLE)
            .build(&mut list_file_frame)?;
        let mut list_file_label = default();
        nwg::Label::builder().parent(&list_file_frame).text("List:").build(&mut list_file_label)?;
        let mut list_file_input = default();
        nwg::TextInput::builder()
            .parent(&list_file_frame)
            .text(&files.list_file)
            .build(&mut list_file_input)?;
        let list_file_flex = default();
        nwg::FlexboxLayout::builder()
            .parent(&list_file_frame)
            .padding(rect(0.0))
            .child(&list_file_label)
            .child_size(Size { width: D::Points(40.0), height: D::Percent(1.0) })
            .child(&list_file_input)
            .child_size(Size { width: D::Percent(1.0), height: D::Percent(1.0) })
            .build(&list_file_flex)?;

        let right_flex = default();
        nwg::FlexboxLayout::builder()
            .flex_direction(FlexDirection::Column)
            .align_items(AlignItems::Center)
            .parent(&right_frame)
            .padding(Rect {
                top: D::Points(0.0),
                bottom: D::Points(10.0),
                start: D::Points(20.0),
                end: D::Points(20.0),
            })
            .child(&list_file_frame)
            .child_size(Size { width: D::Percent(1.0), height: D::Points(35.0) })
            .child_margin(rect(0.0))
            .child_flex_grow(0.0)
            .child_flex_shrink(0.0)
            .child(&cfg_file_frame)
            .child_size(Size { width: D::Percent(1.0), height: D::Points(35.0) })
            .child_margin(rect(0.0))
            .child_flex_grow(0.0)
            .child_flex_shrink(0.0)
            .child(&pitch_canvas)
            .child_size(Size { width: D::Percent(1.0), height: D::Percent(1.0) })
            .child_margin(rect(5.0))
            .child(&radios_frame)
            .child_size(Size { width: D::Points(400.0), height: D::Points(40.0) })
            .child_margin(rect(5.0))
            .build(&right_flex)?;

        let root = default();
        nwg::FlexboxLayout::builder()
            .parent(&window)
            .child(&list_select)
            .child_flex_grow(1.0)
            .child(&right_frame)
            .child_flex_grow(2.0)
            .build(&root)?;

        let mut notice_receiver = default();
        nwg::Notice::builder().parent(&window).build(&mut notice_receiver)?;

        let window_handle = window.handle;
        let mut ui = Ui {
            watching: Watching {
                watcher: Hotwatch::new_with_custom_delay(Duration::from_millis(200))?,
                files: Files { list_file: String::new(), cfg_file: String::new() },
            },
            window,
            notice_receiver,
            list_file_input,
            cfg_file_input,
            list_select,
            pitch_canvas,
            radios,
            pitch_painter: None,
            selected_stroke: None,
            selected_timing: 2,
            cfg_item_offsets: Vec::new(),
            cfg_contents: Vec::new(),
            _other_controls_keepalive: vec![
                Box::new(cfg_file_label),
                Box::new(cfg_file_flex),
                Box::new(cfg_file_frame),
                Box::new(list_file_label),
                Box::new(list_file_flex),
                Box::new(list_file_frame),
                Box::new(radios_frame),
                Box::new(right_flex),
                Box::new(right_frame),
                Box::new(root),
            ],
        };

        let _ = ui.change_data_files(files);

        let ui = Rc::new(RefCell::new(ui));

        let event_ui = Rc::downgrade(&ui);
        let handler = nwg::full_bind_event_handler(&window_handle, move |e, data, h| {
            if let Some(ui) = event_ui.upgrade() {
                // no events we are interested in occur in a re-entrant scenario
                if let Ok(mut ui) = ui.try_borrow_mut() {
                    use nwg::Event as E;
                    match e {
                        E::OnNotice if h == ui.notice_receiver => {
                            ui.load_data_files();
                        }
                        E::OnMinMaxInfo if h == ui.window => {
                            data.on_min_max().set_min_size(650, 550);
                        }
                        E::OnInit if h == ui.window => ui.pitch_painter = Some(PitchPainter::new()),
                        E::OnPaint if h == ui.pitch_canvas => {
                            if let Some(painter) = &ui.pitch_painter {
                                painter.paint(
                                    data.on_paint(),
                                    ui.selected_stroke.as_ref(),
                                    ui.selected_timing,
                                );
                            }
                        }
                        E::OnListBoxSelect if h == ui.list_select => {
                            let i = ui.list_select.selection();
                            ui.update_selected_stroke(i);
                        }
                        E::OnTextInput if h == ui.cfg_file_input || h == ui.list_file_input => {
                            let list_file = ui.list_file_input.text();
                            let cfg_file = ui.cfg_file_input.text();
                            // this does IO on the main thread, but it won't be that slow
                            if let Err(e) = ui.change_data_files(Files { list_file, cfg_file }) {
                                println!("error changing data files: {:#}", e);
                            };
                        }
                        E::OnWindowClose if h == ui.window => nwg::stop_thread_dispatch(),
                        E::OnButtonClick => {
                            if let Some(i) = ui.radios.iter().position(|r| *r == h) {
                                ui.selected_timing = i;
                                ui.pitch_canvas.invalidate();
                            }
                        }
                        _ => {}
                    }
                }
            }
        });

        Ok(App { ui, handlers: [handler] })
    }
}

struct WindowWrapper(HWND);
unsafe impl Sync for WindowWrapper {}
unsafe impl Send for WindowWrapper {}

fn watch_callback(notice_receiver: &nwg::Notice) -> impl FnMut(Event) + Send + 'static {
    let sender = notice_receiver.sender();
    move |event: Event| {
        if let Event::Write(_) = event {
            sender.notice();
        }
    }
}

fn line_number_of(cfg_contents: &[u8], offset: i64) -> usize {
    // offset found experimentally
    let bytes_of_non_strokes: i64 = 0x7c60;
    // https://www.planetcricket.org/forums/threads/config-editor-v3.8697/post-130389
    // offset of first stroke
    let delta = -558891009 + bytes_of_non_strokes;
    let offset = offset + delta;
    cfg_contents[..(offset as usize)].iter().filter(|&&c| c == b'\n').count() + 1
}

fn parse_stroke(cfg_contents: &[u8], offset: i64, offset_next: Option<i64>) -> Stroke {
    // offset found experimentally
    let bytes_of_non_strokes: i64 = 0x7c60;
    // https://www.planetcricket.org/forums/threads/config-editor-v3.8697/post-130389
    // offset of first stroke
    let delta = -558891009 + bytes_of_non_strokes;
    let offset = offset + delta;
    let slice = if let Some(end) = offset_next {
        let end = end + delta;
        &cfg_contents[(offset as usize)..(end as usize)]
    } else {
        &cfg_contents[(offset as usize)..]
    };
    String::from_utf8_lossy(slice);
    Stroke::parse(slice).unwrap()
}
