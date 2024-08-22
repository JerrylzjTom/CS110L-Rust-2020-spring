use linked_list_generic::LinkedList;
pub mod linked_list;
pub mod linked_list_generic;

fn main() {
    let mut list: LinkedList<u32> = LinkedList::new();
    assert!(list.is_empty());
    assert_eq!(list.get_size(), 0);
    for i in 1..12 {
        list.push_front(i);
    }
    let mut list2: LinkedList<u32> = LinkedList::new();
    for i in 1..12 {
        list2.push_front(i);
    }
    let list3 = list.clone();
    println!("list1:{}", list.to_string()); // ToString impl for anything impl Display
    println!("list2:{}", list2.to_string());
    println!("list3:{}", list3.to_string());
    println!("{}", list == list2);
    println!("{}", list == list3);
    // If you implement iterator trait:
    // for val in list {
    //    println!("{}", val);
    // }
}
