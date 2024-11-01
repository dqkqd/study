use std::{
    collections::BTreeMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use rand::RngCore;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct NodeId(u32);

enum Node<T> {
    Virtual(VirtualNode),
    Real(RealNode<T>),
}

#[derive(Debug, Clone)]
struct VirtualNode {
    id: NodeId,
    real_node_id: NodeId,
}

#[derive(Debug)]
struct RealNode<T> {
    id: NodeId,
    data: Vec<T>,
    virtual_nodes: Vec<VirtualNode>,
}

struct HashRing<T> {
    vnodes_per_node: u32,
    nodes: BTreeMap<NodeId, Node<T>>,
    cache_miss: u64,
}

fn calculate_hash<T: Hash>(t: &T) -> u32 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    (s.finish() >> 32) as u32
}

impl<T: Hash> HashRing<T> {
    fn new(vnodes_per_node: u32) -> HashRing<T> {
        HashRing {
            vnodes_per_node,
            nodes: BTreeMap::default(),
            cache_miss: 0,
        }
    }

    fn get_node(&self, node_id: &NodeId) -> Option<&RealNode<T>> {
        let (_, mut node) = self
            .nodes
            .range(node_id..)
            .next()
            .or_else(|| self.nodes.range(..node_id).next())?;

        loop {
            node = match node {
                Node::Virtual(vnode) => self.nodes.get(&vnode.real_node_id)?,
                Node::Real(node) => return Some(node),
            }
        }
    }

    fn get_node_mut(&mut self, node_id: &NodeId) -> Option<&mut RealNode<T>> {
        let next_node = self.get_node(node_id)?;
        let node_id = next_node.id;
        match self.nodes.get_mut(&node_id) {
            Some(Node::Real(node)) => Some(node),
            _ => unreachable!(),
        }
    }

    fn add_node(&mut self, node_id: NodeId) -> bool {
        if self.nodes.contains_key(&node_id) {
            return false;
        }

        let mut node = RealNode::new(node_id);
        let mut rng = rand::thread_rng();
        for _ in 0..self.vnodes_per_node {
            let virtual_id = NodeId(rng.next_u32());
            if self.nodes.contains_key(&virtual_id) {
                continue;
            }
            let virtual_node = VirtualNode {
                id: virtual_id,
                real_node_id: node_id,
            };
            node.add_virtual_node(virtual_node.clone());
            self.nodes
                .insert(virtual_node.id, Node::Virtual(virtual_node));
        }
        self.nodes.insert(node_id, Node::Real(node));

        true
    }

    fn remove_node(&mut self, node_id: NodeId) {
        if let Some(node) = self.get_node(&node_id) {
            let vnode_ids: Vec<NodeId> = node.virtual_nodes.iter().map(|vnode| vnode.id).collect();
            for node_id in vnode_ids {
                self.nodes.remove(&node_id);
            }
            self.nodes.remove(&node_id);
        }
    }
}

impl<T> HashRing<T>
where
    T: Hash + Eq + PartialEq,
{
    fn query(&mut self, data: T) -> &RealNode<T> {
        let node_id = NodeId(calculate_hash(&data));
        let node = self.get_node(&node_id).expect("At least 1 server exists");
        let has_data = node.has_data(&data);
        if !has_data {
            self.cache_miss += 1;
        }

        let node = self.get_node_mut(&node_id).unwrap();
        if !has_data {
            node.add_data(data)
        }

        &*node
    }
}

impl<T> RealNode<T> {
    fn new(id: NodeId) -> RealNode<T> {
        RealNode {
            id,
            data: Vec::new(),
            virtual_nodes: Vec::new(),
        }
    }

    fn add_data(&mut self, data: T) {
        self.data.push(data)
    }

    fn add_virtual_node(&mut self, virtual_node: VirtualNode) {
        self.virtual_nodes.push(virtual_node)
    }
}

impl<T> RealNode<T>
where
    T: Eq + PartialEq,
{
    fn has_data(&self, data: &T) -> bool {
        self.data.iter().any(|d| d == data)
    }
}

fn main() {
    let mut ring = HashRing::new(5);
    let node_ids = vec![NodeId(0), NodeId(20), NodeId(40)];
    for node_id in &node_ids {
        ring.add_node(*node_id);
    }

    ring.query("foo");
    assert_eq!(ring.cache_miss, 1);

    ring.query("bar");
    assert_eq!(ring.cache_miss, 2);

    ring.query("foo");
    assert_eq!(ring.cache_miss, 2);

    let node = ring.query("baz");
    let node_id = node.id;
    for id in node_ids {
        if id != node_id {
            ring.remove_node(id);
        }
    }
    assert_eq!(ring.cache_miss, 3);

    // baz will not be rehashed
    ring.query("baz");
    assert_eq!(ring.cache_miss, 3);
}
