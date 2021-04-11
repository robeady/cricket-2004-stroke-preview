use crate::{pitch_canvas::PitchPainter, strokes::Stroke};
use nwg::stretch::geometry::{Rect, Size};
use nwg::stretch::style::{Dimension as D, FlexDirection};
use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap};

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
    pub stroke_cfg_names: Vec<String>,
    pub strokes: HashMap<String, Stroke>,
}

pub struct Ui {
    window: nwg::Window,
    flex: nwg::FlexboxLayout,

    list_select: nwg::ComboBox<String>,
    pitch_canvas: nwg::ExternCanvas,

    pitch_painter: Option<PitchPainter>,

    data: UiData,
}

impl Ui {
    pub fn update_data(&mut self, new_data: UiData) {
        self.data = new_data;
        self.pitch_canvas.invalidate();
        self.list_select.set_collection(self.data.stroke_cfg_names.clone());
    }
}

pub struct UiWrapper {
    pub ui: Rc<RefCell<Ui>>,
    handler: nwg::EventHandler,
}

impl Drop for UiWrapper {
    /// To make sure that everything is freed without issues, the default handler must be unbound.
    fn drop(&mut self) {
        nwg::unbind_event_handler(&self.handler);
    }
}

impl UiWrapper {
    fn build() -> anyhow::Result<UiWrapper> {
        let mut window = default();
        nwg::Window::builder()
            .flags(
                nwg::WindowFlags::WINDOW | nwg::WindowFlags::VISIBLE | nwg::WindowFlags::RESIZABLE,
            )
            .size((300, 300))
            .position((300, 300))
            .title("Stroke preview")
            .build(&mut window)?;

        let mut list_select = default();
        nwg::ComboBox::builder()
            .collection(vec!["a".to_string()])
            .parent(&window)
            .build(&mut list_select)?;

        let mut pitch_canvas = default();
        nwg::ExternCanvas::builder().parent(Some(&window)).build(&mut pitch_canvas)?;

        // Layouts
        let flex = default();
        nwg::FlexboxLayout::builder()
            .flex_direction(FlexDirection::Column)
            .parent(&window)
            .child(&list_select)
            // note that this flexbox implementation has no notion of a 'content size'
            // so for the combobox we hardcode a fixed height
            .child_min_size(Size { width: D::Auto, height: D::Points(30.0) })
            .child(&pitch_canvas)
            .child_flex_grow(1.0)
            .build(&flex)?;

        let window_handle = window.handle;
        let ui = Rc::new(RefCell::new(Ui {
            window,
            flex,
            list_select,
            pitch_canvas,
            pitch_painter: None,
            data: UiData { strokes: HashMap::new(), stroke_cfg_names: Vec::new() },
        }));

        let event_ui = Rc::downgrade(&ui);
        let handler = nwg::full_bind_event_handler(&window_handle, move |e, data, h| {
            if let Some(ui) = event_ui.upgrade() {
                let mut ui = ui.borrow_mut();
                use nwg::Event as E;
                match e {
                    E::OnInit if h == ui.window => ui.pitch_painter = Some(PitchPainter::new()),
                    E::OnPaint if h == ui.pitch_canvas => {
                        if let Some(painter) = &ui.pitch_painter {
                            painter.paint(
                                data.on_paint(),
                                ui.list_select
                                    .selection()
                                    .map(|i| &ui.data.strokes[&ui.data.stroke_cfg_names[i]]),
                            );
                        }
                    }
                    E::OnComboxBoxSelection if h == ui.list_select => {}
                    E::OnWindowClose if h == ui.window => nwg::stop_thread_dispatch(),
                    _ => {}
                }
            }
        });

        Ok(UiWrapper { ui, handler })
    }
}
