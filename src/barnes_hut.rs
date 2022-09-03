use bevy::{math::vec3, prelude::Vec3};

pub trait Id: Copy + Eq {}
impl<T> Id for T where T: Copy + Eq {}

#[derive(Debug)]
pub struct BarnesHutTree<TId: Id> {
    root: Node<TId>,
    count: usize,
}

impl<TId: Id> BarnesHutTree<TId> {
    pub fn new(min_bound: Vec3, max_bound: Vec3) -> Self {
        Self {
            root: Node::new(
                (max_bound + min_bound) / 2.0,
                (max_bound - min_bound).max_element(),
            ),
            count: 0,
        }
    }

    pub fn insert(&mut self, id: TId, position: Vec3, mass: f32) {
        self.root.insert(id, position, mass);
        self.count += 1;
    }

    pub fn force(
        &self,
        id: TId,
        position: Vec3,
        force_constant: f32,
        distance_exp: i32,
        theta: f32,
    ) -> Vec3 {
        self.root
            .force(id, position, force_constant, distance_exp, theta)
    }
}

#[derive(Debug)]
struct Node<TId: Id> {
    mass: f32,
    center_of_mass: Vec3,
    midpoint: Vec3,
    size: f32,
    kind: NodeKind<TId>,
}

impl<TId: Id> Node<TId> {
    fn new(midpoint: Vec3, size: f32) -> Self {
        Self {
            mass: 0.0,
            center_of_mass: Vec3::ZERO,
            midpoint,
            size,
            kind: NodeKind::Empty,
        }
    }

    fn insert(&mut self, id: TId, position: Vec3, mass: f32) {
        match &mut self.kind {
            NodeKind::Empty => {
                self.kind = NodeKind::Leaf(id);
            }
            NodeKind::Leaf(prev_id) => {
                let sub_size = self.size / 2.0;
                let min = self.midpoint - sub_size;
                let min_midpoint = (min + self.midpoint) / 2.0;

                let mut nodes = Box::new([
                    // X- Y- Z-
                    Node::new(min_midpoint, sub_size),
                    // X+ Y- Z-
                    Node::new(min_midpoint + sub_size * vec3(1.0, 0.0, 0.0), sub_size),
                    // X- Y+ Z-
                    Node::new(min_midpoint + sub_size * vec3(0.0, 1.0, 0.0), sub_size),
                    // X+ Y+ Z-
                    Node::new(min_midpoint + sub_size * vec3(1.0, 1.0, 0.0), sub_size),
                    // X- Y- Z+
                    Node::new(min_midpoint + sub_size * vec3(0.0, 0.0, 1.0), sub_size),
                    // X+ Y- Z+
                    Node::new(min_midpoint + sub_size * vec3(1.0, 0.0, 1.0), sub_size),
                    // X- Y+ Z+
                    Node::new(min_midpoint + sub_size * vec3(0.0, 1.0, 1.0), sub_size),
                    // X+ Y+ Z+
                    Node::new(min_midpoint + sub_size * vec3(1.0, 1.0, 1.0), sub_size),
                ]);

                nodes[branch_index(self.center_of_mass, self.midpoint)].insert(
                    *prev_id,
                    self.center_of_mass,
                    self.mass,
                );

                nodes[branch_index(position, self.midpoint)].insert(id, position, mass);

                self.kind = NodeKind::Node(nodes);
            }
            NodeKind::Node(node) => {
                node[branch_index(position, self.midpoint)].insert(id, position, mass);
            }
        }
        self.center_of_mass =
            (self.center_of_mass * self.mass + position * mass) / (self.mass + mass);
        self.mass += mass;
    }

    fn force(
        &self,
        id: TId,
        position: Vec3,
        force_constant: f32,
        distance_exp: i32,
        theta: f32,
    ) -> Vec3 {
        match &self.kind {
            NodeKind::Empty => Vec3::ZERO,
            NodeKind::Leaf(node_id) => {
                if id != *node_id {
                    force(
                        position,
                        self.center_of_mass,
                        self.mass,
                        force_constant,
                        distance_exp,
                    )
                } else {
                    Vec3::ZERO
                }
            }
            NodeKind::Node(nodes) => {
                let d = (self.center_of_mass - position).length();
                if self.size / d < theta {
                    force(
                        position,
                        self.center_of_mass,
                        self.mass,
                        force_constant,
                        distance_exp,
                    )
                } else {
                    let mut force = Vec3::ZERO;
                    for node in nodes.iter() {
                        force += node.force(id, position, force_constant, distance_exp, theta);
                    }
                    force
                }
            }
        }
    }
}

#[derive(Debug)]
enum NodeKind<TId: Id> {
    Empty,
    Leaf(TId),
    Node(Box<[Node<TId>; 8]>),
}

fn branch_index(position: Vec3, midpoint: Vec3) -> usize {
    let offset = position - midpoint;
    let onoff = (offset.signum() + 1.0) / 2.0;
    onoff.x as usize + onoff.y as usize * 2 + onoff.z as usize * 4
}

fn force(
    position: Vec3,
    body_position: Vec3,
    body_mass: f32,
    force_constant: f32,
    distance_exp: i32,
) -> Vec3 {
    if body_mass <= 0.0 {
        return Vec3::ZERO;
    }
    let d = body_position - position;
    force_constant
        * body_mass
        * d.normalize_or_zero()
        * ((d.length() + f32::EPSILON).powi(distance_exp))
}
