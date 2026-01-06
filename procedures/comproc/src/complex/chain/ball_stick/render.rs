use crate::complex::chain::ball_stick::builder::{BallStick, BallStickNode, BallStickSegment};
use bevy::prelude::*;
use chunk::cascade::CascadeChunk;
use render_item::RenderItem;

pub trait BallStickSpawner {
	/// Computes the appropriate transform for the ball at the given node.
	fn spawn_ball(
		&self,
		commands: &mut Commands,
		transform: Transform,
		cascade_chunk: &CascadeChunk,
		node: &BallStickNode,
	) -> Vec<Entity>;

	/// Computes the appropriate transform for the stick at the given segment.
	fn spawn_stick(
		&self,
		commands: &mut Commands,
		transform: Transform,
		cascade_chunk: &CascadeChunk,
		segment: &BallStickSegment,
	) -> Vec<Entity>;
}

#[derive(Component, Debug, Clone)]
pub struct BallStickRenderItem<P: BallStickSpawner> {
	ballstick: BallStick,
	spawner: P,
}

impl<P: BallStickSpawner> BallStickRenderItem<P> {
	pub fn new(ballstick: BallStick, spawner: P) -> Self {
		Self { ballstick, spawner }
	}

	pub fn with_spawner(mut self, spawner: P) -> Self {
		self.spawner = spawner;
		self
	}

	pub fn with_ballstick(mut self, ballstick: BallStick) -> Self {
		self.ballstick = ballstick;
		self
	}

	pub fn spawn_ball(
		&self,
		commands: &mut Commands,
		transform: Transform,
		cascade_chunk: &CascadeChunk,
		node: &BallStickNode,
	) -> Vec<Entity> {
		self.spawner.spawn_ball(commands, transform, cascade_chunk, node)
	}

	pub fn spawn_stick(
		&self,
		commands: &mut Commands,
		transform: Transform,
		cascade_chunk: &CascadeChunk,
		segment: &BallStickSegment,
	) -> Vec<Entity> {
		self.spawner.spawn_stick(commands, transform, cascade_chunk, segment)
	}

	pub fn into_parts(self) -> (BallStick, P) {
		(self.ballstick, self.spawner)
	}
}

impl<P: BallStickSpawner + Clone> RenderItem for BallStickRenderItem<P> {
	fn spawn_render_items(
		&self,
		commands: &mut Commands,
		cascade_chunk: &CascadeChunk,
		transform: Transform,
	) -> Vec<Entity> {
		for ball in self.ballstick.nodes() {
			let _entities = self.spawn_ball(commands, transform, cascade_chunk, ball);
		}
		for segment in self.ballstick.segments() {
			let _entities = self.spawn_stick(commands, transform, cascade_chunk, &segment);
		}
		vec![]
	}
}
