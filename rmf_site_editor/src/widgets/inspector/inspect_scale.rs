/*
 * Copyright (C) 2023 Open Source Robotics Foundation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
*/

use bevy_egui::egui::{DragValue, Grid, Ui};
use rmf_site_format::Scale;

pub struct InspectScale<'a> {
    pub scale: &'a Scale,
}

impl<'a> InspectScale<'a> {
    pub fn new(scale: &'a Scale) -> Self {
        Self { scale }
    }

    pub fn show(self, ui: &mut Ui) -> Option<Scale> {
        let mut new_scale = self.scale.clone();
        ui.label("Scale");
        Grid::new("inspect_scale").show(ui, |ui| {
            ui.label("x");
            ui.label("y");
            ui.label("z");
            ui.end_row();

            ui.add(DragValue::new(&mut new_scale.0[0]).speed(0.01));
            ui.add(DragValue::new(&mut new_scale.0[1]).speed(0.01));
            ui.add(DragValue::new(&mut new_scale.0[2]).speed(0.01));
            ui.end_row();
        });
        ui.add_space(5.0);

        if new_scale != *self.scale {
            return Some(new_scale);
        }
        None
    }
}