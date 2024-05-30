use std::any::TypeId;
use std::mem::{align_of, size_of};
use std::ptr;

pub struct ComponentMeta {
    id: TypeId,
    size: usize,
    align: usize,
    drop_fn: Option<fn(*mut u8)>,
}

impl ComponentMeta {
    pub fn of<C: Sized + 'static>() -> Self {
        ComponentMeta {
            id: TypeId::of::<C>(),
            size: size_of::<C>(),
            align: align_of::<C>(),
            drop_fn: if std::mem::needs_drop::<C>() {
                Some(|ptr| unsafe { std::ptr::drop_in_place(ptr as *mut C) })
            } else {
                None
            },
        }
    }
}

pub struct Layout {
    components: Vec<ComponentMeta>,
}

impl Layout {
    pub fn new() -> Self {
        Layout {
            components: Vec::new(),
        }
    }

    pub fn add_component<C: Sized + 'static>(&mut self) {
        self.components.push(ComponentMeta::of::<C>());
    }

    pub fn has_component<C: Sized + 'static>(&self) -> bool {
        let id = TypeId::of::<C>();
        self.has_component_id(id)
    }

    pub fn has_component_id(&self, id: TypeId) -> bool {
        self.components.iter().any(|c| c.id == id)
    }

    pub fn matches(&self, components: &[TypeId]) -> bool {
        self.components.len() == components.len() && components.iter().all(|id| self.has_component_id(*id))
    }
}

pub struct Archtype {
    entities: Vec<u32>,
    components: Vec<*mut u8>,
    capacity: usize,
    size: usize,
    layout: Layout,
}

impl Archtype {
    pub fn new(layout: Layout, capacity: usize) -> Self {
        let mut components = Vec::with_capacity(layout.components.len());

        for c in &layout.components {
            let buf = unsafe { std::alloc::alloc(std::alloc::Layout::from_size_align_unchecked(c.size * capacity, c.align)) };
            components.push(buf);
        }

        Self {
            components,
            capacity,
            layout,
            size: 0,
            entities: Vec::new(),
        }
    }

    unsafe fn get_ptr<C: 'static>(&mut self) -> *mut C {
        for (i, l) in self.layout.components.iter().enumerate() {
            if l.id == TypeId::of::<C>() {
                return self.components[i] as *mut C;
            }
        }
        return ptr::null_mut();
    }
}

pub trait Component: Sized + 'static {}


pub trait ArchtypeOps<C> {
    fn add_components(&mut self, components: C);
}

impl<A: Component> ArchtypeOps<A> for Archtype {
    fn add_components(&mut self, component: A) {
        unsafe {
            let ptr = self.get_ptr::<A>();
            assert!(!ptr.is_null());
            let ptr = ptr.add(self.size);
            ptr::write(ptr, component);
        };
    }
}


impl<A: Component, B: Component> ArchtypeOps<(A, B)> for Archtype {
    fn add_components(&mut self, components: (A, B)) {
        assert_ne!(TypeId::of::<A>(), TypeId::of::<B>());

        self.add_components(components.0);
        self.add_components(components.1);
    }
}

impl Drop for Archtype {
    fn drop(&mut self) {
        for (&c, l) in self.components.iter().zip(&self.layout.components) {
            unsafe {
                if let Some(drop_fn) = l.drop_fn {
                    for i in (0..self.size) {
                        drop_fn(c.add(i))
                    }
                }

                std::alloc::dealloc(c, std::alloc::Layout::from_size_align_unchecked(l.size * self.capacity, l.align));
            }
        }
    }
}

