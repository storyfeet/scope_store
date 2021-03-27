use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

struct Scope<T> {
    data: BTreeMap<String, T>,
    parent: Option<PScope<T>>,
    root: Option<PScope<T>>,
}

impl<T> Scope<T> {
    fn new() -> Self {
        Self {
            data: BTreeMap::new(),
            parent: None,
            root: None,
        }
    }
    pub fn set_local(&mut self, id: String, val: T) {
        self.data.insert(id, val);
    }

    pub fn set_global(&mut self, id: String, val: T) {
        match &self.root {
            Some(v) => v.set(id, val),
            None => drop(self.data.insert(id, val)),
        }
    }

    pub fn set(&mut self, id: String, val: T) {
        if let Some(_) = self.data.get(&id) {
            self.data.insert(id, val);
            return;
        }
        match &self.parent {
            Some(p) => {
                if let Some(v) = p.try_replace(&id, val) {
                    self.data.insert(id, v);
                }
            }
            None => drop(self.data.insert(id, val)),
        }
    }

    /// Some<T> means not set, use T, to insert in local scope
    fn try_replace(&mut self, id: &str, val: T) -> Option<T> {
        if let Some(v) = self.data.get_mut(id) {
            *v = val;
            return None;
        }
        match &self.parent {
            Some(p) => p.p.borrow_mut().try_replace(id, val),
            None => Some(val),
        }
    }

    pub fn update<F: Fn(&mut T) -> A, A>(&mut self, k: &str, f: F) -> Option<A> {
        if let Some(v) = self.data.get_mut(k) {
            return Some(f(v));
        }
        match &self.parent {
            Some(v) => v.update(k, f),
            None => None,
        }
    }
}

impl<T: Clone> Scope<T> {
    pub fn get(&self, k: &str) -> Option<T> {
        if let Some(v) = self.data.get(k) {
            return Some(v.clone());
        }
        match &self.parent {
            Some(p) => p.get(k),
            None => None,
        }
    }
}

pub struct PScope<T> {
    p: Rc<RefCell<Scope<T>>>,
}

impl<T> Clone for PScope<T> {
    fn clone(&self) -> Self {
        PScope { p: self.p.clone() }
    }
}

///
/// ```rust
/// use scope_store::PScope;
/// let root = PScope::new();
///
/// let a1 = root.child();
///
/// a1.set_global("a".to_string(), 23);
/// assert_eq!(root.get("a"), Some(23));
/// assert_eq!(root.get("b"), None);
///
/// root.update("a", |n| *n += 1);
/// assert_eq!(a1.get("a"), Some(24));
///
/// let b1 = a1.child();
/// let a2 = root.child();
/// a1.set_local("cat".to_string(), 7);
/// assert_eq!(b1.get("cat"), Some(7));
/// assert_eq!(a2.get("cat"), None);
///
/// a1.set_global("dog".to_string(), 8);
/// assert_eq!(b1.get("dog"), Some(8));
/// assert_eq!(a2.get("dog"), Some(8));
///
/// assert_eq!(
///     b1.update("cat", |n| {
///         *n += 3;
///         *n
///     }),
///     Some(10)
/// );
/// assert_eq!(a1.get("cat"),Some(10));
/// ```
///
impl<T> PScope<T> {
    pub fn new() -> Self {
        PScope {
            p: Rc::new(RefCell::new(Scope::new())),
        }
    }

    pub fn set_local(&self, id: String, val: T) {
        self.p.borrow_mut().set_local(id, val);
    }

    pub fn set_global(&self, id: String, val: T) {
        self.p.borrow_mut().set_global(id, val);
    }

    pub fn set(&self, id: String, val: T) {
        self.p.borrow_mut().set(id, val);
    }

    pub fn try_replace(&self, id: &str, val: T) -> Option<T> {
        self.p.borrow_mut().try_replace(id, val)
    }

    pub fn update<F: Fn(&mut T) -> A, A>(&self, id: &str, f: F) -> Option<A> {
        self.p.borrow_mut().update(id, f)
    }

    pub fn child(&self) -> Self {
        let root = match &self.p.borrow().root {
            Some(r) => Some(r.clone()),
            None => Some(self.clone()),
        };
        let parent = Some(self.clone());
        PScope {
            p: Rc::new(RefCell::new(Scope {
                data: BTreeMap::new(),
                root,
                parent,
            })),
        }
    }
}
impl<T: Clone> PScope<T> {
    pub fn get(&self, id: &str) -> Option<T> {
        self.p.borrow().get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let root = PScope::new();

        let a1 = root.child();

        a1.set_global("a".to_string(), 23);
        assert_eq!(root.get("a"), Some(23));
        assert_eq!(root.get("b"), None);

        root.update("a", |n| *n += 1);
        assert_eq!(a1.get("a"), Some(24));

        let b1 = a1.child();
        let a2 = root.child();
        a1.set_local("cat".to_string(), 7);
        assert_eq!(b1.get("cat"), Some(7));
        assert_eq!(a2.get("cat"), None);

        a1.set_global("dog".to_string(), 8);
        assert_eq!(b1.get("dog"), Some(8));
        assert_eq!(a2.get("dog"), Some(8));

        assert_eq!(
            b1.update("cat", |n| {
                *n += 3;
                *n
            }),
            Some(10)
        );
    }
}
