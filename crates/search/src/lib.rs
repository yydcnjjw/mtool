#![windows_subsystem = "windows"]

mod search;
use qt_core::{qs, AlignmentFlag, QBox, QFlags, QStringList, WindowType};
use qt_widgets::{QApplication, QCompleter, QLineEdit};

struct CmdLine {
    widget: QBox<QLineEdit>,
}

impl CmdLine {
    fn new() -> CmdLine {
        CmdLine {
            widget: CmdLine::build_widget(),
        }
    }
    fn build_widget() -> QBox<QLineEdit> {
        unsafe {
            let edit = QLineEdit::new();
            edit.set_window_flags(QFlags::from(
                WindowType::CustomizeWindowHint | WindowType::Dialog,
            ));

            let screen = edit.screen();
            let screen_width = screen.available_size().width();
            let w = (screen_width as f32 * 0.6) as i32;
            let left = (screen_width - w) / 2;
            edit.move_2a(left, 10);
            edit.resize_2a(w, 48);

            edit.set_frame(false);
            edit.set_alignment(QFlags::from(AlignmentFlag::AlignCenter));
            edit.set_style_sheet(&qs("
QLineEdit {
    font-size: 28px;
    font-family: \"Hack\";
    padding: 0px 4px 0px 4px;
    background-color: rgba(255, 255, 255, 75%);
}
"));

            let complete = QCompleter::from_q_string_list(&QStringList::from_q_string(&qs("test")));
            edit.set_completer(&complete);
            edit.show();
            edit
        }
    }
}

pub fn run() {
    QApplication::init(|_| unsafe {
        let cmdline = CmdLine::new();

        QApplication::exec()
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
