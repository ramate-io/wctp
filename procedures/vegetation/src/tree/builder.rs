use bevy::prelude::*;
use comproc::{complex::chain::ball_stick::builder::BallStick, noise::config::NoiseConfig};
use noise::{NoiseFn, Seedable};
use std::fmt::Debug;

pub struct Tree {
	anchor: Vec3,
	height: f32,
	branch_ball_sticks: Vec<BallStick>,
}

pub struct TreeBuilder<N: NoiseFn<f64, 3> + Seedable + Debug + Clone> {
	anchor: Vec3,
	height: f32,
	noise_config: Option<NoiseConfig<3, N>>,
}
