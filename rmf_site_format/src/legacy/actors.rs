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

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Behavior {
    pub nodes: Vec<BehaviorNode>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BehaviorNode {
    #[serde(rename = "type")]
    pub type_: String,
    pub name: String,
    #[serde(default)]
    pub params: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ActorGroup {
    pub size: i32,
    pub behavior: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Actors {
    pub behaviors: BTreeMap<String, Behavior>,
    pub groups: BTreeMap<String, ActorGroup>,
}
