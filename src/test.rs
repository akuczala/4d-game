use itertools::Itertools;

struct LineSource {
	lines : Vec<i32>,
	index : usize
}
impl LineSource {
	fn new() -> Self {
		LineSource{
			lines: vec![1,2,3,4],
			index: 0
		}
	}
}
impl Iterator for LineSource {
	type Item = i32;
	fn next(&mut self) -> Option<Self::Item> {
		if self.index < self.lines.len() - 1 {
			let out = Some(self.lines[self.index]);
			self.index += 1;
			out
		} else {
			None
		}
		
	}
}

struct TransformSource {
	line_source : LineSource,
	add_vec : Vec<i32>,
	index : usize
}
impl TransformSource {
	fn new() -> Self {
		TransformSource{
			line_source : LineSource::new(),
			add_vec : vec![1,2],
			index: 0
		}
	}
}
impl Iterator for TransformSource {
	type Item = i32;
	fn next(&mut self) -> Option<Self::Item> {
		if self.index < self.add_vec.len() - 1 {
			let out = line_source.next() + add_vec[i]
			self.index += 1;
			out
		} else {
			None
		}
		
	}
}