#[derive(Debug)]
#[allow(dead_code)]
pub struct BTree<T, const D: usize = 2>
where
    T: Ord + std::fmt::Debug,
{
    root: Node<T, D>,
}

impl<T, const D: usize> BTree<T, D>
where
    T: Ord + std::fmt::Debug,
{
    pub fn new() -> Self {
        Self { root: Node::new() }
    }

    pub fn insert(&mut self, value: T) {
        if self.root.is_full() {
            let mut new_root = Node::new();
            new_root
                .children
                .push(std::mem::replace(&mut self.root, Node::new()));
            new_root.split(0);
            self.root = new_root;
        }

        self.root.insert(value);
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct Node<T, const D: usize>
where
    T: Ord + std::fmt::Debug,
{
    keys: Vec<T>,
    children: Vec<Node<T, D>>,
}

#[allow(dead_code)]
impl<T, const D: usize> Node<T, D>
where
    T: Ord + std::fmt::Debug,
{
    fn new() -> Self {
        Self {
            keys: Vec::with_capacity(D * 2 - 1),
            children: Vec::with_capacity(D * 2),
        }
    }

    fn insert(&mut self, value: T) {
        if self.is_leaf() {
            if let Err(i) = self.keys.as_slice().binary_search(&value) {
                self.keys.insert(i, value);
            }
        } else {
            if let Err(mut i) = self.keys.as_slice().binary_search(&value) {
                if self.children[i].is_full() {
                    self.split(i);

                    if value > self.keys[i] {
                        i += 1;
                    }
                }

                self.children[i].insert(value);
            }
        }
    }

    fn split(&mut self, index: usize) {
        let original_child = &mut self.children[index];
        let mut left: Node<T, D> = Node::new();
        let mut right: Node<T, D> = Node::new();
        let mid = D - 1;
        let right_keys = original_child.keys.split_off(mid + 1);
        let mid_key = original_child.keys.pop().unwrap();

        left.keys = std::mem::take(&mut original_child.keys);
        right.keys = right_keys;

        self.keys.insert(index, mid_key);

        if !original_child.is_leaf() {
            right.children = original_child.children.split_off(D);
            left.children = std::mem::take(&mut original_child.children);
        }

        self.children[index] = left;
        self.children.insert(index + 1, right);
    }

    fn is_full(&self) -> bool {
        self.keys.len() >= D * 2 - 1
    }

    fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_insert() {
        let mut btree = BTree::<_, 2>::new();
        println!("{:#?}", btree);
        for i in 1..=20 {
            btree.insert(i);
        }
        println!("{:#?}", btree);
    }
}
