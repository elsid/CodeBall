use crate::model::Arena;
use crate::my_strategy::common::{Clamp, Square};
use crate::my_strategy::plane::Plane;
use crate::my_strategy::simulator::Solid;
use crate::my_strategy::sphere::Sphere;
use crate::my_strategy::vec2::Vec2;
use crate::my_strategy::vec3::Vec3;

impl Arena {
    pub fn get_goal_target(&self) -> Vec3 {
        Vec3::new(0.0, self.goal_height / 2.0, self.depth / 2.0 + self.goal_depth / 2.0)
    }

    pub fn get_defend_target(&self) -> Vec3 {
        Vec3::new(0.0, self.goal_height / 2.0, -self.depth / 2.0 + self.goal_depth / 2.0)
    }

    pub fn collide(&self, e: &mut Solid) -> Option<Vec3> {
        let (distance, normal) = self.distance_and_normal_to_arena(e.position());
        let penetration = e.radius() - distance;
        if penetration > 0.0 {
            let e_position = e.position() + normal * penetration;
            e.set_position(e_position);
            let velocity = normal.dot(e.velocity()) - e.radius_change_speed();
            if velocity < 0.0 {
                let e_velocity = e.velocity() - normal * (1.0 + e.arena_e()) * velocity;
                e.set_velocity(e_velocity);
                Some(normal)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_penetration(&self, e: &Solid) -> f64 {
        let (distance, normal) = self.distance_and_normal_to_arena(e.position());
        e.radius() - distance
    }

    pub fn contains(&self, position: Vec3) -> bool {
        let (distance, _) = self.distance_and_normal_to_arena(position);
        distance > 0.0
    }

    pub fn distance_and_normal_to_arena(&self, mut position: Vec3) -> (f64, Vec3) {
        let negate_x = position.x() < 0.0;
        let negate_z = position.z() < 0.0;
        if negate_x {
            position = position.with_neg_x();
        }
        if negate_z {
            position = position.with_neg_z();
        }
        let (distance, mut normal) = self.distance_and_normal_to_arena_quarter(position);
        if negate_x {
            normal = normal.with_neg_x();
        }
        if negate_z {
            normal = normal.with_neg_z();
        }
        (distance, normal)
    }

    pub fn distance_and_normal_to_arena_quarter(&self, position: Vec3) -> (f64, Vec3) {
        let (mut distance, mut normal) = Arena::distance_and_normal_to_ground(position);
        self.collide_ceiling(position, &mut distance, &mut normal);
        self.collide_side_x(position, &mut distance, &mut normal);
        self.collide_goal_side_z(position, &mut distance, &mut normal);
        self.collide_side_z(position, &mut distance, &mut normal);
        self.collide_goal_side_x_and_ceiling(position, &mut distance, &mut normal);
        self.collide_goal_back_corners(position, &mut distance, &mut normal);
        self.collide_corner(position, &mut distance, &mut normal);
        self.collide_goal_outer_corner(position, &mut distance, &mut normal);
        self.collide_goal_inside_top_corners(position, &mut distance, &mut normal);
        self.collide_bottom_corners(position, &mut distance, &mut normal);
        self.collide_ceiling_corners(position, &mut distance, &mut normal);
        (distance, normal)
    }

    pub fn distance_and_normal_to_ground(position: Vec3) -> (f64, Vec3) {
        let ground = Plane::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0)
        );
        (ground.distance(position), ground.normal())
    }

    pub fn collide_ceiling(&self, position: Vec3, distance: &mut f64, normal: &mut Vec3) {
        Plane::new(
            Vec3::new(0.0, self.height, 0.0),
            Vec3::new(0.0, -1.0, 0.0)
        ).collide(position, distance, normal);
    }

    pub fn collide_side_x(&self, position: Vec3, distance: &mut f64, normal: &mut Vec3) {
        Plane::new(
            Vec3::new(self.width / 2.0, 0.0, 0.0),
            Vec3::new(-1.0, 0.0, 0.0)
        ).collide(position, distance, normal);
    }

    pub fn collide_goal_side_z(&self, position: Vec3, distance: &mut f64, normal: &mut Vec3) {
        Plane::new(
            Vec3::new(0.0, 0.0, (self.depth / 2.0) + self.goal_depth),
            Vec3::new(0.0, 0.0, -1.0)
        ).collide(position, distance, normal);
    }

    pub fn collide_side_z(&self, position: Vec3, distance: &mut f64, normal: &mut Vec3) {
        let v = position.xy() - Vec2::new(
            (self.goal_width / 2.0) - self.goal_top_radius,
            self.goal_height - self.goal_top_radius
        );
        if position.x() >= (self.goal_width / 2.0) + self.goal_side_radius
            || position.y() >= self.goal_height + self.goal_side_radius
            || (v.x() > 0.0 && v.y() > 0.0 && v.norm() >= self.goal_top_radius + self.goal_side_radius) {
            Plane::new(
                Vec3::new(0.0, 0.0, self.depth / 2.0),
                Vec3::new(0.0, 0.0, -1.0)
            ).collide(position, distance, normal);
        }
    }

    pub fn collide_goal_side_x_and_ceiling(&self, position: Vec3, distance: &mut f64, normal: &mut Vec3) {
        if position.z() >= (self.depth / 2.0) + self.goal_side_radius {
            Plane::new(
                Vec3::new(self.goal_width / 2.0, 0.0, 0.0),
                Vec3::new(-1.0, 0.0, 0.0)
            ).collide(position, distance, normal);
            Plane::new(
                Vec3::new(0.0, self.goal_height, 0.0),
                Vec3::new(0.0, -1.0, 0.0)
            ).collide(position, distance, normal);
        }
    }

    pub fn collide_goal_back_corners(&self, position: Vec3, distance: &mut f64, normal: &mut Vec3) {
        assert_eq!(self.bottom_radius, self.goal_top_radius);
        if position.z() >= (self.depth / 2.0) + self.goal_side_radius {
            Sphere::new(
                Vec3::new(
                    position.x().clamp(
                        self.bottom_radius - (self.goal_width / 2.0),
                        (self.goal_width / 2.0) - self.bottom_radius
                    ),
                    position.y().clamp(
                        self.bottom_radius,
                        self.goal_height - self.goal_top_radius
                    ),
                    (self.depth / 2.0) + self.goal_depth - self.bottom_radius
                ),
                self.bottom_radius
            ).inner_collide(position, distance, normal);
        }
    }

    pub fn collide_corner(&self, position: Vec3, distance: &mut f64, normal: &mut Vec3) {
        if position.x() > (self.width / 2.0) - self.corner_radius
            && position.z() >= (self.depth / 2.0) - self.corner_radius {
            Sphere::new(
                Vec3::new(
                    (self.width / 2.0) - self.corner_radius,
                    position.y(),
                    (self.depth / 2.0) - self.corner_radius
                ),
                self.corner_radius
            ).inner_collide(position, distance, normal);
        }
    }

    pub fn collide_goal_outer_corner(&self, position: Vec3, distance: &mut f64, normal: &mut Vec3) {
        if position.z() < (self.depth / 2.0) + self.goal_side_radius {
            if position.x() < (self.goal_width / 2.0) + self.goal_side_radius {
                Sphere::new(
                    Vec3::new(
                        (self.goal_width / 2.0) + self.goal_side_radius,
                        position.y(),
                        (self.depth / 2.0) + self.goal_side_radius
                    ),
                    self.goal_side_radius
                ).outer_collide(position, distance, normal);
            }
            // Ceiling
            if position.y() < self.goal_height + self.goal_side_radius {
                Sphere::new(
                    Vec3::new(
                        position.x(),
                        self.goal_height + self.goal_side_radius,
                        (self.depth / 2.0) + self.goal_side_radius
                    ),
                    self.goal_side_radius
                ).outer_collide(position, distance, normal);
            }
            // Top corner
            let o = Vec2::new(
                (self.goal_width / 2.0) - self.goal_top_radius,
                self.goal_height - self.goal_top_radius
            );
            let v = Vec2::new(position.x(), position.y()) - o;
            if v.x() > 0.0 && v.y() > 0.0 {
                let o = o + v.normalized() * (self.goal_top_radius + self.goal_side_radius);
                Sphere::new(
                    Vec3::new(o.x(), o.y(), (self.depth / 2.0) + self.goal_side_radius),
                    self.goal_side_radius
                ).outer_collide(position, distance, normal);
            }
        }
    }

    pub fn collide_goal_inside_top_corners(&self, position: Vec3, distance: &mut f64, normal: &mut Vec3) {
        if position.z() > (self.depth / 2.0) + self.goal_side_radius
            && position.y() > self.goal_height - self.goal_top_radius {
            if position.x() > (self.goal_width / 2.0) - self.goal_top_radius {
                Sphere::new(
                    Vec3::new(
                        (self.goal_width / 2.0) - self.goal_top_radius,
                        self.goal_height - self.goal_top_radius,
                        position.z()
                    ),
                    self.goal_top_radius
                ).inner_collide(position, distance, normal);
            }
            if position.z() > (self.depth / 2.0) + self.goal_depth - self.goal_top_radius {
                Sphere::new(
                    Vec3::new(
                        position.x(),
                        self.goal_height - self.goal_top_radius,
                        (self.depth / 2.0) + self.goal_depth - self.goal_top_radius
                    ),
                    self.goal_top_radius
                ).inner_collide(position, distance, normal);
            }
        }
    }

    pub fn collide_bottom_corners(&self, position: Vec3, distance: &mut f64, normal: &mut Vec3) {
        if position.y() < self.bottom_radius {
            if position.x() > (self.width / 2.0) - self.bottom_radius {
                Sphere::new(
                    Vec3::new(
                        (self.width / 2.0) - self.bottom_radius,
                        self.bottom_radius,
                        position.z()
                    ),
                    self.bottom_radius
                ).inner_collide(position, distance, normal);
            }
            if position.z() > (self.depth / 2.0) - self.bottom_radius
                && position.x() >= (self.goal_width / 2.0) + self.goal_side_radius {
                Sphere::new(
                    Vec3::new(
                        position.x(),
                        self.bottom_radius,
                        (self.depth / 2.0) - self.bottom_radius
                    ),
                    self.bottom_radius
                ).inner_collide(position, distance, normal);
            }
            if position.z() > (self.depth / 2.0) + self.goal_depth - self.bottom_radius {
                Sphere::new(
                    Vec3::new(
                        position.x(),
                        self.bottom_radius,
                        (self.depth / 2.0) + self.goal_depth - self.bottom_radius
                    ),
                    self.bottom_radius
                ).inner_collide(position, distance, normal);
                let o = Vec2::new(
                    (self.goal_width / 2.0) + self.goal_side_radius,
                    (self.depth / 2.0) + self.goal_side_radius
                );
                let v = Vec2::new(position.x(), position.z()) - o;
                if v.x() < 0.0 && v.y() < 0.0 && v.norm() < self.goal_side_radius + self.bottom_radius {
                    let o = o + v.normalized() * (self.goal_side_radius + self.bottom_radius);
                    Sphere::new(
                        Vec3::new(o.x(), self.bottom_radius, o.y()),
                        self.bottom_radius
                    ).inner_collide(position, distance, normal);
                }
                if position.z() >= (self.depth / 2.0) + self.goal_side_radius
                    && position.x() > (self.goal_width / 2.0) - self.bottom_radius {
                    Sphere::new(
                        Vec3::new(
                            (self.goal_width / 2.0) - self.bottom_radius,
                            self.bottom_radius,
                            position.z()
                        ),
                        self.bottom_radius
                    ).inner_collide(position, distance, normal);
                }
                if position.x() > (self.width / 2.0) - self.corner_radius
                    && position.z() > (self.depth / 2.0) - self.corner_radius {
                    let corner_o = Vec2::new(
                        (self.width / 2.0) - self.corner_radius,
                        (self.depth / 2.0) - self.corner_radius
                    );
                    let n = Vec2::new(position.x(), position.z()) - corner_o;
                    let dist = n.norm();
                    if dist > self.corner_radius - self.bottom_radius {
                        let n = n / dist;
                        let o2 = corner_o + n * (self.corner_radius - self.bottom_radius);
                        Sphere::new(
                            Vec3::new(o2.x(), self.bottom_radius, o2.y()),
                            self.bottom_radius
                        ).inner_collide(position, distance, normal);
                    }
                }
            }
        }
    }

    pub fn collide_ceiling_corners(&self, position: Vec3, distance: &mut f64, normal: &mut Vec3) {
        if position.y() > self.height - self.top_radius {
            if position.x() > (self.width / 2.0) - self.top_radius {
                Sphere::new(
                    Vec3::new(
                        (self.width / 2.0) - self.top_radius,
                        self.height - self.top_radius,
                        position.z(),
                    ),
                    self.top_radius
                ).inner_collide(position, distance, normal);
            }
            if position.z() > (self.depth / 2.0) - self.top_radius {
                Sphere::new(
                    Vec3::new(
                        position.x(),
                        self.height - self.top_radius,
                        (self.depth / 2.0) - self.top_radius,
                    ),
                    self.top_radius
                ).inner_collide(position, distance, normal);
            }
            if position.x() > (self.width / 2.0) - self.corner_radius
                && position.z() > (self.depth / 2.0) - self.corner_radius {
                let corner_o = Vec2::new(
                    (self.width / 2.0) - self.corner_radius,
                    (self.depth / 2.0) - self.corner_radius
                );
                let dv = Vec2::new(position.x(), position.z()) - corner_o;
                if dv.norm() > self.corner_radius - self.top_radius {
                    let n = dv.normalized();
                    let o2 = corner_o + n * (self.corner_radius - self.top_radius);
                    Sphere::new(
                        Vec3::new(
                            o2.x(),
                            self.height - self.top_radius,
                            o2.y()
                        ),
                        self.top_radius
                    ).inner_collide(position, distance, normal);
                }
            }
        }
    }
}