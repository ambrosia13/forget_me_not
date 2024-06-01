use bevy_ecs::schedule::ScheduleLabel;

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub struct RenderInitSchedule;

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub struct RenderUpdateSchedule;
