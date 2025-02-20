use egui::{AboveOrBelow, Grid, PopupCloseBehavior, Response, ScrollArea, Sense, Ui, Widget};

use crate::{DecorationKind, KeyedListModel, KeyedViewItem, ListModel, ViewItem};

pub fn model_drop_down<'a, M, I, DD>(
    model: &'a M,
    decoration_dependencies: &'a DD,
    selected: &'a mut Option<String>,
) -> impl Widget + 'a
where
    M: KeyedListModel<I>,
    I: KeyedViewItem<DecorationDependencies = DD>,
{
    move |ui: &mut Ui| ModelDropDown::default().show(ui, model, decoration_dependencies, selected)
}

pub fn indexed_model_drop_down<'a, M, I, DD>(
    model: &'a M,
    decoration_dependencies: &'a DD,
    selected: &'a mut Option<usize>,
) -> impl Widget + 'a
where
    M: ListModel<I>,
    I: ViewItem<DecorationDependencies = DD>,
{
    move |ui: &mut Ui| {
        ModelDropDown::default().show_indexed(ui, model, decoration_dependencies, selected)
    }
}

fn drop_down_item_ui<M, I, DD>(
    ui: &mut Ui,
    model: &M,
    decoration_dependencies: &DD,
    index: usize,
    selected: bool,
) -> Response
where
    M: ListModel<I>,
    I: ViewItem<DecorationDependencies = DD>,
{
    if let Some(item) = model.item(index) {
        if let Some(item) = model.item(index) {
            item.with_decoration(decoration_dependencies, DecorationKind::DropDown, |image| {
                match image {
                    Some(image) => ui.add(image),
                    None => ui.label(""),
                }
            });
        } else {
            ui.label("");
        }
        item.with_text(|text| ui.selectable_label(selected, text))
    } else {
        // Out of bounds - fill with empty space.
        ui.label("");
        ui.label("")
    }
}

#[derive(Default)]
pub struct ModelDropDown<'a> {
    key_transform: Option<&'a dyn Fn(&str) -> String>,
    key_reverse_transform: Option<&'a dyn Fn(&str) -> String>,
    force_refresh: bool,
}

impl<'a> ModelDropDown<'a> {
    pub fn transform(
        mut self,
        transform: &'a dyn Fn(&str) -> String,
        reverse_transform: &'a dyn Fn(&str) -> String,
    ) -> Self {
        self.key_transform = Some(transform);
        self.key_reverse_transform = Some(reverse_transform);
        self
    }

    pub fn force_refresh(mut self, force_refresh: bool) -> Self {
        self.force_refresh = force_refresh;
        self
    }

    fn show_impl<M, I, DD>(
        &self,
        ui: &mut Ui,
        model: &M,
        decoration_dependencies: &DD,
        selected_index: Option<usize>,
    ) -> (Response, Option<usize>)
    where
        M: ListModel<I>,
        I: ViewItem<DecorationDependencies = DD>,
    {
        let id = ui.auto_id_with("model_combo_box");
        let popup_id = ui.auto_id_with("model_combo_box_popup");
        let mut selection = None;

        let display_text = selected_index
            .and_then(|index| model.item(index))
            .map(|item| item.with_text(|text| text.to_string()))
            .unwrap_or_default();

        let mut search = ui.memory_mut(|mem| {
            if mem.is_popup_open(popup_id) {
                mem.data
                    .get_persisted_mut_or_default::<String>(id)
                    .to_string()
            } else {
                display_text.to_string()
            }
        });

        let background_color = ui.visuals().widgets.open.weak_bg_fill;
        ui.visuals_mut().extreme_bg_color = background_color;

        let text_edit_response = ui.text_edit_singleline(&mut search);
        if text_edit_response.gained_focus() {
            search = Default::default();
            ui.memory_mut(|mem| {
                mem.data.insert_persisted(id, search.clone());
                mem.open_popup(popup_id);
            });
        } else if self.force_refresh {
            search = display_text.to_string();
            ui.memory_mut(|mem| {
                mem.data.insert_persisted(id, search.clone());
            });
        } else if text_edit_response.changed() {
            ui.memory_mut(|mem| {
                mem.data.insert_persisted(id, search.clone());
            });
        }

        ui.reset_style();

        // Copied from egui's ComboBox implementation.
        let above_or_below = if ui.next_widget_position().y + ui.spacing().interact_size.y + 200.0
            < ui.ctx().screen_rect().bottom()
        {
            AboveOrBelow::Below
        } else {
            AboveOrBelow::Above
        };

        egui::popup_above_or_below_widget(
            ui,
            popup_id,
            &text_edit_response,
            above_or_below,
            PopupCloseBehavior::CloseOnClickOutside,
            |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    if I::decorated(DecorationKind::DropDown) {
                        Grid::new(ui.auto_id_with("__model_combo_box_grid"))
                            .num_columns(2)
                            .show(ui, |ui| {
                                for i in 0..model.len() {
                                    if let Some(item) = model.item(i) {
                                        item.with_text(|text| {
                                            if search.is_empty() || text.contains(&search) {
                                                let response = drop_down_item_ui(
                                                    ui,
                                                    model,
                                                    decoration_dependencies,
                                                    i,
                                                    Some(i) == selected_index,
                                                );
                                                ui.end_row();
                                                if response.clicked() {
                                                    selection = Some(i);
                                                    ui.memory_mut(|mem| {
                                                        mem.data
                                                            .insert_persisted(id, text.to_string());
                                                        mem.close_popup();
                                                    });
                                                }
                                            }
                                        });
                                    }
                                }
                            });
                    } else {
                        for i in 0..model.len() {
                            if let Some(item) = model.item(i) {
                                item.with_text(|text| {
                                    if search.is_empty() || text.contains(&search) {
                                        ui.vertical(|ui| {
                                            if ui
                                                .selectable_label(Some(i) == selected_index, text)
                                                .clicked()
                                            {
                                                selection = Some(i);
                                                ui.memory_mut(|mem| {
                                                    mem.data.insert_persisted(id, text.to_string());
                                                    mem.close_popup();
                                                });
                                            }
                                        });
                                    }
                                });
                            }
                        }
                    }
                });
            },
        );

        let mut response = ui.interact(
            text_edit_response.rect,
            id,
            Sense::focusable_noninteractive(),
        );
        if selection.is_some() {
            response.mark_changed();
        }
        (response, selection)
    }

    pub fn show<M, I, DD>(
        self,
        ui: &mut Ui,
        model: &M,
        decoration_dependencies: &DD,
        key: &mut Option<String>,
    ) -> Response
    where
        M: KeyedListModel<I>,
        I: KeyedViewItem<DecorationDependencies = DD>,
    {
        let index = key.as_deref().and_then(|key| match self.key_transform {
            Some(transform) => model.index_of(&transform(key)),
            None => model.index_of(key),
        });

        let (response, selection) = self.show_impl(ui, model, decoration_dependencies, index);
        if let Some(i) = selection {
            if let Some(new_key) = model.item(i).map(|item| item.key()) {
                *key = Some(match self.key_reverse_transform {
                    Some(transform) => transform(&new_key),
                    None => new_key.to_string(),
                });
            }
        }
        response
    }

    pub fn show_indexed<M, I, DD>(
        self,
        ui: &mut Ui,
        model: &M,
        decoration_dependencies: &DD,
        index: &mut Option<usize>,
    ) -> Response
    where
        M: ListModel<I>,
        I: ViewItem<DecorationDependencies = DD>,
    {
        let (response, selection) = self.show_impl(ui, model, decoration_dependencies, *index);
        if let Some(i) = selection {
            *index = Some(i);
        }
        response
    }
}
