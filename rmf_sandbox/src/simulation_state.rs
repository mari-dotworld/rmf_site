use derivative::Derivative;

#[derive(Derivative)]
#[derivative(Default)]
pub struct SimulationState {
    #[derivative(Default(value="true"))]
    pub paused: bool,
}
