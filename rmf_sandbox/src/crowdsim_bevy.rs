use rmf_crowdsim::local_planners::zanlungo::Zanlungo;
use rmf_crowdsim::local_planners::no_local_plan::NoLocalPlan;
use rmf_crowdsim::source_sink::source_sink::{PoissonCrowd, SourceSink};
use rmf_crowdsim::spatial_index::location_hash_2d::LocationHash2D;
use rmf_crowdsim::spatial_index::spatial_index::SpatialIndex;
use rmf_crowdsim::*;

use bevy::ecs::system::SystemParam;
use bevy::{ecs::schedule::ShouldRun, prelude::*};

use crate::simulation_state::SimulationState;

use std::sync::{Arc, Mutex};

use std::collections::VecDeque;

struct NoMap {}

impl Map for NoMap {
    fn get_occupancy(&self, _pt: Point) -> Option<bool> {
        return Some(true);
    }
}

struct StubHighLevelPlan {
    default_vel: Vec2f,
}

impl StubHighLevelPlan {
    fn new(default_vel: Vec2f) -> Self {
        StubHighLevelPlan {
            default_vel: default_vel,
        }
    }
}

impl<M: Map> HighLevelPlanner<M> for StubHighLevelPlan {
    fn get_desired_velocity(
        &mut self,
        _agent: &Agent,
        _time: std::time::Duration,
    ) -> Option<Vec2f> {
        Some(self.default_vel)
    }

    /// Set the target position for a given agent
    fn set_target(&mut self, _agent: &Agent, _point: Point, _tolerance: Vec2f) {
        // For now do nothing
    }
    /// Remove an agent
    fn remove_agent_id(&mut self, _agent: AgentId) {
        // Do nothing
    }

    fn set_map(&mut self, _map: Arc<M>) {
        // Do nothing
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
    pub crowd_simulation: Simulation<NoMap, LocationHash2D>,
    pub event_listener: Arc<Mutex<CrowdEventListener>>
}

impl CrowdSimComponent {

    fn new() -> Self
    {
        let speed = Vec2f::new(1.0f64, 0f64);

        let map = Arc::new(NoMap {});
        let stub_spatial = LocationHash2D::new(1000f64, 1000f64, 20f64, Point::new(-500f64, -500f64));

        let high_level_planner = Arc::new(Mutex::new(StubHighLevelPlan::new(speed)));
        let local_planner = Arc::new(Mutex::new(NoLocalPlan{}));

        let crowd_sim = Simulation::<NoMap, LocationHash2D>::new(map, stub_spatial);
        let event_listener = Arc::new(Mutex::new(CrowdEventListener::new()));

        let crowd_generator = Arc::new(PoissonCrowd::new(0.2f64));

        /// TODO: Keep for testing purpose only
        let source_sink = Arc::new(SourceSink::<NoMap> {
            source: Vec2f::new(0f64, 0f64),
            sink: Vec2f::new(20f64, 0f64),
            radius_sink: 1f64,
            crowd_generator: crowd_generator,
            high_level_planner: high_level_planner,
            local_planner: local_planner,
            agent_eyesight_range: 5f64,
        });

        let mut res = Self {
            crowd_simulation: crowd_sim,
            event_listener: event_listener.clone()
        };

        res.crowd_simulation.add_event_listener(event_listener);
        res.crowd_simulation.add_source_sink(source_sink);

        res
    }
}

#[derive(Default)]
pub struct CrowdSimPlugin;

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
    time: ResMut<Time>
) {
    crowd_sim.crowd_simulation.step(time.delta());
    while let Some(agent) = crowd_sim.event_listener.lock().unwrap().to_add.pop_front() {
        let (agent_id, pos) = agent;
        commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.5 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(pos.x as f32, pos.y as f32, 0.0),
            ..default()
        }).insert(Actor {id: agent_id});
    }

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