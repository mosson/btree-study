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

    pub fn iter(&self) -> BTreeIterator<T, D> {
        BTreeIterator::new(self)
    }

    pub fn delete(&mut self, value: &T) -> Result<(), Box<dyn std::error::Error>> {
        match self.find_parent_and_index(value) {
            None => self.root.delete_intermediate(value),
            Some((parent, index)) => {
                let target = parent.children.get(index).unwrap();
                if !target.is_leaf() {
                    return self.root.delete_intermediate(value);
                }

                if !parent.is_delete_child(index) {
                    if index == parent.keys.len() {
                        parent.pivot_right(index - 1);
                    } else {
                        parent.pivot_left(index);
                    }

                    return self.delete(value);
                }

                let target = parent.children.get_mut(index).unwrap();
                target.delete_key(value);

                Ok(())
            }
        }
    }

    fn find_parent_and_index(&mut self, value: &T) -> Option<(&mut Node<T, D>, usize)> {
        match self.root.keys.as_slice().binary_search(value) {
            Ok(_) => None,
            Err(i) => {
                let hit = match self.root.children.get(i) {
                    Some(child) => match child.keys.as_slice().binary_search(value) {
                        Ok(_) => Some(true),
                        Err(_) => Some(false),
                    },
                    None => None,
                };

                match hit {
                    Some(true) => Some((&mut self.root, i)),
                    Some(false) => self
                        .root
                        .children
                        .get_mut(i)
                        .unwrap()
                        .find_parent_and_index(value),
                    None => None,
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct BTreeIterator<'a, T, const D: usize>
where
    T: Ord + std::fmt::Debug,
{
    stack: Vec<(usize, &'a Node<T, D>)>,
}

impl<'a, T, const D: usize> BTreeIterator<'a, T, D>
where
    T: Ord + std::fmt::Debug,
{
    pub fn new(source: &'a BTree<T, D>) -> Self {
        let mut iter = BTreeIterator { stack: vec![] };
        iter.stacking(&source.root);
        iter
    }

    fn stacking(&mut self, node: &'a Node<T, D>) {
        self.stack.push((0, node));

        if let Some(first_child) = node.children.first() {
            self.stacking(first_child);
        }
    }
}

impl<'a, T, const D: usize> Iterator for BTreeIterator<'a, T, D>
where
    T: Ord + std::fmt::Debug,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((i, node)) = self.stack.pop() {
                if i < node.keys.len() {
                    let value = &node.keys[i];

                    self.stack.push((i + 1, node));

                    if !node.is_leaf() {
                        self.stacking(&node.children[i + 1]);
                    }

                    return Some(value);
                }
            } else {
                break None;
            }
        }
    }
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub struct Node<T, const D: usize>
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

    fn find_parent_and_index(&mut self, value: &T) -> Option<(&mut Node<T, D>, usize)> {
        match self.keys.as_slice().binary_search(value) {
            Ok(_) => unreachable!("根の検査でここには到達しない"),
            Err(i) => {
                let hit = match self.children.get(i) {
                    Some(child) => match child.keys.as_slice().binary_search(value) {
                        Ok(_) => Some(true),
                        Err(_) => Some(false),
                    },
                    None => None,
                };

                match hit {
                    Some(true) => Some((self, i)),
                    Some(false) => self
                        .children
                        .get_mut(i)
                        .unwrap()
                        .find_parent_and_index(value),
                    None => None,
                }
            }
        }
    }

    fn pivot_left(&mut self, index: usize) {
        let value = self
            .keys
            .get_mut(index)
            .expect("親ノードの値が取得できません");
        let (left_nodes, right_nodes) = self.children.split_at_mut(index + 1);
        let left_node = left_nodes.last_mut().expect("左ノードが取得できません");
        let right_node = right_nodes.first_mut().expect("右ノードが取得できません");

        if right_node.keys.len() < D {
            return self.merge_right(index);
        }

        let right_value = right_node.keys.remove(0);
        let pivot_key = std::mem::replace(value, right_value);
        left_node.keys.push(pivot_key);
    }

    fn merge_right(&mut self, index: usize) {
        let mut right_node = self.children.remove(index + 1);
        let mut left_node = self.children.remove(index);

        let mut new_right_keys = vec![self.keys.remove(index)];
        let left_keys = std::mem::take(&mut left_node.keys);
        new_right_keys.extend(left_keys);
        let rest = std::mem::replace(&mut right_node.keys, new_right_keys);
        right_node.keys.extend(rest);

        let new_right_children = left_node.children;
        let rest_children = std::mem::replace(&mut right_node.children, new_right_children);
        right_node.children.extend(rest_children);

        if self.keys.is_empty() {
            std::mem::swap(self, &mut right_node);
        } else {
            self.children.insert(index, right_node);
        }
    }

    fn pivot_right(&mut self, index: usize) {
        let (left_nodes, right_nodes) = self.children.split_at_mut(index + 1);
        let left_node = left_nodes.last_mut().expect("左ノードが取得できません");
        let right_node = right_nodes.first_mut().expect("右ノードが取得できません");

        if left_node.keys.len() < D {
            return self.merge_left(index);
        }

        let value = self
            .keys
            .get_mut(index)
            .expect("親ノードの値が取得できません");

        let left_value = left_node.keys.remove(left_node.keys.len() - 1);
        let pivot_key = std::mem::replace(value, left_value);
        right_node.keys.insert(0, pivot_key);
    }

    fn merge_left(&mut self, index: usize) {
        let mut right_node = self.children.remove(index + 1);
        let mut left_node = self.children.remove(index);
        let value = self.keys.remove(index);
        left_node.keys.push(value);
        let right_keys = std::mem::take(&mut right_node.keys);
        left_node.keys.extend(right_keys);
        left_node.children.extend(right_node.children);

        if self.keys.is_empty() {
            std::mem::swap(self, &mut left_node);
        } else {
            self.children.insert(index, left_node);
        }
    }

    pub fn delete_intermediate(&mut self, value: &T) -> Result<(), Box<dyn std::error::Error>> {
        match self.find_key(value) {
            Ok(i) => {
                self.take_left_max(i);
            },
            Err(i) => match self.get_mut_child(i) {
                None => return Err("削除対象のノードが存在しません".into()),
                Some(node) => {
                    match node.find_key(value) {
                        Ok(i) => {
                            self.take_left_max(i);
                        },
                        Err(_) => {
                            node.delete_intermediate(value)?;
                        }
                    }
                }
            },
        }

        Ok(())
    }

    pub fn take_left_max(&mut self, index: usize) -> Option<T> {
        if self.is_leaf() {
            return Some(self.keys.remove(index));
        }

        let v = match self.children.get_mut(index) {
            Some(left) => {
                let last_index = left.keys.len() - 1;
                left.take_left_max(last_index)
                    .map(|max_value| {
                        std::mem::replace(&mut self.keys[index], max_value)
                    })
            }
            None => None,
        };

        match self.children.get_mut(index) {
            Some(left) => {
                // println!("!! {:?}, {:?}", left, index);
                if left.keys.len() < D {
                    self.pivot_left(index);
                }
            }
            _ => {}
        }

        v
    }

    pub fn find_key(&mut self, value: &T) -> Result<usize, usize> {
        self.keys.as_slice().binary_search(value)
    }

    pub fn get_mut_child(&mut self, index: usize) -> Option<&mut Node<T, D>> {
        self.children.get_mut(index)
    }

    fn is_full(&self) -> bool {
        self.keys.len() >= D * 2 - 1
    }

    fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    fn is_delete_child(&self, index: usize) -> bool {
        self.children
            .get(index)
            .expect("削除対象のキーを含むノードが取得できません")
            .keys
            .len()
            >= D
    }

    fn delete_key(&mut self, value: &T) {
        match self.keys.as_slice().binary_search(value) {
            Ok(i) => {
                self.keys.remove(i);
            }
            _ => {} // noop
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_insert() {
        let mut btree = BTree::<_, 2>::new();
        println!("{:#?}", btree);
        for i in 1..=9 {
            btree.insert(i);
        }
        println!("{:#?}", btree);
        println!(
            "value is {:?}",
            btree.iter().map(|i| *i).collect::<Vec<_>>()
        );

        let mut btree = BTree::<_, 3>::new();
        println!("{:#?}", btree);
        for i in 1..=20 {
            btree.insert(i);
        }
        println!("{:#?}", btree);
        println!(
            "value is {:?}",
            btree.iter().map(|i| *i).collect::<Vec<_>>()
        );

        let mut btree = BTree::<_, 5>::new();
        println!("{:#?}", btree);
        for i in 1..=50 {
            btree.insert(i);
        }
        println!("{:#?}", btree);
        println!(
            "value is {:?}",
            btree.iter().map(|i| *i).collect::<Vec<_>>()
        );
    }

    #[rstest::rstest]
    #[case(1, Some(0))]
    #[case(2, Some(0))]
    #[should_panic(expected = "根に対象値が記録されている")]
    #[case(3, None)]
    #[case(4, None)]
    #[case(5, Some(1))]
    #[should_panic(expected = "根に対象値が記録されている")]
    #[case(6, None)]
    #[case(7, Some(2))]
    #[should_panic(expected = "根に対象値が記録されている")]
    #[case(8, None)]
    #[case(9, Some(3))]
    fn test_find_parent_and_index(#[case] search_value: i32, #[case] index: Option<usize>) {
        let mut btree: BTree<i32, 2> = BTree::new();
        //        [3, 6, 8]
        // [1,2] | [5] | [7] | [9]
        for i in [1, 2, 3, 5, 6, 7, 8, 9].into_iter().rev() {
            btree.insert(i);
        }

        let search_result = btree.find_parent_and_index(&search_value);
        if index.is_some() {
            assert!(search_result.is_some());
            let search_result = search_result.unwrap();
            assert_eq!(search_result.0.keys, vec![3, 6, 8]);
            assert_eq!(search_result.1, index.unwrap());
        } else {
            assert!(search_result.is_none());
        }
    }

    #[test]
    fn test_delete_leaf_node() -> Result<(), Box<dyn std::error::Error>> {
        let mut btree: BTree<i32, 2> = BTree::new();
        //        [3, 6, 8]
        // [1,2] | [5] | [7] | [9]
        for i in [1, 2, 3, 5, 6, 7, 8, 9].into_iter().rev() {
            btree.insert(i);
        }

        println!("{:#?}", btree);
        btree.delete(&1)?;
        println!("{:#?}", btree);
        //        [3, 6, 8]
        // [2] | [5] | [7] | [9]
        btree.delete(&2)?;
        println!("{:#?}", btree);
        //        [6, 8]
        // [3, 5] | [7] | [9]
        btree.delete(&9)?;
        println!("{:#?}", btree);
        btree.delete(&5)?;
        println!("{:#?}", btree);
        btree.delete(&8)?;
        println!("{:#?}", btree);

        Ok(())
    }

    #[test]
    fn test_delete_intermediate_node() -> Result<(), Box<dyn std::error::Error>> {
        let mut btree: BTree<i32, 2> = BTree::new();
        //             [4]
        //    [2]       |   [ 6,    8,   10 ]
        // [1] | [3]      [5] | [7] | [9] | [11,12]

        //             [2]
        //    [1]       |  [  6,    8,   10  ]
        // [ ] | [3]      [5] | [7] | [9] | [11,12]

        // [2]
        // [1, 3]

        for i in 1..=12 {
            btree.insert(i);
        }
        println!("{:#?}", btree);
        btree.delete(&2)?;
        println!("{:#?}", btree);

        Ok(())
    }
}
