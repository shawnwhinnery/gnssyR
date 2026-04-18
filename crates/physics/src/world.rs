use crate::{
    body::{Body, BodyHandle},
    contact::Contact,
    narrow,
};

/// Baumgarte correction: fraction of penetration corrected per step.
const CORRECTION_FACTOR: f32 = 0.2;
/// Penetration slop — depths below this are not corrected (prevents jitter).
const SLOP: f32 = 0.01;

/// Manages a set of rigid bodies and advances their simulation.
pub struct PhysicsWorld {
    bodies: Vec<Option<Body>>,
    last_contacts: Vec<(BodyHandle, BodyHandle, Contact)>,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        PhysicsWorld {
            bodies: Vec::new(),
            last_contacts: Vec::new(),
        }
    }

    /// Add a body and return its stable handle.
    pub fn add_body(&mut self, body: Body) -> BodyHandle {
        let idx = self.bodies.len();
        self.bodies.push(Some(body));
        BodyHandle(idx)
    }

    /// Remove a body. Its handle becomes invalid.
    pub fn remove_body(&mut self, handle: BodyHandle) {
        self.bodies[handle.0] = None;
    }

    /// Immutable access to a body.
    ///
    /// # Panics
    /// Panics if `handle` was removed.
    pub fn body(&self, handle: BodyHandle) -> &Body {
        self.bodies[handle.0]
            .as_ref()
            .expect("BodyHandle is invalid (body was removed)")
    }

    /// Mutable access to a body.
    ///
    /// # Panics
    /// Panics if `handle` was removed.
    pub fn body_mut(&mut self, handle: BodyHandle) -> &mut Body {
        self.bodies[handle.0]
            .as_mut()
            .expect("BodyHandle is invalid (body was removed)")
    }

    /// Contacts produced by the most recent [`step`](Self::step) call.
    pub fn contacts(&self) -> &[(BodyHandle, BodyHandle, Contact)] {
        &self.last_contacts
    }

    /// Advance the simulation by `dt` seconds.
    ///
    /// Steps:
    /// 1. Integrate positions for dynamic bodies.
    /// 2. Broadphase AABB scan.
    /// 3. Narrowphase detection.
    /// 4. Impulse resolution + Baumgarte positional correction.
    pub fn step(&mut self, dt: f32) {
        // 1. Integrate
        for body in self.bodies.iter_mut().flatten() {
            if !body.is_static() {
                body.position += body.velocity * dt;
            }
        }

        // 2 + 3. Broadphase → Narrowphase
        let n = self.bodies.len();
        let mut raw_contacts: Vec<(usize, usize, Contact)> = Vec::new();

        for i in 0..n {
            for j in (i + 1)..n {
                let (bi, bj) = match (&self.bodies[i], &self.bodies[j]) {
                    (Some(a), Some(b)) => (a, b),
                    _ => continue,
                };

                // Broad phase: world-space AABB overlap
                let aabb_i = bi.collider.local_aabb().translate(bi.position);
                let aabb_j = bj.collider.local_aabb().translate(bj.position);
                if !aabb_i.overlaps(aabb_j) {
                    continue;
                }

                // Narrow phase
                if let Some(contact) =
                    narrow::detect(bi.position, &bi.collider, bj.position, &bj.collider)
                {
                    raw_contacts.push((i, j, contact));
                }
            }
        }

        // 4. Resolution
        for &(i, j, ref contact) in &raw_contacts {
            let inv_i = {
                let b = self.bodies[i].as_ref().unwrap();
                if b.is_static() {
                    0.0
                } else {
                    1.0 / b.mass
                }
            };
            let inv_j = {
                let b = self.bodies[j].as_ref().unwrap();
                if b.is_static() {
                    0.0
                } else {
                    1.0 / b.mass
                }
            };
            let inv_sum = inv_i + inv_j;
            if inv_sum < 1e-10 {
                continue; // both static
            }

            let (bi, bj) = get_pair_mut(&mut self.bodies, i, j);

            let rel_vel = bj.velocity - bi.velocity;
            let vel_along_normal = rel_vel.dot(contact.normal);

            // Skip if already separating.
            if vel_along_normal >= 0.0 {
                continue;
            }

            let e = bi.restitution.min(bj.restitution);
            let j_mag = -(1.0 + e) * vel_along_normal / inv_sum;
            let impulse = contact.normal * j_mag;

            bi.velocity -= impulse * inv_i;
            bj.velocity += impulse * inv_j;

            // Baumgarte positional correction.
            let correction_mag = (contact.depth - SLOP).max(0.0) * CORRECTION_FACTOR / inv_sum;
            let correction = contact.normal * correction_mag;
            bi.position -= correction * inv_i;
            bj.position += correction * inv_j;
        }

        // Convert raw indices to handles for the public API.
        self.last_contacts = raw_contacts
            .into_iter()
            .map(|(i, j, c)| (BodyHandle(i), BodyHandle(j), c))
            .collect();
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns mutable references to two distinct elements of a `Vec<Option<Body>>`.
///
/// # Panics
/// Panics if `i >= j` or either slot is `None`.
fn get_pair_mut(bodies: &mut Vec<Option<Body>>, i: usize, j: usize) -> (&mut Body, &mut Body) {
    assert!(i < j, "get_pair_mut: i must be strictly less than j");
    let (left, right) = bodies.split_at_mut(j);
    (
        left[i].as_mut().expect("body i was removed during step"),
        right[0].as_mut().expect("body j was removed during step"),
    )
}
