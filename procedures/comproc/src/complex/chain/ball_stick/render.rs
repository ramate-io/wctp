use crate::complex::chain::ball_stick::builder::{BallStick, BallStickNode, BallStickSegment};
use bevy::prelude::*;
use chunk::cascade::CascadeChunk;
use render_item::RenderItem;
use std::marker::PhantomData;

pub trait BallStickPrespawn<B: RenderItem, S: RenderItem> {
	/// Computes the appropriate transform for the ball at the given node.
	fn prespawn_ball(
		&self,
		cascade_chunk: &CascadeChunk,
		node: &BallStickNode,
		transform: Transform,
	) -> Option<(CascadeChunk, B, Transform)>;

	/// Computes the appropriate transform for the stick at the given segment.
	fn prespawn_stick(
		&self,
		cascade_chunk: &CascadeChunk,
		segment: &BallStickSegment,
		transform: Transform,
	) -> Option<(CascadeChunk, S, Transform)>;
}

#[derive(Component, Debug, Clone)]
pub struct BallStickRenderItem<B: RenderItem, S: RenderItem, P: BallStickPrespawn<B, S>> {
	ballstick: BallStick,
	prespawn: P,
	__ball_marker: PhantomData<B>,
	__stick_marker: PhantomData<S>,
}

impl<B: RenderItem, S: RenderItem, P: BallStickPrespawn<B, S>> BallStickRenderItem<B, S, P> {
	pub fn new(ballstick: BallStick, prespawn: P) -> Self {
		Self { ballstick, prespawn, __ball_marker: PhantomData, __stick_marker: PhantomData }
	}

	pub fn with_prespawn(mut self, prespawn: P) -> Self {
		self.prespawn = prespawn;
		self
	}

	pub fn with_ballstick(mut self, ballstick: BallStick) -> Self {
		self.ballstick = ballstick;
		self
	}

	pub fn prespawn_ball(
		&self,
		cascade_chunk: &CascadeChunk,
		node: &BallStickNode,
		transform: Transform,
	) -> Option<(CascadeChunk, B, Transform)> {
		self.prespawn.prespawn_ball(cascade_chunk, node, transform)
	}

	pub fn prespawn_stick(
		&self,
		cascade_chunk: &CascadeChunk,
		segment: &BallStickSegment,
		transform: Transform,
	) -> Option<(CascadeChunk, S, Transform)> {
		self.prespawn.prespawn_stick(cascade_chunk, segment, transform)
	}

	pub fn into_parts(self) -> (BallStick, P) {
		(self.ballstick, self.prespawn)
	}
}

impl<B: RenderItem, S: RenderItem, P: BallStickPrespawn<B, S> + Clone> RenderItem
	for BallStickRenderItem<B, S, P>
{
	fn spawn_render_items(
		&self,
		commands: &mut Commands,
		cascade_chunk: &CascadeChunk,
		transform: Transform,
	) -> Vec<Entity> {
		for ball in self.ballstick.nodes() {
			if let Some((cascade_chunk, ball, transform)) =
				self.prespawn_ball(cascade_chunk, ball, transform)
			{
				ball.spawn_render_items(commands, &cascade_chunk, transform);
			};
		}
		for segment in self.ballstick.segments() {
			if let Some((cascade_chunk, stick, transform)) =
				self.prespawn_stick(cascade_chunk, &segment, transform)
			{
				stick.spawn_render_items(commands, &cascade_chunk, transform);
			};
		}
		vec![]
	}
}
