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

use crate::{Anchor, Angle, AssetSource, Category, Level, NameInSite, Pose, Rotation, Site};
use sdformat_rs::*;
use std::collections::{hash_map::Entry, HashMap};

#[derive(Debug)]
pub enum SdfConversionError {
    /// An asset that can't be converted to an sdf world was found.
    UnsupportedAssetType,
    /// Entity referenced a non existing anchor.
    BrokenAnchorReference,
}

impl AssetSource {
    fn to_sdf(&self) -> Result<String, SdfConversionError> {
        // TODO(luca) check this function
        match self {
            AssetSource::Local(path) => Ok(path.clone()),
            AssetSource::Remote(name) => Ok(name.clone()),
            AssetSource::Search(name) => {
                let name = name
                    .rsplit("/")
                    .next()
                    .ok_or(SdfConversionError::UnsupportedAssetType)?;
                Ok("model://".to_owned() + name)
            }
            AssetSource::Bundled(_) | AssetSource::Package(_) | AssetSource::OSMTile { .. } => {
                Err(SdfConversionError::UnsupportedAssetType)
            }
        }
    }
}

impl Pose {
    fn to_sdf(&self, elevation: f32) -> SdfPose {
        let p = &self.trans;
        let r = match self.rot {
            Rotation::Yaw(angle) => format!("0 0 {}", angle.radians()),
            Rotation::EulerExtrinsicXYZ(rpy) => format!(
                "{} {} {}",
                rpy[0].radians(),
                rpy[1].radians(),
                rpy[2].radians()
            ),
            Rotation::Quat(quat) => format!("{} {} {} {}", quat[0], quat[1], quat[2], quat[3]),
        };
        SdfPose {
            data: format!("{} {} {} {}", p[0], p[1], p[2] + elevation, r),
            ..Default::default()
        }
    }
}

impl NameInSite {
    fn to_sdf(&self, model_counts: &mut HashMap<String, i64>) -> String {
        match model_counts.entry(self.0.to_string()) {
            Entry::Occupied(mut entry) => {
                let name = format!("{}_{}", self.0, entry.get());
                *entry.get_mut() += 1;
                name
            }
            Entry::Vacant(entry) => {
                entry.insert(1);
                self.0.clone()
            }
        }
    }
}

impl Site {
    pub fn to_sdf(&self) -> Result<SdfRoot, SdfConversionError> {
        let get_anchor = |id: u32, level: &Level, site: &Site| -> Option<Anchor> {
            level
                .anchors
                .get(&id)
                .or_else(|| self.anchors.get(&id))
                .cloned()
        };
        //let mut levels = Vec::new();
        let mut includes = Vec::new();
        let mut models = Vec::new();
        // Models must have a unique name, use this to add a counter
        let mut model_counts = HashMap::new();
        let mut floor_count = 0;
        let mut wall_count = 0;
        let floor_thickness = 0.01_f32;
        let wall_thickness = 0.01;
        let wall_height = 2.5;
        for level in self.levels.values() {
            let z = level.properties.elevation.0;
            // TODO(luca) meshes for floor, walls,
            for model in level.models.values() {
                let source = model.source.to_sdf()?;
                includes.push(SdfWorldInclude {
                    name: Some(model.name.to_sdf(&mut model_counts)),
                    uri: source,
                    pose: Some(model.pose.to_sdf(z)),
                    r#static: Some(model.is_static.0),
                    ..Default::default()
                })
            }
            for floor in level.floors.values() {
                // TODO(luca) materials for floors
                floor_count += 1;
                let anchors = floor
                    .anchors
                    .0
                    .iter()
                    .map(|id| {
                        let anchor = get_anchor(*id, level, self)
                            .ok_or(SdfConversionError::BrokenAnchorReference)?;
                        let pose = anchor.translation_for_category(Category::General);
                        Ok(format!("{} {}", pose[0], pose[1]))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let geometry = SdfGeometry::Polyline(SdfPolylineShape {
                    point: anchors,
                    height: floor_thickness as f64,
                });
                models.push(SdfModel {
                    name: format!("Floor_{}", floor_count),
                    r#static: Some(true),
                    pose: Some(Pose::default().to_sdf(z - floor_thickness / 2.0)),
                    link: vec![SdfLink {
                        name: format!("Floor_{}", floor_count),
                        collision: vec![SdfCollision {
                            name: "collision".into(),
                            geometry: geometry.clone(),
                            ..Default::default()
                        }],
                        visual: vec![SdfVisual {
                            name: "visual".into(),
                            geometry,
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                })
            }
            for wall in level.walls.values() {
                wall_count += 1;
                // TODO(luca) materials for walls
                let start = get_anchor(wall.anchors.start(), level, self)
                    .ok_or(SdfConversionError::BrokenAnchorReference)?;
                let end = get_anchor(wall.anchors.end(), level, self)
                    .ok_or(SdfConversionError::BrokenAnchorReference)?;
                let start = start.translation_for_category(Category::General);
                let end = end.translation_for_category(Category::General);
                let length = ((start[0] - end[0]).powi(2) + (start[1] - end[1]).powi(2)).sqrt();
                let geometry = SdfGeometry::r#Box(SdfBoxShape {
                    size: Vector3d::new(length.into(), wall_thickness, wall_height),
                });
                let pose = Pose {
                    trans: Default::default(),
                    rot: Rotation::Yaw(Angle::Rad((start[0] - end[0]).atan2(start[1] - end[1]))),
                };
                models.push(SdfModel {
                    name: format!("Wall_{}", wall_count),
                    r#static: Some(true),
                    pose: Some(pose.to_sdf(0.0)),
                    link: vec![SdfLink {
                        name: format!("Wall_{}", wall_count),
                        collision: vec![SdfCollision {
                            name: "collision".into(),
                            geometry: geometry.clone(),
                            ..Default::default()
                        }],
                        visual: vec![SdfVisual {
                            name: "visual".into(),
                            geometry,
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                })
            }
        }

        Ok(SdfRoot {
            version: "1.7".to_string(),
            world: vec![SdfWorld {
                name: self.properties.name.0.clone(),
                include: includes,
                model: models,
                atmosphere: SdfAtmosphere {
                    r#type: "adiabatic".to_string(),
                    ..Default::default()
                },
                scene: SdfScene {
                    ambient: "1 1 1".to_string(),
                    background: "0.8 0.8 0.8".to_string(),
                    ..Default::default()
                },
                ..Default::default()
            }],
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legacy::building_map::BuildingMap;

    #[test]
    fn serde_roundtrip() {
        let data = std::fs::read("../assets/demo_maps/hotel.building.yaml").unwrap();
        let map = BuildingMap::from_bytes(&data).unwrap();
        let site = map.to_site().unwrap();
        // Convert to an sdf
        let sdf = site.to_sdf().unwrap();
        dbg!(&sdf);
        let config = yaserde::ser::Config {
            perform_indent: true,
            write_document_declaration: true,
            ..Default::default()
        };
        let s = yaserde::ser::to_string_with_config(&sdf, &config).unwrap();
        println!("{}", s);
        std::fs::write("test.sdf", s);
        panic!();
    }
}