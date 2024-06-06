/*
 * Copyright (C) 2022 Open Source Robotics Foundation
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

use super::demo_world::*;
use crate::{AppState, LoadWorkspace, WorkspaceData};
use bevy::{app::AppExit, prelude::*, sprite::Anchor, tasks::Task};
use bevy_egui::{egui, EguiContexts};
use rmf_site_format::{Level, Location, LocationTag, NameInSite, NavGraph};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Resource)]
pub struct Autoload {
    pub filename: Option<PathBuf>,
    pub import: Option<PathBuf>,
    pub importing: Option<Task<Option<(Entity, rmf_site_format::Site)>>>,
}

impl Autoload {
    pub fn file(filename: PathBuf, import: Option<PathBuf>) -> Self {
        Autoload {
            filename: Some(filename),
            import,
            importing: None,
        }
    }
}

fn egui_ui(
    mut egui_context: EguiContexts,
    mut _exit: EventWriter<AppExit>,
    mut _load_workspace: EventWriter<LoadWorkspace>,
    mut _app_state: ResMut<State<AppState>>,
    autoload: Option<ResMut<Autoload>>,
) {
    if let Some(mut autoload) = autoload {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(filename) = autoload.filename.clone() {
                _load_workspace.send(LoadWorkspace::Path(filename));
            }
            autoload.filename = None;
        }
        return;
    }

    egui::Window::new("Welcome!")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0., 0.))
        .show(egui_context.ctx_mut(), |ui| {
            ui.heading("Welcome to The RMF Site Editor!");
            ui.add_space(10.);

            ui.horizontal(|ui| {
                if ui.button("View demo map").clicked() {
                    _load_workspace.send(LoadWorkspace::Data(WorkspaceData::LegacyBuilding(
                        demo_office(),
                    )));
                }

                if ui.button("load locations").clicked() {
                    let mut site_id = 0_u32..;
                    let level_id = site_id.next().unwrap();

                    let mut levels = BTreeMap::new();

                    let mut drawings: BTreeMap<u32, rmf_site_format::Drawing> = BTreeMap::new();
                    drawings.insert(
                        level_id,
                        rmf_site_format::Drawing {
                            properties: (rmf_site_format::DrawingProperties {
                                name: NameInSite(("june").to_string()),
                                source: rmf_site_format::AssetSource::Remote(
                                    ("/home/ros1/guru/rmf_site/media/june-1-test.png").to_string(),
                                ),
                                pixels_per_meter: rmf_site_format::PixelsPerMeter(20.0),
                                ..default()
                            }),
                            ..default()
                        },
                    );

                    // 9.114 , -8.9135
                    //  8.791, -8.325

                    let mut anchors = BTreeMap::new();
                    // anchors.insert(site_id.next().unwrap(), [0.0, 0.0].into()); //1
                    // anchors.insert(site_id.next().unwrap(), [0.0, -1.0].into()); //2

                    // let resolution = 0.05000000074505806;
                    // let map_height = 373.0 * resolution;
                    // let origin_x = -9.6096923828125;
                    // let origin_y = -8.197599029541015;

                    // anchors.insert(
                    //     site_id.next().unwrap(),
                    //     [-0.002755206940574878 + -origin_x, -(map_height - -0.2878865198422611 + origin_y)].into(),
                    // );

                    // anchors.insert(
                    //     site_id.next().unwrap(),
                    //     [-1.0626822656723895 + -origin_x, -(map_height - 0.20681487232052476 + origin_y)].into(),
                    // );

                    // anchors.insert(
                    //     site_id.next().unwrap(),
                    //     [-2.75840185353086 + -origin_x, -(map_height -  0.6271782009182569 + origin_y)].into(),
                    // );

                    // anchors.insert(
                    //     site_id.next().unwrap(),
                    //     [-1.6098684664274894 + -origin_x, -(map_height - 0.6585474380100034 + origin_y)].into(),
                    // );

                    // vijay
                    anchors.insert(
                        site_id.next().unwrap(),
                        [
                            ((331.82741765229474 + 5.0) * 0.05),
                            -(((161.3932803576622 + 5.0) * 0.05) - (0.34617525 / 2.0) + (0.14235048) + 0.02),
                        ]
                        .into(),
                    );

                    anchors.insert(
                        site_id.next().unwrap(),
                        [
                            ((293.5692656948175 + 5.0) * 0.05),
                            -(((166.25516386533914 + 5.0) * 0.05) - (0.34617525 / 2.0) + (0.14235048) + 0.02),
                        ]
                        .into(),
                    );

                    anchors.insert(
                        site_id.next().unwrap(),
                        [
                            ((195.28149446847155 + 5.0) * 0.05),
                            -(((176.99880369350066 + 5.0 ) * 0.05 ) - (0.34617525 / 2.0) + (0.14235048) + 0.02)
                        ]
                        .into(),
                    );

                    anchors.insert(
                        site_id.next().unwrap(),
                        [
                            ((177.6069650019961 + 5.0) * 0.05),
                            -(((160.40290695967013 + 5.0 ) * 0.05 ) - (0.34617525 / 2.0) + (0.14235048) + 0.02),
                        ]
                        .into(),
                    );

                    anchors.insert(
                        site_id.next().unwrap(),
                        [
                            ((255.11392445298804 + 5.0) * 0.05),
                            -(((155.801186257567 + 5.0 ) * 0.05 ) - (0.34617525 / 2.0) + (0.14235048) + 0.02)                       ]
                        .into(),
                    );

                    levels.insert(
                        level_id,
                        Level {
                            properties: rmf_site_format::LevelProperties {
                                name: NameInSite("l1".to_string()),
                                ..default()
                            },
                            drawings,
                            anchors,
                            ..default()
                        },
                    );

                    let mut locations = BTreeMap::new();
                    let mut tags = Vec::new();
                    tags.push(LocationTag::Charger);

                    locations.insert(
                        site_id.next().unwrap(),
                        Location {
                            name: NameInSite("one".to_string()),
                            tags: rmf_site_format::LocationTags(tags.clone()),
                            graphs: rmf_site_format::AssociatedGraphs::All,
                            anchor: rmf_site_format::Point(1),
                        },
                    );

                    locations.insert(
                        site_id.next().unwrap(),
                        Location {
                            name: NameInSite("one".to_string()),
                            tags: rmf_site_format::LocationTags(tags.clone()),
                            graphs: rmf_site_format::AssociatedGraphs::All,
                            anchor: rmf_site_format::Point(2),
                        },
                    );

                    locations.insert(
                        site_id.next().unwrap(),
                        Location {
                            name: NameInSite("one".to_string()),
                            tags: rmf_site_format::LocationTags(tags.clone()),
                            graphs: rmf_site_format::AssociatedGraphs::All,
                            anchor: rmf_site_format::Point(3),
                        },
                    );

                    locations.insert(
                        site_id.next().unwrap(),
                        Location {
                            name: NameInSite("one".to_string()),
                            tags: rmf_site_format::LocationTags(tags.clone()),
                            graphs: rmf_site_format::AssociatedGraphs::All,
                            anchor: rmf_site_format::Point(4),
                        },
                    );

                    let mut graphs = BTreeMap::new();
                    graphs.insert(
                        site_id.next().unwrap(),
                        NavGraph {
                            name: NameInSite("navgraph".to_string()),
                            ..default()
                        },
                    );

                    let guided = rmf_site_format::Guided {
                        graphs,
                        locations,
                        ..default()
                    };

                    // create new site and convert to bytes
                    let site = rmf_site_format::Site {
                        levels,
                        navigation: rmf_site_format::Navigation { guided },
                        ..default()
                    };

                    println!("site json data : ->{:?}", site);

                    // convert site to bytes
                    let site_bytes: Vec<u8> = ron::to_string(&site).unwrap().as_bytes().to_vec();

                    _load_workspace.send(LoadWorkspace::Data(WorkspaceData::Site(site_bytes)));
                }
                // TODO(@mxgrey): Bring this back when we have finished developing
                // the key features for workcell editing.
                // if ui.button("Workcell Editor").clicked() {
                //     _load_workspace.send(LoadWorkspace::Data(WorkspaceData::Workcell(
                //         demo_workcell(),
                //     )));
                // }

                // TODO(@mxgrey): Bring this back when we have time to fix the
                // warehouse generator.
                // if ui.button("Warehouse generator").clicked() {
                //     info!("Entering warehouse generator");
                //     _app_state.overwrite_set(AppState::WarehouseGenerator).unwrap();
                // }
            });

            #[cfg(not(target_arch = "wasm32"))]
            {
                ui.add_space(20.);
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Exit").clicked() {
                            _exit.send(AppExit);
                        }
                    });
                });
            }
        });
}

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, egui_ui.run_if(in_state(AppState::MainMenu)));
    }
}
