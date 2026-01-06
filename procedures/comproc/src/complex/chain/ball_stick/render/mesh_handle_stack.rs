use crate::complex::chain::ball_stick::builder::{BallStickNode, BallStickSegment};
use crate::complex::chain::ball_stick::render::BallStickSpawner;
use bevy::prelude::*;
use chunk::cascade::CascadeChunk;
use render_item::mesh::{handle::MeshHandle, IdentifiedMesh, MeshBuilder, MeshDispatch};

#[derive(Component, Debug, Clone)]
pub struct MeshHandleStackSpawner<
	B: MeshBuilder + IdentifiedMesh,
	S: MeshBuilder + IdentifiedMesh,
	M: Material,
> {
	pub ball_mesh_handle_stack: Vec<MeshHandle<B>>,
	pub ball_material: MeshMaterial3d<M>,
	pub ball_scale: Vec3,
	pub stick_mesh_handle_stack: Vec<MeshHandle<S>>,
	pub stick_material: MeshMaterial3d<M>,
	pub stick_scale: Vec3,
}

impl<B: MeshBuilder + IdentifiedMesh, S: MeshBuilder + IdentifiedMesh, M: Material>
	MeshHandleStackSpawner<B, S, M>
{
	pub fn new(ball_material: MeshMaterial3d<M>, stick_material: MeshMaterial3d<M>) -> Self {
		Self {
			ball_mesh_handle_stack: vec![],
			ball_material,
			ball_scale: Vec3::splat(0.5),
			stick_mesh_handle_stack: vec![],
			stick_scale: Vec3::splat(1.0),
			stick_material,
		}
	}

	pub fn with_ball_mesh_handle_stack(
		mut self,
		ball_mesh_handle_stack: Vec<MeshHandle<B>>,
	) -> Self {
		self.ball_mesh_handle_stack = ball_mesh_handle_stack;
		self
	}

	pub fn with_stick_mesh_handle_stack(
		mut self,
		stick_mesh_handle_stack: Vec<MeshHandle<S>>,
	) -> Self {
		self.stick_mesh_handle_stack = stick_mesh_handle_stack;
		self
	}

	pub fn with_ball_scale(mut self, ball_scale: Vec3) -> Self {
		self.ball_scale = ball_scale;
		self
	}

	pub fn with_stick_scale(mut self, stick_scale: Vec3) -> Self {
		self.stick_scale = stick_scale;
		self
	}

	pub fn get_ball_for_index(&self, index: usize) -> Option<MeshHandle<B>> {
		if self.ball_mesh_handle_stack.is_empty() {
			return None;
		}
		Some(self.ball_mesh_handle_stack[index % self.ball_mesh_handle_stack.len()].clone())
	}

	pub fn get_stick_for_index(&self, index: usize) -> Option<MeshHandle<S>> {
		if self.stick_mesh_handle_stack.is_empty() {
			return None;
		}
		Some(self.stick_mesh_handle_stack[index % self.stick_mesh_handle_stack.len()].clone())
	}
}

impl<B: MeshBuilder + IdentifiedMesh, S: MeshBuilder + IdentifiedMesh, M: Material> BallStickSpawner
	for MeshHandleStackSpawner<B, S, M>
where
	(
		CascadeChunk,
		MeshDispatch<MeshHandle<B>>,
		bevy::prelude::Transform,
		bevy::prelude::MeshMaterial3d<M>,
	): Bundle,
	(
		CascadeChunk,
		MeshDispatch<MeshHandle<S>>,
		bevy::prelude::Transform,
		bevy::prelude::MeshMaterial3d<M>,
	): Bundle,
	// todo: not really sure why we need to restrict the types, like this, but otherwise we get complaints about [MeshDispatch<MeshHandle<B>>] not being a bundle
{
	fn spawn_ball(
		&self,
		commands: &mut Commands,
		_transform: Transform,
		cascade_chunk: &CascadeChunk,
		node: &BallStickNode,
		index: usize,
	) -> Vec<Entity> {
		if let Some(mesh_handle) = self.get_ball_for_index(index) {
			let scale = self.ball_scale;

			// spawn one on the point
			let ball_transform = Transform::from_translation(node.position).with_scale(scale); // Scale for leaf ball size
			commands.spawn((
				cascade_chunk.clone(),
				MeshDispatch::new(mesh_handle.clone()),
				ball_transform,
				MeshMaterial3d(self.ball_material.0.clone()),
			));

			vec![]
		} else {
			vec![]
		}
	}

	fn spawn_stick(
		&self,
		commands: &mut Commands,
		_transform: Transform,
		cascade_chunk: &CascadeChunk,
		segment: &BallStickSegment,
		index: usize,
	) -> Vec<Entity> {
		if let Some(mesh_handle) = self.get_stick_for_index(index) {
			let ray = segment.ray();
			let direction = ray.clone().normalize();
			let length = ray.length();

			let up = direction;

			// Pick a reference axis that is NOT parallel
			let reference = if up.abs_diff_eq(Vec3::Y, 1e-4) { Vec3::X } else { Vec3::Y };

			let right = up.cross(reference).normalize();
			let forward = right.cross(up);

			let rotation = Quat::from_mat3(&Mat3::from_cols(right, up, forward));

			let pivot_offset = Vec3::new(0.5, 0.0, 0.5);
			let scale =
				Vec3::new(segment.start.radius, length, segment.start.radius) * self.stick_scale;

			let transform = Transform {
				translation: segment.start.position - rotation * (pivot_offset * scale),
				rotation,
				scale,
			};

			commands.spawn((
				cascade_chunk.clone(),
				MeshDispatch::new(mesh_handle.clone()),
				transform,
				MeshMaterial3d(self.stick_material.0.clone()),
			));

			vec![]
		} else {
			vec![]
		}
	}
}
