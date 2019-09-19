// use std::iter::Iterator;
use std::ops;

#[derive(Clone, PartialEq, Debug)]
pub struct Stack<T> {
    items: Vec<T>,
}

//@todo probably just make this all Buffer?

impl<T> Stack<T> {
    pub fn new() -> Self {
        Stack { items: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    //@todo probably should be inline, all of these here should be.
    pub fn push(&mut self, item: T) {
        self.items.push(item)
    }

    pub fn set(&mut self, index: usize, item: T) {
        //@todo necessary?
        // if (index < 0)
        //   index += this.items.length;

        //@todo should this be an assert or Result?.
        assert!(index <= self.items.len());

        self.items[index] = item;
    }
}

impl<T> ops::Deref for Stack<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl<T> ops::DerefMut for Stack<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}

//TODO ideally we do this for now we are just going to use Deref
// impl<T> Iterator for Stack<T> {
//     type Item = T;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.items.next()
//     }
// }
