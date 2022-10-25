use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Deserialize, Serialize, Clone)]
pub struct Behavior {
    pub name: String,
    pub nodes: Vec<BehaviorNode>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct BehaviorNode {
    #[serde(rename = "type")]
    pub type_: String,
    pub name: String,
    pub params: HashMap<String, String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ActorGroup {
    pub size: i32,
    pub behavior: String,
}

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct Actors {
    behaviors: BTreeMap<String, Behavior>,
    groups: BTreeMap<String, ActorGroup>,
}
