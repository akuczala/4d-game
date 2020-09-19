//thanks to https://stackoverflow.com/a/61547339

pub type FPSFloat = f64;
pub const TARGET_FPS : FPSFloat = 60.0;

use std::time::{Instant};

pub struct FPSTimer {
    start : Instant,
    fps : Option<FPSFloat>,
    i : i32,
}
impl FPSTimer {

	pub fn new() -> Self {
		FPSTimer{start : Instant::now(), fps : None, i : 0}
	}

	//get time at start of pass
	pub fn start(&mut self) {
		self.start = Instant::now();
	}

	//return extra time needed to sleep at end of pass
	pub fn end(&mut self) -> Instant {

		let elapsed_time = Instant::now().duration_since(self.start).as_millis() as u64;
		let wait_millis = match 1000 / (TARGET_FPS as u64) >= elapsed_time {
                true => 1000 / (TARGET_FPS as u64) - elapsed_time,
                false => 0
        };

        self.update_fps(elapsed_time, wait_millis);

        //self.debug(elapsed_time, wait_millis);

        let new_inst = self.start + std::time::Duration::from_millis(wait_millis);
        new_inst
	}

	//print time (ms) taken on this pass, as well as how many ms to wait
	#[allow(dead_code)]
	fn debug(&mut self, elapsed_time : u64, wait_millis : u64) {
		if self.i == 0 {
			println!("{0}, {1}",elapsed_time,wait_millis);
			println!("{:?}", self.fps);
		}
		self.i = (self.i + 1) % 300;
	}

	//compute instantaneous fps (seems to work better than some kind of average)
	fn update_fps(&mut self, elapsed_time : u64, wait_millis : u64) {
		let cur_fps = {
			let frame_seconds = match wait_millis {
				0 => (elapsed_time as FPSFloat)/1000.,
				_ => 1.0/TARGET_FPS,
			};
			1.0/frame_seconds
		};

		//println!("{:?}",cur_fps);
		self.fps = match self.fps {
			//Some(fps_val) => Some((fps_val + cur_fps)/2.0),
			_ => Some(cur_fps)
		}
	}
	pub fn get_fps(&self) -> FPSFloat {
		match self.fps {
			Some(f) => f,
			None => TARGET_FPS
		}
	}
	pub fn get_frame_length(&self) -> FPSFloat {
		//println!("{:?}",self.get_fps());
		1.0/self.get_fps()
	}
}