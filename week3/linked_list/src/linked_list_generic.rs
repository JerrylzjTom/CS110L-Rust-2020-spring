use std::fmt::{self, Display};
use std::option::Option;

pub struct LinkedList<T> {
    head: Option<Box<Node<T>>>,
    size: usize,
}

struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    pub fn new(value: T, next: Option<Box<Node<T>>>) -> Node<T> {
        Node {value: value, next: next}
    }
}

impl<T> LinkedList<T> {
    pub fn new() -> LinkedList<T> {
        LinkedList {head: None, size: 0}
    }

    pub fn get_size(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.get_size() == 0
    }

    pub fn push_front(&mut self, value: T) {
        let new_node = Box::new(Node::new(value, self.head.take()));
        self.head = Some(new_node);
        self.size += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        let node = self.head.take()?;
        self.head = node.next;
        self.size -= 1;
        Some(node.value)
    }

    pub fn push_tail(&mut self, value: T) {
        let mut current = &mut self.head;
        while let Some(node) = current {
            current = &mut node.next;
        }
        *current = Some(Box::new(Node::new(value, None)));
        self.size += 1;
    }
}

impl<T: Display> fmt::Display for LinkedList<T> {
    
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut current = &self.head;
        let mut result = String::new();
        loop {
            match current {
                Some(node) => {
                    result = format!("{} {}", result, node.value);
                    current = &node.next;
                },
                None => break,
            }
        }
        write!(f, "{}", result)
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        let mut current = self.head.take();
        while let Some(mut node) = current {
            current = node.next.take();
        }
    }
}

impl<T: Clone> Clone for LinkedList<T> {
    fn clone(&self) -> Self {
        let mut new_list = LinkedList::new();
        let mut current = &self.head;
        while let Some(node) = current {
            new_list.push_tail(node.value.clone());
            current = &node.next;
        }
        new_list
    }
}

impl<T: PartialEq> PartialEq for LinkedList<T> {
    fn eq(&self, other: &Self) -> bool {
        let mut current1 = &self.head;
        let mut current2 = &other.head;
        while let (Some(node1), Some(node2)) = (current1, current2) {
            if node1.value != node2.value {
                return false;
            }
            current1 = &node1.next;
            current2 = &node2.next;
        }
        current1.is_none() && current2.is_none()
    }
}