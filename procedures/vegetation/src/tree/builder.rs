use bevy::prelude::*;
use chunk::cascade::CascadeChunk;
use comproc::{
	complex::chain::ball_stick::{
		builder::{BallStick, BallStickBuilder},
		render::{mesh_handle_stack::MeshHandleStackSpawner, BallStickRenderItem},
	},
	noise::config::NoiseConfig,
};
use noise::{NoiseFn, Seedable};
use render_item::{
	mesh::{cache::handle::map::HandleMap, handle::MeshHandle, IdentifiedMesh, MeshBuilder},
	NormalizeChunk, RenderItem,
};
use std::fmt::Debug;

pub trait MeshFromTreeNum: MeshBuilder + NormalizeChunk + IdentifiedMesh {
	fn from_tree_num(tree_num: f32) -> Self;
}

#[derive(Component, Debug, Clone)]
pub struct Tree<
	BallMesh: MeshFromTreeNum,
	StickMesh: MeshFromTreeNum,
	LeafMesh: MeshFromTreeNum,
	StickMaterial: Material,
	LeafMaterial: Material,
> {
	anchor: Vec3,
	height: f32,
	stick_material: MeshMaterial3d<StickMaterial>,
	leaf_material: MeshMaterial3d<LeafMaterial>,
	trunk_meshes: Vec<MeshHandle<StickMesh>>,
	branch_ball_sticks: Vec<BallStick>,
	branch_spawner: MeshHandleStackSpawner<BallMesh, StickMesh, StickMaterial>,
	leaf_spawner: MeshHandleStackSpawner<LeafMesh, LeafMesh, LeafMaterial>,
}

pub struct TreeBuilder<
	BallMesh: MeshFromTreeNum,
	StickMesh: MeshFromTreeNum,
	LeafMesh: MeshFromTreeNum,
	N: NoiseFn<f64, 4> + Seedable + Debug + Clone,
	M: NoiseFn<f64, 3> + Seedable + Debug + Clone,
	StickMaterial: Material,
	LeafMaterial: Material,
> {
	pub anchor: Vec3,
	pub height: f32,
	pub variety: u32,
	pub branch_count: usize,
	pub leaf_ball_scale: Vec3,
	pub noise_config_3d: NoiseConfig<3, M>,
	pub noise_config_4d: NoiseConfig<4, N>,
	pub ball_cache: HandleMap<BallMesh>,
	pub stick_cache: HandleMap<StickMesh>,
	pub leaf_cache: HandleMap<LeafMesh>,
	pub stick_material: MeshMaterial3d<StickMaterial>,
	pub leaf_material: MeshMaterial3d<LeafMaterial>,
}

impl<
		BallMesh: MeshFromTreeNum,
		StickMesh: MeshFromTreeNum,
		LeafMesh: MeshFromTreeNum,
		StickMaterial: Material,
		LeafMaterial: Material,
		N: NoiseFn<f64, 4> + Seedable + Debug + Clone,
		M: NoiseFn<f64, 3> + Seedable + Debug + Clone,
	> TreeBuilder<BallMesh, StickMesh, LeafMesh, N, M, StickMaterial, LeafMaterial>
{
	pub fn get_branch_height(&self, last_position: Vec3) -> f32 {
		let noise_value = self.noise_config_3d.vec3_on_unit(last_position) as f32;
		noise_value * self.height
	}

	pub fn branch_builder(&self, anchor: Vec3, initial_ray: Vec3) -> BallStickBuilder<N, M> {
		BallStickBuilder::common_tree_builder()
			.with_anchor(anchor)
			.with_initial_ray(initial_ray)
			.with_bias_ray(initial_ray + Vec3::new(0.0, 0.01, 0.0))
			.with_bias_amount(0.2)
			.with_angle_tolerance(2.0)
			.with_splitting_coefficient(0.6)
			.with_min_segment_length(0.002)
			.with_max_segment_length(0.01)
			.with_min_radius(0.001)
			.with_max_radius(0.002)
			.with_depth(4)
			.with_noise_config_3d(self.noise_config_3d.clone())
			.with_noise_config_4d(self.noise_config_4d.clone())
	}

	pub fn compute_radial_branches(&self) -> Vec<BallStick> {
		let mut branches = Vec::new();
		let pre_height = self.get_branch_height(self.anchor);
		let mut last_position = self.anchor + Vec3::new(0.0, pre_height, 0.0);

		for i in 0..self.branch_count {
			let height = self.get_branch_height(last_position);
			let angle = i as f32 * 2.0 * std::f32::consts::PI / self.branch_count as f32;
			let initial_ray =
				Vec3::new(angle.cos(), angle.sin() + angle.cos(), angle.sin()).normalize();

			let branch_builder = self.branch_builder(last_position, initial_ray);
			let branch = branch_builder.build();
			branches.push(branch);

			last_position = last_position + Vec3::new(0.0, height, 0.0);
		}

		branches
	}

	pub fn tree_num(&self) -> f32 {
		self.noise_config_3d.vec3_on_unit(self.anchor) as f32
	}

	pub fn build(self) -> Tree<BallMesh, StickMesh, LeafMesh, StickMaterial, LeafMaterial> {
		let branch_ball_sticks = self.compute_radial_branches();
		let tree_num = self.tree_num();

		let stick_meshes: Vec<MeshHandle<StickMesh>> = (0..self.variety)
			.map(|i| {
				MeshHandle::new(StickMesh::from_tree_num(tree_num + i as f32))
					.with_handle_cache(self.stick_cache.clone())
			})
			.collect();

		let ball_meshes: Vec<MeshHandle<BallMesh>> = (0..self.variety)
			.map(|i| {
				MeshHandle::new(BallMesh::from_tree_num(tree_num + i as f32))
					.with_handle_cache(self.ball_cache.clone())
			})
			.collect();

		let leaf_meshes: Vec<MeshHandle<LeafMesh>> = (0..self.variety)
			.map(|i| {
				MeshHandle::new(LeafMesh::from_tree_num(tree_num + i as f32))
					.with_handle_cache(self.leaf_cache.clone())
			})
			.collect();

		let branch_spawner =
			MeshHandleStackSpawner::new(self.stick_material.clone(), self.stick_material.clone())
				.with_stick_mesh_handle_stack(stick_meshes.clone())
				.with_ball_mesh_handle_stack(ball_meshes.clone());

		let leaf_spawner =
			MeshHandleStackSpawner::new(self.leaf_material.clone(), self.leaf_material.clone())
				.with_ball_mesh_handle_stack(leaf_meshes.clone())
				.with_ball_scale(self.leaf_ball_scale);

		Tree {
			anchor: self.anchor,
			height: self.height,
			stick_material: self.stick_material,
			leaf_material: self.leaf_material,
			trunk_meshes: stick_meshes.clone(),
			branch_ball_sticks,
			branch_spawner,
			leaf_spawner,
		}
	}
}
