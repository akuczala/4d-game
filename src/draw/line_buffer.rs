use crate::vector::{VectorTrait};
use super::DrawLine;

// pub struct LineBuffer<V : VectorTrait>{
// 	lines : Vec<Option<DrawLine<V>>>, //this is an option type for now, but likely won't need it
// 	cur_index : usize,
// 	cur_size : usize,
// }

// impl<V : VectorTrait> LineBuffer<V> {
// 	pub fn new() -> Self {
// 		LineBuffer{
// 			lines : Vec::new(),
// 			cur_index : 0,
// 			cur_size : 0
// 		}
// 	}
// 	pub fn add(&mut self, line : Option<DrawLine<V>>) {
// 		if self.cur_index < self.lines.len() {
// 			self.lines[self.cur_index] = line;
// 		} else {
// 			self.lines.push(line);
// 		}
// 		self.cur_index +=1 ;
// 		self.cur_size +=1 ;
// 	}
// 	//apply fn? indexing
// 	pub fn map_verts<F : Fn(V) -> V>(&mut self, f : F) {
// 		for i in 0..self.cur_size {
// 			self.lines[i] = self.lines[i].clone()
// 			.map(|draw_line| draw_line
// 				.map_line(|line| line
// 					.map(|v| f(v))));
// 		}
// 	}

// 	pub fn new_batch(&mut self) {
// 		self.cur_index = 0;
// 		self.cur_size = 0;
// 	}
// }

pub struct Buffer<T : Clone>{
	vec : Vec<T>,
	cur_index : usize,
	pub cur_size : usize,
}
impl<T : Clone> Buffer<T> {
	pub fn new() -> Self {
		Buffer{
			vec : Vec::new(),
			cur_index : 0,
			cur_size : 0
		}
	}
	fn check_index(&self,index : usize) {
		if index >= self.cur_size {
			panic!("Tried to access bus at {} but bus has size {}",index,self.cur_size);
		}
	}
	//adds element or modifies element at next index. increment index by 1.
	pub fn add(&mut self, new_val : T) {
		if self.vec.len() == 0 {
			self.vec.push(new_val);
			//keep cur_index at 0
		} else {
			//if next index is larger than vec
			if self.cur_index + 1 == self.vec.len() {
				self.vec.push(new_val);
			} else {
				self.vec[self.cur_index + 1] = new_val;
				self.cur_index += 1;
			}
		}
		self.cur_size = self.cur_size.max(self.cur_index + 1);
	}
	//append element to end of buffer. do not change current index.
	pub fn add_to_end(&mut self, new_val : T) {
		if self.vec.len() == 0 {
			self.add(new_val);
		} else {
			if self.cur_size == self.vec.len() {
				self.vec.push(new_val);
				
			} else {
				self.vec[self.cur_size] = new_val;
			}
		}
		self.cur_size += 1;
	}
	pub fn get_ref(&self, index : usize) -> &T {
		self.check_index(index);
		&self.vec[index]

	}
	pub fn get_mut_ref(&mut self, index : usize) -> &mut T {
		self.check_index(index);
		&mut self.vec[index]
	}
	pub fn set(&mut self, index : usize, value  : T) {
		self.check_index(index);
		self.vec[index] = value;
	}
	pub fn clear(&mut self) {
		self.cur_index = 0;
		self.cur_size = 0;
	}
	pub fn to_beginning(&mut self) {
		self.cur_index = 0;
	}
	pub fn copy_to_buffer(&mut self, target : &mut Buffer<T>) {
		self.to_beginning();
		for i in 0..self.cur_size {
			target.add(self.get_ref(i).clone());
		}
	}
	pub fn get_slice(&self) -> &[T] {
		&self.vec[0..self.cur_size]
	}
}
