/// A macro for generating render functions for an enum with minimal boiler plate.
/// 
/// Example invocation:
/// ```
/// enum_combo_box!(my_enum_combo_box, MyEnum,
///     MyEnum::Variant1 => "Variant 1",
///     MyEnum::Variant2 => "Variant 2",
/// );
/// ```
/// 
/// Renderer usage:
/// ```
/// ui.add(my_enum_combo_box(&mut my_enum_value));
/// ```
#[macro_export]
macro_rules! enum_combo_box {
    ($name:ident, $target:ty, $($key:path => $label:expr,)+) => {
        pub fn $name(value: &mut $target) -> impl egui::Widget + '_ {
            move |ui: &mut egui::Ui| {
                {
                    let mut changed = false;
                    let id = ui.auto_id_with("__thane_static_combo");
                    let mut response = egui::ComboBox::from_id_source(id)
                        .width(ui.spacing().text_edit_width)
                        .selected_text(match value {
                            $(
                                $key => $label,
                            )+
                            _ => "",
                        })
                        .show_ui(ui, |ui| {
                            let mut response: Option<egui::Response> = None;
                            $(
                                let value_response = ui.selectable_value(value, $key, $label);
                                changed |= value_response.changed();
                                match response {
                                    Some(r) => response = Some(r.union(value_response)),
                                    None => response = Some(value_response),
                                }
                            )+
                            response.unwrap()
                        })
                        .response;
                    if changed {
                        response.mark_changed();
                    }
                    response
                }
            }
        }
    };
}
