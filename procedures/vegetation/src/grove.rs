use crate::tree::meshes::canopy::ball::NoisyBall;
use crate::tree::meshes::trunk::segment::SimpleTrunkSegment;
use crate::tree::TreeRenderItem;
use bevy::prelude::*;
use chunk::cascade::CascadeChunk;
use render_item::mesh::cache::handle::map::HandleMap;
use render_item::RenderItem;

use noise::{NoiseFn, Perlin};

#[derive(Debug, Clone)]
pub struct NoiseConfig {
	scale: f32,
	noise: Perlin,
}

impl Default for NoiseConfig {
	fn default() -> Self {
		Self { scale: 10.0, noise: Perlin::new(42) }
	}
}

impl NoiseConfig {
	pub fn get(&self, position: Vec3) -> f32 {
		self.noise.get([
			position.x as f64 * self.scale as f64,
			position.y as f64 * self.scale as f64,
			position.z as f64 * self.scale as f64,
		]) as f32
	}

	pub fn get_on_unit_interval(&self, position: Vec3) -> f32 {
		let noise = self.get(position);
		log::info!("Noise: {:?}", noise);
		noise * 0.5 + 0.5
	}
}

#[derive(Component, Clone)]
pub struct GroveBuilder<T: Material, L: Material> {
	noise_config: NoiseConfig,
	threshold: f32,
	anchor: Vec3,
	step_size: f32,
	count: usize,
	trunk_material: MeshMaterial3d<T>,
	leaf_material: MeshMaterial3d<L>,
	tree_cache: HandleMap<SimpleTrunkSegment>,
	leaf_cache: HandleMap<NoisyBall>,
}

impl<T: Material, L: Material> GroveBuilder<T, L> {
	pub fn new(trunk_material: MeshMaterial3d<T>, leaf_material: MeshMaterial3d<L>) -> Self {
		Self {
			noise_config: NoiseConfig::default(),
			threshold: 0.5,
			anchor: Vec3::ZERO,
			step_size: 4.0,
			count: 64,
			trunk_material,
			leaf_material,
			tree_cache: HandleMap::new(),
			leaf_cache: HandleMap::new(),
		}
	}

	pub fn with_tree_cache(mut self, tree_cache: HandleMap<SimpleTrunkSegment>) -> Self {
		self.tree_cache = tree_cache;
		self
	}

	pub fn with_leaf_cache(mut self, leaf_cache: HandleMap<NoisyBall>) -> Self {
		self.leaf_cache = leaf_cache;
		self
	}

	pub fn meets_threshold(&self, position: Vec3) -> bool {
		let noise = self.noise_config.get_on_unit_interval(position);
		log::info!("Noise: {:?}", noise);
		noise > self.threshold
	}

	pub fn inner_noise(&self, position: Vec3) -> f32 {
		self.noise_config.get(position) * self.step_size / 2.0
	}

	pub fn build(&self) -> Grove<T, L> {
		let mut trees = Vec::new();
		for i in 0..self.count {
			for j in 0..self.count {
				let pre_position = self.anchor
					+ Vec3::new(i as f32 * self.step_size, 0.0, j as f32 * self.step_size);

				let position = pre_position
					+ Vec3::new(
						self.inner_noise(pre_position),
						0.0,
						self.inner_noise(pre_position),
					);

				log::info!("Position: {:?}", position);

				if self.meets_threshold(position) {
					log::info!("Meets threshold");
					let tree = TreeRenderItem::new(
						self.trunk_material.clone(),
						self.leaf_material.clone(),
					)
					.with_tree_cache(self.tree_cache.clone())
					.with_leaf_cache(self.leaf_cache.clone());
					trees.push((position, tree));
				}
			}
		}
		Grove { trees }
	}
}

#[derive(Component, Clone)]
pub struct Grove<T: Material, L: Material> {
	trees: Vec<(Vec3, TreeRenderItem<T, L>)>,
}

impl<T: Material, L: Material> RenderItem for Grove<T, L> {
	fn spawn_render_items(
		&self,
		commands: &mut Commands,
		cascade_chunk: &CascadeChunk,
		transform: Transform,
	) -> Vec<Entity> {
		let mut entities = Vec::new();
		for (position, tree) in &self.trees {
			let transform = transform.with_translation(*position);
			entities.extend(tree.spawn_render_items(commands, cascade_chunk, transform));
		}
		entities
	}
}
