//! [`FixedJoint2`] component.
//! Copied from bevy_xpbd FixedJoint to add rotation offset support.

use bevy::{
    ecs::entity::{EntityMapper, MapEntities},
    prelude::*,
};
use bevy_xpbd_3d::{
    math::{Scalar, Vector},
    plugins::solver::joint_damping,
    prelude::*,
    SubstepSchedule, SubstepSet,
};

pub struct FixedJoint2Plugin;

impl Plugin for FixedJoint2Plugin {
    fn build(&self, app: &mut App) {
        let substeps = app
            .get_schedule_mut(SubstepSchedule)
            .expect("add SubstepSchedule first");

        substeps.add_systems(
            solve_constraint::<FixedJoint2, 2>
                .after(solve_constraint::<FixedJoint, 2>)
                .before(solve_constraint::<RevoluteJoint, 2>)
                .in_set(SubstepSet::SolveConstraints),
        );

        substeps.add_systems(
            joint_damping::<FixedJoint2>
                .after(joint_damping::<FixedJoint>)
                .before(joint_damping::<RevoluteJoint>)
                .in_set(SubstepSet::SolveVelocities),
        );
    }
}

/// A fixed joint prevents any relative movement of the attached bodies.
///
/// You should generally prefer using a single body instead of multiple bodies fixed together,
/// but fixed joints can be useful for things like rigid structures where a force can dynamically break the joints connecting individual bodies.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct FixedJoint2 {
    /// First entity constrained by the joint.
    pub entity1: Entity,
    /// Second entity constrained by the joint.
    pub entity2: Entity,
    /// Attachment point on the first body.
    pub local_anchor1: Vector,
    /// Attachment point on the second body.
    pub local_anchor2: Vector,
    /// Rotation offset from body 1 to body 2.
    pub rotation_offset: Rotation,
    /// Linear damping applied by the joint.
    pub damping_linear: Scalar,
    /// Angular damping applied by the joint.
    pub damping_angular: Scalar,
    /// Lagrange multiplier for the positional correction.
    pub position_lagrange: Scalar,
    /// Lagrange multiplier for the angular correction caused by the alignment of the bodies.
    pub align_lagrange: Scalar,
    /// The joint's compliance, the inverse of stiffness, has the unit meters / Newton.
    pub compliance: Scalar,
    /// The force exerted by the joint.
    pub force: Vector,
    /// The torque exerted by the joint when aligning the bodies.
    pub align_torque: Vector,
}

impl XpbdConstraint<2> for FixedJoint2 {
    fn entities(&self) -> [Entity; 2] {
        [self.entity1, self.entity2]
    }

    fn clear_lagrange_multipliers(&mut self) {
        self.position_lagrange = 0.0;
        self.align_lagrange = 0.0;
    }

    fn solve(&mut self, bodies: [&mut RigidBodyQueryItem; 2], dt: Scalar) {
        let [body1, body2] = bodies;
        let compliance = self.compliance;

        // Align orientation
        let dq = self.get_delta_q(&body1.rotation, &body2.rotation);
        let mut lagrange = self.align_lagrange;
        self.align_torque = self.align_orientation(body1, body2, dq, &mut lagrange, compliance, dt);
        self.align_lagrange = lagrange;

        // Align position of local attachment points
        let mut lagrange = self.position_lagrange;
        self.force = self.align_position(
            body1,
            body2,
            self.local_anchor1,
            self.local_anchor2,
            &mut lagrange,
            compliance,
            dt,
        );
        self.position_lagrange = lagrange;
    }
}

impl Joint for FixedJoint2 {
    fn new(entity1: Entity, entity2: Entity) -> Self {
        Self {
            entity1,
            entity2,
            local_anchor1: Vector::ZERO,
            local_anchor2: Vector::ZERO,
            rotation_offset: Rotation(Quat::IDENTITY),
            damping_linear: 1.0,
            damping_angular: 1.0,
            position_lagrange: 0.0,
            align_lagrange: 0.0,
            compliance: 0.0,
            force: Vector::ZERO,
            align_torque: Vector::ZERO,
        }
    }

    fn with_compliance(self, compliance: Scalar) -> Self {
        Self { compliance, ..self }
    }

    fn with_local_anchor_1(self, anchor: Vector) -> Self {
        Self {
            local_anchor1: anchor,
            ..self
        }
    }

    fn with_local_anchor_2(self, anchor: Vector) -> Self {
        Self {
            local_anchor2: anchor,
            ..self
        }
    }

    fn with_linear_velocity_damping(self, damping: Scalar) -> Self {
        Self {
            damping_linear: damping,
            ..self
        }
    }

    fn with_angular_velocity_damping(self, damping: Scalar) -> Self {
        Self {
            damping_angular: damping,
            ..self
        }
    }

    fn local_anchor_1(&self) -> Vector {
        self.local_anchor1
    }

    fn local_anchor_2(&self) -> Vector {
        self.local_anchor2
    }

    fn damping_linear(&self) -> Scalar {
        self.damping_linear
    }

    fn damping_angular(&self) -> Scalar {
        self.damping_angular
    }
}

impl FixedJoint2 {
    pub fn with_rotation_offset(self, offset: Rotation) -> Self {
        let offset = offset.normalize();
        // let offset = if offset.w < 0.0 { -offset } else { offset };
        Self {
            rotation_offset: offset.into(),
            ..self
        }
    }

    fn get_delta_q(&self, rot1: &Rotation, rot2: &Rotation) -> Vector {
        let delta_q = rot1.0 * self.rotation_offset.0 * rot2.inverse().0;
        let delta_q = if delta_q.w < 0.0 { delta_q } else { -delta_q };
        2.0 * delta_q.xyz()
    }
}

impl PositionConstraint for FixedJoint2 {}

impl AngularConstraint for FixedJoint2 {}

impl MapEntities for FixedJoint2 {
    fn map_entities(&mut self, entity_mapper: &mut EntityMapper) {
        self.entity1 = entity_mapper.get_or_reserve(self.entity1);
        self.entity2 = entity_mapper.get_or_reserve(self.entity2);
    }
}
