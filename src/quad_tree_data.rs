#[derive(Clone)]
pub enum QuadTreeNode<T> {
    Data(T),
    Node(Box<QuadTreeNode<T>>, Box<QuadTreeNode<T>>, Box<QuadTreeNode<T>>, Box<QuadTreeNode<T>>)
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
    pub fn run_on_data<F>(&self, closure: F) where F: Fn(&T) {
        match self {
            QuadTreeNode::Data(data) => {closure(data)}
            QuadTreeNode::Node(a, b, c, d) => {
                a.run_on_data(&closure);
                b.run_on_data(&closure);
                c.run_on_data(&closure);
                d.run_on_data(&closure);
            }
        }
    }

    pub fn get_data(&mut self, depth: i32, position: [i32; 2]) -> Option<&mut T> {
        if depth == 0 {
            return match self {
                QuadTreeNode::Data(data) => {Some(data)}
                QuadTreeNode::Node(_, _, _, _) => {None}
            };
        }

        return match self {
            QuadTreeNode::Data(_) => {None}
            QuadTreeNode::Node(a, b, c, d) => {
                let divider = 2_i32.pow(depth as u32 - 1);

                if position[0] / divider == 0 {
                    if position[1] / divider == 0 {
                        return a.get_data(depth - 1, position);
                    } else {
                        return c.get_data(depth - 1, [position[0], position[1] - divider]);
                    }
                } else {
                    if position[1] / divider == 0 {
                        return b.get_data(depth - 1, [position[0] - divider, position[1]]);
                    } else {
                        return d.get_data(depth - 1, [position[0] - divider, position[1] - divider]);
                    }
                }
            }
        }
    }
}