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

use crate::site::*;
use crate::{interaction::CameraControls, CurrentWorkspace};
use bevy::prelude::*;

pub fn update_level_visibility(
    mut levels: Query<(Entity, &mut Visibility), With<LevelElevation>>,
    current_level: Res<CurrentLevel>,
) {
    if current_level.is_changed() {
        for (e, mut visibility) in &mut levels {
            *visibility = if Some(e) == **current_level {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}

pub fn assign_orphan_levels_to_site(
    mut commands: Commands,
    new_levels: Query<Entity, (Without<Parent>, Added<LevelElevation>)>,
    open_sites: Query<Entity, With<NameOfSite>>,
    current_workspace: Res<CurrentWorkspace>,
) {
    if let Some(site) = current_workspace.to_site(&open_sites) {
        for level in &new_levels {
            commands.entity(site).add_child(level);
        }
    } else {
        warn!(
            "Unable to assign level to any site because there is no \
            current site"
        );
    }
}

pub fn assign_orphan_elements_to_level<T: Component>(
    mut commands: Commands,
    orphan_elements: Query<Entity, (With<T>, Without<Parent>)>,
    current_level: Res<CurrentLevel>,
) {
    let current_level = match current_level.0 {
        Some(c) => c,
        None => return,
    };

    for orphan in &orphan_elements {
        commands.entity(current_level).add_child(orphan);
    }
}

pub fn set_camera_transform_on_level_change(
    current_level: Res<CurrentLevel>,
    mut camera_controls: ResMut<CameraControls>,
    camera_poses: Query<&CameraPoses>,
    mut transforms: Query<&mut Transform>,
) {
    if current_level.is_changed() {
        let Some(level) = current_level.0 else {
            return;
        };

        if let Ok(poses) = camera_poses.get(level) {
            // TODO(luca) Add an actual default pose rather than first in map
            let Some(pose) = poses.0.values().next() else {
                return;
            };
            if let Ok(mut tf) = transforms.get_mut(camera_controls.perspective_camera_entities[0]) {
                *tf = pose.transform();
            }
            let mut translation = pose.transform().translation;
            // TODO(luca) these are the same value that are in rmf_site_format, should we change
            // them?
            translation.x = translation.x + 10.0;
            translation.y = translation.y + 10.0;
            translation.z = 0.0;
            camera_controls.orbit_center = translation;
        }
    }
}
