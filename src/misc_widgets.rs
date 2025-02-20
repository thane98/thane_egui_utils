use egui::{Color32, Frame, Stroke, TextEdit, Ui, Widget};
use rfd::FileDialog;

pub fn blank_slate(ui: &mut Ui, placeholder: &str) {
    ui.centered_and_justified(|ui| {
        ui.heading(placeholder);
    });
}

pub fn raised_heading(text: &str) -> impl Widget + '_ {
    move |ui: &mut Ui| {
        let fill = ui.visuals().code_bg_color;
        Frame::group(ui.style())
            .fill(fill)
            .stroke(Stroke::new(0., Color32::default()))
            .show(ui, |ui| ui.heading(text))
            .response
    }
}

pub fn folder_picker<'a>(value: &'a mut String, placeholder: &'a str) -> impl Widget + 'a {
    move |ui: &mut Ui| {
        ui.horizontal(|ui| {
            TextEdit::singleline(value).hint_text(placeholder).show(ui);
            if ui.button("Open").clicked() {
                if let Some(path) = FileDialog::new().pick_folder() {
                    *value = path.to_string_lossy().to_string();
                }
            }
        })
        .response
    }
}
