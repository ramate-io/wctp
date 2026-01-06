use bevy::prelude::*;
use comproc::{complex::chain::ball_stick::builder::BallStick, noise::config::NoiseConfig};
use noise::{NoiseFn, Seedable};
use render_item::{
	mesh::{IdentifiedMesh, MeshBuilder},
	NormalizeChunk,
};
use std::fmt::Debug;

pub struct Tree<
	StickMesh: MeshBuilder + NormalizeChunk + IdentifiedMesh,
	TrunkMesh: MeshBuilder + NormalizeChunk + IdentifiedMesh,
	LeafMesh: MeshBuilder + NormalizeChunk + IdentifiedMesh,
	TrunkMaterial: Material,
	LeafMaterial: Material,
> {
	anchor: Vec3,
	height: f32,
	trunk_material: MeshMaterial3d<T>,
	leaf_material: MeshMaterial3d<L>,
	branch_ball_sticks: Vec<BallStick>,
	leaf_ball_scale: Vec3,
	trunk_cache: HandleMap<SimpleTrunkSegment>,
	leaf_cache: HandleMap<NoisyBall>,
}

pub struct TreeBuilder<N: NoiseFn<f64, 3> + Seedable + Debug + Clone> {
	anchor: Vec3,
	height: f32,
	noise_config: Option<NoiseConfig<3, N>>,
}
