use crate::world_generation::chunk_loading::quad_tree_data::QuadTreeNode::Node;
use bevy::prelude::{Commands, Entity};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub enum QuadTreeNode<T> {
    Data(T, Vec<Entity>),
    Node(
        Box<QuadTreeNode<T>>,
        Box<QuadTreeNode<T>>,
        Box<QuadTreeNode<T>>,
        Box<QuadTreeNode<T>>,
        Arc<Mutex<i32>>,
        Vec<Entity>,
    ),
}

pub enum QuadTreeDistinction {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Into<i32> for QuadTreeDistinction {
    fn into(self) -> i32 {
        match self {
            QuadTreeDistinction::TopLeft => 0,
            QuadTreeDistinction::TopRight => 1,
            QuadTreeDistinction::BottomLeft => 2,
            QuadTreeDistinction::BottomRight => 3,
        }
    }
}

impl<T> QuadTreeNode<T> {
    pub fn run_on_data<F>(&self, closure: F)
    where
        F: Fn(&T),
    {
        match self {
            QuadTreeNode::Data(data, _) => closure(data),
            QuadTreeNode::Node(a, b, c, d, _, _) => {
                a.run_on_data(&closure);
                b.run_on_data(&closure);
                c.run_on_data(&closure);
                d.run_on_data(&closure);
            }
        }
    }

    pub fn add_to_parent(&mut self, depth: i32, position: [i32; 2], commands: &mut Commands) {
        let mut further = false;
        if let Some(Node(_, _, _, _, child_progress, entities)) =
            self.get_parent_node(depth, position)
        {
            let mut child_progress_lock = child_progress.lock().unwrap();
            *child_progress_lock += 1;

            if *child_progress_lock == 4 {
                for entity in entities {
                    if let Ok(mut entity) = commands.get_entity(entity.clone()) {
                        entity.despawn();
                    }
                }

                if depth != 1 {
                    further = true;
                }
            }
        }

        if further {
            self.add_to_parent(depth - 1, [position[0] / 2, position[1] / 2], commands);
        }
    }

    pub fn get_parent_node(&mut self, depth: i32, position: [i32; 2]) -> Option<&QuadTreeNode<T>> {
        if depth <= 1 {
            return Some(self);
        }

        return match self {
            QuadTreeNode::Data(_, _) => None,
            QuadTreeNode::Node(a, b, c, d, _, _) => {
                let divider = 2_i32.pow(depth as u32 - 1);

                return if position[0] / divider == 0 {
                    if position[1] / divider == 0 {
                        a.get_parent_node(depth - 1, position)
                    } else {
                        c.get_parent_node(depth - 1, [position[0], position[1] - divider])
                    }
                } else {
                    if position[1] / divider == 0 {
                        b.get_parent_node(depth - 1, [position[0] - divider, position[1]])
                    } else {
                        d.get_parent_node(depth - 1, [position[0] - divider, position[1] - divider])
                    }
                };
            }
        };
    }

    pub fn get_node(&mut self, depth: i32, position: [i32; 2]) -> Option<&mut Self> {
        if depth == 0 {
            return Some(self);
        }

        return match self {
            QuadTreeNode::Data(_, _) => None,
            QuadTreeNode::Node(a, b, c, d, _, _) => {
                let divider = 2_i32.pow(depth as u32 - 1);

                return if position[0] / divider == 0 {
                    if position[1] / divider == 0 {
                        a.get_node(depth - 1, position)
                    } else {
                        c.get_node(depth - 1, [position[0], position[1] - divider])
                    }
                } else {
                    if position[1] / divider == 0 {
                        b.get_node(depth - 1, [position[0] - divider, position[1]])
                    } else {
                        d.get_node(depth - 1, [position[0] - divider, position[1] - divider])
                    }
                };
            }
        };
    }
}
