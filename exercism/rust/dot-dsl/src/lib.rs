pub mod graph {
    use std::collections::HashMap;

    use graph_items::{edge::Edge, node::Node};

    #[derive(Debug, Default)]
    pub struct Graph {
        pub edges: Vec<Edge>,
        pub nodes: Vec<Node>,
        pub attrs: HashMap<String, String>,
    }

    impl Graph {
        pub fn new() -> Self {
            Self::default()
        }
        pub fn node(&self, name: &str) -> Option<&Node> {
            self.nodes.iter().find(|node| node.name == name)
        }
        pub fn with_nodes(mut self, nodes: &[Node]) -> Self {
            self.nodes = nodes.to_vec();
            self
        }
        pub fn with_edges(mut self, edges: &[Edge]) -> Self {
            self.edges = edges.to_vec();
            self
        }
        pub fn with_attrs(mut self, attrs: &[(&str, &str)]) -> Self {
            self.attrs =
                HashMap::from_iter(attrs.iter().map(|(k, v)| (k.to_string(), v.to_string())));
            self
        }
    }

    pub mod graph_items {
        pub mod edge {
            use std::collections::HashMap;
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct Edge {
                from: String,
                to: String,
                attrs: HashMap<String, String>,
            }
            impl Edge {
                pub fn new(from: &str, to: &str) -> Self {
                    Self {
                        from: from.into(),
                        to: to.into(),
                        attrs: HashMap::new(),
                    }
                }
                pub fn attr(&self, attr: &str) -> Option<&str> {
                    self.attrs.get(attr).map(|v| v.as_str())
                }
                pub fn with_attrs(mut self, attrs: &[(&str, &str)]) -> Self {
                    self.attrs = HashMap::from_iter(
                        attrs.iter().map(|(k, v)| (k.to_string(), v.to_string())),
                    );
                    self
                }
            }
        }
        pub mod node {
            use std::collections::HashMap;

            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct Node {
                pub name: String,
                pub attrs: HashMap<String, String>,
            }

            impl Node {
                pub fn new(name: &str) -> Self {
                    Self {
                        name: name.into(),
                        attrs: HashMap::default(),
                    }
                }
                pub fn attr(&self, attr: &str) -> Option<&str> {
                    self.attrs.get(attr).map(|v| v.as_str())
                }
                pub fn with_attrs(mut self, attrs: &[(&str, &str)]) -> Self {
                    self.attrs = HashMap::from_iter(
                        attrs.iter().map(|(k, v)| (k.to_string(), v.to_string())),
                    );
                    self
                }
            }
        }
    }
}
