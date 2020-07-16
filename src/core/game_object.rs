use bit_vec::BitVec;
use crate::core::{Component, ComponentType};
use std::cell::RefCell;
use std::rc::Rc;

pub struct GameObject
{
	components: Vec<Rc<RefCell<dyn Component>>>,
	active_components: BitVec,
	pub children: Vec<GameObject>,
}

impl GameObject
{
	pub fn new() -> GameObject
	{
		return GameObject {
			components: Vec::new(),
			active_components: BitVec::from_elem(ComponentType::LENGTH_OF_ENUM as usize, false),
			children: Vec::new(),
		};
	}

	pub fn add_child(&mut self, child: GameObject)
	{
		self.children.push(child);
	}

	pub fn add_component<T: Component + 'static>(&mut self, component: T)
	{
		let component_type = component.get_component_type() as usize;
		// Don't add components that are already set
		debug_assert!(!self.active_components.get(component_type).unwrap());
		self.components.push(Rc::new(RefCell::new(component)));
		self.active_components.set(component_type, true);
	}

	pub fn has_component(&self, component_type: ComponentType) -> bool
	{
		return self.active_components.get(component_type as usize).unwrap();
	}

	pub fn get_component(&mut self, component_type: ComponentType) -> Option<Rc<RefCell<dyn Component>>>
	{
		if self.has_component(component_type)
		{
			for component in self.components.iter_mut()
			{
				if component.borrow().get_component_type() == component_type
				{
					return Some(component.clone());
				}
			}
		}
		return None;
	}
}
