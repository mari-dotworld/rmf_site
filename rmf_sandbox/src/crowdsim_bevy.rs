use rmf_crowdsim::local_planners::zanlungo::Zanlungo;
use rmf_crowdsim::local_planners::no_local_plan::NoLocalPlan;
use rmf_crowdsim::source_sink::source_sink::{PoissonCrowd, SourceSink};
use rmf_crowdsim::spatial_index::location_hash_2d::LocationHash2D;
use rmf_crowdsim::spatial_index::spatial_index::SpatialIndex;
use rmf_crowdsim::*;

use mapf::graph::{Graph, Edge};
use mapf::motion::{
    TimePoint,
    Motion,
    r2::{
        Position,
    graph_search::{TimeInvariantExpander},
    direct_travel::DirectTravelHeuristic,
    timed_position::{LineFollow, Waypoint}},
    trajectory::DurationCostCalculator,
    reach::NoReach
};
use mapf::tree::garden::{Garden, Error};
use mapf::motion::Trajectory;

use bevy::ecs::system::SystemParam;
use bevy::{ecs::schedule::ShouldRun, asset::LoadState, prelude::*};

use line_drawing::Bresenham;

use crate::simulation_state::SimulationState;
use crate::spawner::{SiteMapRoot, VerticesManagers};
use crate::traffic_editor::EditableTag;
use crate::vertex::Vertex;
use crate::{building_map::BuildingMap, wall::Wall};
use crate::site_map::SiteMapCurrentLevel;


use std::sync::{Arc, Mutex};

use std::collections::{VecDeque, HashMap, HashSet};

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
struct SparseOccupancyIndex
{
    pub x: i64,
    pub y: i64
}

impl SparseOccupancyIndex
{
    pub fn from_vec2f(vec: &Vec2f, resolution: f64) -> Self
    {
        let res = vec / resolution;

        Self {
            x: res.x as i64,
            y: res.y as i64
        }
    }

    pub fn to_vec2f(&self, resolution: f64) -> Vec2f
    {
        Vec2f::new(self.x as f64 * resolution, self.y as f64 * resolution)
    }
}

#[derive(Debug, Clone)]
struct SparseOccupancy
{
    grid: HashMap<SparseOccupancyIndex, HashSet<u32>>,
    entity_list: HashMap<u32, Vec<SparseOccupancyIndex>>,
    resolution: f64
}

impl SparseOccupancy
{
    fn new(resolution: f64) -> Self {
        Self {
            grid: HashMap::new(),
            entity_list: HashMap::new(),
            resolution
        }
    }

    fn mark(&mut self, index: &SparseOccupancyIndex, id: &u32) {
        if let Some(set) = self.grid.get_mut(index) {
            set.insert(*id);
        }
        else {
            self.grid.insert(*index, HashSet::from([*id]));
        }
    }

    fn check_in_bounds(&self, index: &SparseOccupancyIndex) -> bool {
        // For now don't use the floor, just keep the radius limited to 20m
        index.to_vec2f(self.resolution).norm() < 20f64
    }

    fn get_occupancy_from_idx(&self, index: &SparseOccupancyIndex) -> Option<bool> {
        if let Some(s) = self.grid.get(index)
        {
            return Some(s.len() > 0);
        }

        if (self.check_in_bounds(index))
        {
            // TODO: Consider floor
            return Some(false);
        }
        return None;
    }

    /// Add or update a wall to the map.
    pub fn add_or_update_wall(&mut self, id: &u32, start: &Vec2f, end: &Vec2f)
    {
        let start = SparseOccupancyIndex::from_vec2f(start, self.resolution);
        let end = SparseOccupancyIndex::from_vec2f(end, self.resolution);

        if let Some(points) = self.entity_list.get_mut(id) {
            points.iter().map(|i| {
                self.grid.get_mut(&i).unwrap().remove(id);
            });
            points.clear();
        }

        let mut occupancy = vec!();
        for (x, y) in Bresenham::new((start.x, start.y), (end.x, end.y)) {
            let idx = SparseOccupancyIndex{x,y};
            self.mark(&idx, id);
            occupancy.push(idx);
        }
        self.entity_list.insert(*id, occupancy);

        println!("Adding wall {}", id);
    }
}

impl Map for SparseOccupancy {
    fn get_occupancy(&self, pt: Point) -> Option<bool>
    {
        let index = SparseOccupancyIndex::from_vec2f(&pt, self.resolution);
        self.get_occupancy_from_idx(&index)
    }
}

impl Edge<SparseOccupancyIndex> for (SparseOccupancyIndex, SparseOccupancyIndex) {
    fn from_vertex(&self) -> &SparseOccupancyIndex {
        &self.0
    }

    fn to_vertex(&self) -> &SparseOccupancyIndex {
        &self.1
    }
}

impl Graph for SparseOccupancy {
    type Key = SparseOccupancyIndex;
    type Vertex = Position;
    type Edge = (SparseOccupancyIndex, SparseOccupancyIndex);

    type EdgeIter<'a> where Self: 'a = impl Iterator<Item=(SparseOccupancyIndex, SparseOccupancyIndex)> + 'a;

    fn vertex (&self, key: Self::Key) -> Option<Position> {
        //self.vertices.get(key).cloned()
        if self.check_in_bounds(&key)
        {
            let pos = key.to_vec2f(self.resolution);
            return Some(Position::new(pos.x, pos.y));
        }
        return None;
    }

    fn edges_from_vertex<'a>(&'a self, from_key: Self::Key) -> Self::EdgeIter<'a> {
        // Iterate through possible routes
        let from_key_copy = from_key;
        [(-1, 1),   (0, 1),   (1,1),
         (-1, 0), /*(0, 0),*/ (1,0),
         (-1,-1),   (0,-1),  (1,-1)].into_iter().filter(move |(dx, dy)|
            {
                let idx = SparseOccupancyIndex{
                    x: from_key.x + dx,
                    y: from_key.y + dy
                };
                if let Some(s) = self.get_occupancy_from_idx(&idx)
                {
                    return !s;
                }
                return false;
            }
        ).map(move |(dx,dy)|
            {
                let idx = SparseOccupancyIndex{
                    x: from_key_copy.x + dx,
                    y: from_key_copy.y + dy
                };
                return (from_key_copy, idx)
            }
        )
    }
}

type DefaultExpander = TimeInvariantExpander<SparseOccupancy, DurationCostCalculator, DirectTravelHeuristic<SparseOccupancy, DurationCostCalculator>>;

fn make_default_expander(
    graph: Arc<SparseOccupancy>,
    extrapolator: Arc<LineFollow>,
) -> DefaultExpander {
    let cost_calculator = Arc::new(DurationCostCalculator);
    let heuristic = Arc::new(DirectTravelHeuristic{
        graph: graph.clone(),
        cost_calculator: cost_calculator.clone(),
        extrapolator: (*extrapolator).clone(),
    });

    DefaultExpander{
        graph, extrapolator, cost_calculator, heuristic, reacher: Arc::new(NoReach)
    }
}

trait Plannable {
    fn plan(&self, source: &Vec2f, target: &Vec2f) -> Option<Trajectory<Waypoint>>;
       // -> Result<Option<Solution>, Error<E, Z>>;
}

impl Plannable for SparseOccupancy where
{
    fn plan(&self, source: &Vec2f, target: &Vec2f) -> Option<Trajectory<Waypoint>>
    {
        let graphref = Arc::new(self.clone());

        let expander = Arc::new(make_default_expander(
            graphref,
            Arc::new(LineFollow::new(1.0f64).unwrap())));

        // TODO: Arjo persist garden
        let garden = Garden::new(expander);

        let solution = garden.solve(
            &SparseOccupancyIndex::from_vec2f(&source, self.resolution),
            &SparseOccupancyIndex::from_vec2f(&target, self.resolution));

        if let Ok(solution) = solution {
            if let Some(solution) = solution {
                return solution.motion().clone();
            }
        }
        return None;
        //return solution;
    }
}

struct StubHighLevelPlan<M: Map> {
    default_vel: Vec2f,
    target: Option<Vec2f>,
    map: Arc<Mutex<M>>,
    solution_cache: HashMap<SparseOccupancyIndex, Vec2f>,
    sample_resolution: f64
}

impl<M: Map> StubHighLevelPlan<M> {
    fn new(default_vel: Vec2f,
        sample_resolution: f64,
        map: Arc<Mutex<M>>) -> Self {
        StubHighLevelPlan {
            default_vel: default_vel,
            target: None,
            map: map,
            solution_cache: HashMap::new(),
            sample_resolution
        }
    }


    fn plan_to_sol_cache(&mut self, trajectory: &Trajectory<Waypoint>) {

        let mut prev_vel: Option<Vec2f> = None;
        for waypoint in trajectory.iter()
        {
            let spatial_index =
                SparseOccupancyIndex::from_vec2f(
                    &Vec2f::new(waypoint.position.x, waypoint.position.y),
                    self.sample_resolution);
            // TODO: whats-up with Point->vec2f translation
            let time = waypoint.time;
            let velocity = trajectory.motion().compute_velocity(&time);
            //let velocity = waypoint.velocity.norm();
            if let Ok(velocity) = velocity {
                if let Some(prev_vel) = prev_vel {
                    self.solution_cache.insert(spatial_index, prev_vel);
                }
                prev_vel = Some(Vec2f::new(velocity.x, velocity.y));
            }
        }
    }

}

impl<M: Map> HighLevelPlanner<M> for StubHighLevelPlan<M> where
    M: Plannable {
    fn get_desired_velocity(
        &mut self,
        agent: &Agent,
        _time: std::time::Duration,
    ) -> Option<Vec2f> {
        if let Some(target) = self.target
        {
            let idx = SparseOccupancyIndex::from_vec2f(&agent.position, self.sample_resolution);
            if let Some(velocity) = self.solution_cache.get(&idx) {
                let dir_vec = velocity;
                return Some(*dir_vec);
            }
            else {
                let plan = self.map.lock().unwrap().plan(&agent.position, &target);
                if let Some(plan) = plan
                {
                    self.plan_to_sol_cache(&plan);
                    let velocity = plan.motion().compute_velocity(&TimePoint {
                        nanos_since_zero: 0
                    });
                    if let Ok(velocity) = velocity {
                        return Some(Vec2f::new(velocity.x, velocity.y));
                    }
                }
            }
        }

        Some(Vec2f::new(0f64, 0f64))
    }

    /// Set the target position for a given agent
    fn set_target(&mut self, agent: &Agent, point: Point, _tolerance: Vec2f) {
        // For now only care about the first target
        if let Some(pt) = self.target {
            return;
        }
        self.target = Some(point);
        let source = agent.position;
        let plan = self.map.lock().unwrap().plan(&source, &point);

        if let Some(plan) = plan
        {
            //let mut prev_vel : Option<Vec2f> = None;
            self.plan_to_sol_cache(&plan);
        }
    }
    /// Remove an agent
    fn remove_agent_id(&mut self, _agent: AgentId) {
        // Do nothing
    }

    fn set_map(&mut self, map: Arc<Mutex<M>>) {
        //println!("Setting map");
        //self.map = map;
    }
}

struct CrowdEventListener {
    pub to_add: VecDeque<(AgentId, Vec2f)>,
    //pub to_remove: VecDeque<AgentId>,
}

impl CrowdEventListener {
    fn new() -> Self {
        Self {
            to_add: VecDeque::new(),
            //to_remove: VecDeque::new()
        }
    }
}

impl EventListener for CrowdEventListener {
    fn agent_spawned(&mut self, position: Vec2f, agent: AgentId) {
        self.to_add.push_back((agent, position));
    }

    /// Called each time an agent is destroyed
    fn agent_destroyed(&mut self, agent: AgentId) {
        //println!("Remove agent");
        //self.to_remove.push_back(agent);
    }
}

#[derive(Component, Default)]
pub struct Actor {
    pub id: AgentId
}

/// The structure of rmf_crowd_sim forces it to be a non-send resource.
struct CrowdSimComponent {
    pub crowd_simulation: Simulation<SparseOccupancy, LocationHash2D>,
    pub event_listener: Arc<Mutex<CrowdEventListener>>,
    pub map: Arc<Mutex<SparseOccupancy>>
}

impl CrowdSimComponent {

    fn new() -> Self
    {
        let speed = Vec2f::new(1.0f64, 0f64);

        // 20cm resolution grid
        let map = Arc::new(Mutex::new(SparseOccupancy::new(0.5)));
        let stub_spatial = LocationHash2D::new(1000f64, 1000f64, 20f64, Point::new(-500f64, -500f64));

        let high_level_planner = Arc::new(Mutex::new(StubHighLevelPlan::new(speed, 0.5, map.clone())));
        let local_planner = Arc::new(Mutex::new(NoLocalPlan{}));

        let crowd_sim = Simulation::<SparseOccupancy, LocationHash2D>::new(map.clone(), stub_spatial);
        let event_listener = Arc::new(Mutex::new(CrowdEventListener::new()));

        let crowd_generator = Arc::new(PoissonCrowd::new(0.2f64));

        /// TODO: Keep for testing purpose only
        let source_sink = Arc::new(SourceSink::<SparseOccupancy> {
            source: Vec2f::new(0f64, 3f64),
            sink: Vec2f::new(0f64, -1f64),
            radius_sink: 1f64,
            crowd_generator: crowd_generator,
            high_level_planner: high_level_planner,
            local_planner: local_planner,
            agent_eyesight_range: 5f64,
        });

        let mut res = Self {
            crowd_simulation: crowd_sim,
            event_listener: event_listener.clone(),
            map: map
        };

        res.crowd_simulation.add_event_listener(event_listener);
        res.crowd_simulation.add_source_sink(source_sink);

        res
    }
}

#[derive(Default)]
pub struct CrowdSimPlugin;

#[derive(Default)]
struct HumanModelLoaded {
    pub scene: Option<Handle<Scene>>,
    pub entities: HashSet<Entity>
}

fn init_model_loader(mut commands: Commands)
{
    commands.insert_resource(HumanModelLoaded::default());
}

fn init_crowd_manager(
    world: &mut World)
{
    world.insert_non_send_resource(CrowdSimComponent::new());
}

fn step_crowd_manager(
    mut commands: Commands,
    mut crowd_sim: NonSendMut<CrowdSimComponent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(Entity, &mut Transform, &Actor)>,
    level: Res<Option<SiteMapCurrentLevel>>,
    vertices_mgrs: Res<VerticesManagers>,
    vertices: Query<(&Vertex, ChangeTrackers<Vertex>)>,
    mut walls: Query<(Entity, &Wall, ChangeTrackers<Wall>)>,
    asset_server: Res<AssetServer>,
    mut loading_models: ResMut<HumanModelLoaded>,
    time: ResMut<Time>
) {
    // Build an obstacle map
    let level = match level.as_ref() {
        Some(level) => level,
        None => {
            return;
        }
    };

    // Update walls
    for (e, wall, change) in walls.iter_mut() {
        let v1_entity = vertices_mgrs.0[&level.0].id_to_entity(wall.0).unwrap();
        let (v1, v1_change) = vertices.get(v1_entity).unwrap();
        let v2_entity = vertices_mgrs.0[&level.0].id_to_entity(wall.1).unwrap();
        let (v2, v2_change) = vertices.get(v2_entity).unwrap();

        if change.is_changed() || v1_change.is_changed() || v2_change.is_changed() {
            let v1_coords = Vec2f::new(v1.0, v1.1);
            let v2_coords = Vec2f::new(v2.0, v2.1);
            crowd_sim.map.lock().unwrap().add_or_update_wall(&e.id(), &v1_coords, &v2_coords);
        }
    }

    // Step the simulation
    crowd_sim.crowd_simulation.step(time.delta());

    /*if loading_models.scene == None
    {
        let bundle_path =
                String::from("sandbox://OfficeChairBlack.glb#Scene0");
        let glb: Handle<Scene> = asset_server.load(&bundle_path);
        loading_models.scene = Some(glb);
    }

    if let Some(h) = loading_models.scene {
        if asset_server.get_load_state(h) == LoadState::Loaded {
            //for e 
        }
    }*/

    // Handle newly spawned agents
    while let Some(agent) = crowd_sim.event_listener.lock().unwrap().to_add.pop_front() {
        let (agent_id, pos) = agent;

        /*commands.spawn_bundle(SpatialBundle {
            transform: Transform::from_xyz(pos.x as f32, pos.y as f32, 0.0),
            ..default()
        }).insert(Actor {id: agent_id});*/
        commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.5 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(pos.x as f32, pos.y as f32, 0.0),
            ..default()
        }).insert(Actor {id: agent_id});
        //loading_models.entities.insert();
    }

    // Set or despawn existing agent's positions
    for (entity, mut transform, actor) in query.iter_mut() {
        let agent_data = crowd_sim.crowd_simulation.agents.get(&actor.id);
        if let Some(agent_data) = agent_data {
            transform.translation = Vec3{
                x: agent_data.position.x as f32, y: agent_data.position.y as f32, z: 0.0};
        }
        else {
            // Agent doesn't exist anymore. Despawn it
            //bevy::log::error!("Agent not found");
            commands.entity(entity).despawn();
        }
    }
}

impl Plugin for CrowdSimPlugin {

    fn build(&self, app: &mut App)
    {
        app.add_startup_system(init_crowd_manager.exclusive_system())
            .add_startup_system(init_model_loader)
            .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_run_criteria(
                    |state: Res<SimulationState>|
                    {
                        if state.paused {
                            return ShouldRun::No;
                        }
                        ShouldRun::Yes
                    }
                )
                .with_system(step_crowd_manager)
            );
    }
}