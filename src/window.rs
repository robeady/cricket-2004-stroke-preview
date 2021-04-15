use crate::pitch_canvas::PitchPainter;
use crate::strokes::Stroke;
use nwg::stretch::geometry::{Rect, Size};
use nwg::stretch::style::{AlignItems, Dimension as D, FlexDirection};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

fn default<T: Default>() -> T {
    Default::default()
}

pub fn render_ui(initial_data: UiData) -> anyhow::Result<()> {
    nwg::init()?;
    nwg::Font::set_global_family("Segoe UI")?;
    let ui = UiWrapper::build()?;
    ui.ui.borrow_mut().update_data(initial_data);
    nwg::dispatch_thread_events();
    Ok(())
}

pub struct UiData {
    pub cfg_items: Vec<(String, i64)>,
    pub cfg_contents: Vec<u8>,
}

pub struct Ui {
    window: nwg::Window,

    list_file: nwg::TextInput,
    cfg_file: nwg::TextInput,

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
    pub fn update_data(&mut self, new_data: UiData) {
        self.cfg_item_offsets = new_data.cfg_items.iter().map(|(_, offset)| *offset).collect();
        self.cfg_contents = new_data.cfg_contents;
        self.pitch_canvas.invalidate();
        self.list_select
            .set_collection(new_data.cfg_items.into_iter().map(|(name, _)| name).collect());
    }
}

pub struct UiWrapper {
    pub ui: Rc<RefCell<Ui>>,
    handlers: [nwg::EventHandler; 1],
}

impl Drop for UiWrapper {
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

impl UiWrapper {
    fn build() -> anyhow::Result<UiWrapper> {
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
            .collection(vec!["a".to_string()])
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
        let mut cfg_file = default();
        nwg::TextInput::builder().parent(&cfg_file_frame).text("AI.cfg").build(&mut cfg_file)?;
        let cfg_file_flex = default();
        nwg::FlexboxLayout::builder()
            .parent(&cfg_file_frame)
            .padding(rect(0.0))
            .child(&cfg_file_label)
            .child_size(Size { width: D::Points(40.0), height: D::Percent(1.0) })
            .child(&cfg_file)
            .child_size(Size { width: D::Percent(1.0), height: D::Percent(1.0) })
            .build(&cfg_file_flex)?;

        let mut list_file_frame = default();
        nwg::Frame::builder()
            .parent(&right_frame)
            .flags(nwg::FrameFlags::VISIBLE)
            .build(&mut list_file_frame)?;
        let mut list_file_label = default();
        nwg::Label::builder().parent(&list_file_frame).text("List:").build(&mut list_file_label)?;
        let mut list_file = default();
        nwg::TextInput::builder()
            .parent(&list_file_frame)
            .text("list.txt")
            .build(&mut list_file)?;
        let list_file_flex = default();
        nwg::FlexboxLayout::builder()
            .parent(&list_file_frame)
            .padding(rect(0.0))
            .child(&list_file_label)
            .child_size(Size { width: D::Points(40.0), height: D::Percent(1.0) })
            .child(&list_file)
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

        let window_handle = window.handle;
        let ui = Rc::new(RefCell::new(Ui {
            window,
            list_file,
            cfg_file,
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
        }));

        let event_ui = Rc::downgrade(&ui);
        let handler = nwg::full_bind_event_handler(&window_handle, move |e, data, h| {
            if let Some(ui) = event_ui.upgrade() {
                // no events we are interested in occur in a re-entrant scenario
                if let Ok(mut ui) = ui.try_borrow_mut() {
                    use nwg::Event as E;
                    match e {
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
                            if let Some(i) = ui.list_select.selection() {
                                ui.selected_stroke = Some(parse_stroke(
                                    &ui.cfg_contents,
                                    ui.cfg_item_offsets[i],
                                    ui.cfg_item_offsets.get(i + 1).copied(),
                                ));
                                ui.pitch_canvas.invalidate();
                            }
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

        Ok(UiWrapper { ui, handlers: [handler] })
    }
}

fn parse_stroke(cfg_contents: &[u8], offset: i64, offset_next: Option<i64>) -> Stroke {
    // https://www.planetcricket.org/forums/threads/config-editor-v3.8697/post-130389
    // offset of first stroke
    let delta = -558891009;
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
