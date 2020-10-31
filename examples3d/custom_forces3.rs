extern crate nalgebra as na;

use na::{Isometry3, Point3, Unit, Vector3};
use ncollide3d::shape::{Cuboid, ShapeHandle};
use rapier3d::dynamics::{JointSet, RigidBodyBuilder, RigidBodySet};
use rapier3d::geometry::{ColliderBuilder, ColliderSet, ColliderShape};
use rapier_testbed3d::Testbed;
use salva3d::integrations::rapier::{
    ColliderCouplingSet, ColliderSampling, FluidsPipeline, FluidsRenderingMode, FluidsTestbedPlugin,
};
use salva3d::object::{Boundary, Fluid};
use salva3d::solver::NonPressureForce;
use salva3d::solver::{ArtificialViscosity, DFSPHSolver};
use salva3d::LiquidWorld;
use std::f32;

#[path = "./helper.rs"]
mod helper;

const PARTICLE_RADIUS: f32 = 0.025;
const SMOOTHING_FACTOR: f32 = 2.0;

pub fn init_world(testbed: &mut Testbed) {
    /*
     * World
     */
    testbed.physics_state_mut().gravity.fill(0.0);
    let mut plugin = FluidsTestbedPlugin::new();
    let mut bodies = RigidBodySet::new();
    let mut colliders = ColliderSet::new();
    let joints = JointSet::new();
    let mut fluids_pipeline = FluidsPipeline::new(PARTICLE_RADIUS, SMOOTHING_FACTOR);

    // fluids.
    let nparticles = 10;
    let custom_force1 = CustomForceField {
        origin: Point3::new(1.0, 0.0, 0.0),
    };
    let custom_force2 = CustomForceField {
        origin: Point3::new(-1.0, 0.0, 0.0),
    };
    let mut fluid = helper::cube_fluid(nparticles, nparticles, nparticles, PARTICLE_RADIUS, 1000.0);
    fluid.nonpressure_forces.push(Box::new(custom_force1));
    fluid.nonpressure_forces.push(Box::new(custom_force2));
    let fluid_handle = fluids_pipeline.liquid_world.add_fluid(fluid);
    plugin.set_fluid_color(fluid_handle, Point3::new(0.8, 0.7, 1.0));

    /*
     * Set up the testbed.
     */
    plugin.set_pipeline(fluids_pipeline);
    plugin.set_fluid_rendering_mode(FluidsRenderingMode::VelocityColor { min: 0.0, max: 5.0 });
    testbed.add_plugin(plugin);
    testbed.set_world(bodies, colliders, joints);
    testbed.integration_parameters_mut().set_dt(1.0 / 200.0);
    testbed.look_at(Point3::new(3.0, 3.0, 3.0), Point3::origin());
}

fn main() {
    let testbed = Testbed::from_builders(0, vec![("Boxes", init_world)]);
    testbed.run()
}

struct CustomForceField {
    origin: Point3<f32>,
}

impl NonPressureForce for CustomForceField {
    fn solve(
        &mut self,
        _timestep: &salva3d::TimestepManager,
        _kernel_radius: f32,
        _fluid_fluid_contacts: &salva3d::geometry::ParticlesContacts,
        _fluid_boundaries_contacts: &salva3d::geometry::ParticlesContacts,
        fluid: &mut Fluid,
        _boundaries: &[Boundary],
        _densities: &[f32],
    ) {
        for (pos, acc) in fluid.positions.iter().zip(fluid.accelerations.iter_mut()) {
            if let Some((dir, dist)) = Unit::try_new_and_get(self.origin - pos, 0.1) {
                *acc += *dir / dist;
            }
        }
    }

    fn apply_permutation(&mut self, _permutation: &[usize]) {}
}
