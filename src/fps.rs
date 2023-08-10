//thanks to https://stackoverflow.com/a/61547339

pub type FPSFloat = f64;
pub const TARGET_FPS: FPSFloat = 60.0;
pub const N_TIMES: usize = 50;
use std::time::Instant;

pub struct FPSTimer {
    pub start: Instant,
    fps: Option<FPSFloat>,
    elapsed_time_list: [u64; N_TIMES],
    i: usize,
    pub elapsed_time: u64,
    pub gui_last_time: Instant,
}
impl FPSTimer {
    pub fn new() -> Self {
        FPSTimer {
            start: Instant::now(),
            elapsed_time_list: [0; N_TIMES],
            gui_last_time: Instant::now(),
            fps: None,
            i: 0,
            elapsed_time: 0,
        }
    }

    //get time at start of pass
    pub fn start(&mut self) {
        self.start = Instant::now();
    }

    //return extra time needed to sleep at end of pass
    pub fn end(&mut self) {
        self.elapsed_time = Instant::now().duration_since(self.start).as_millis() as u64;
        // let wait_millis = match 1000 / (TARGET_FPS as u64) >= self.elapsed_time {
        //               true => 1000 / (TARGET_FPS as u64) - self.elapsed_time,
        //               false => 0
        //       };

        self.update_fps(self.elapsed_time);

        //self.debug(self.elapsed_time, wait_millis);
    }

    //print time (ms) taken on this pass, as well as how many ms to wait
    #[allow(dead_code)]
    fn debug(&mut self, elapsed_time: u64, wait_millis: u64) {
        self.elapsed_time_list[self.i] = self.elapsed_time;
        self.i = (self.i + 1) % N_TIMES;
        if self.i == 0 {
            println!("{0}, {1}", elapsed_time, wait_millis);
            println!("{:?}", self.fps);
            let mut out_str = "".to_string();
            let mut mean: f32 = 0.;
            for &e in self.elapsed_time_list.iter() {
                mean += e.max(16) as f32;
                out_str = format!("{} {}", out_str, e);
            }
            let mean = mean / (N_TIMES as f32);
            println!("{}", mean);
            println!("{}", out_str);
        }
    }

    //compute instantaneous fps (seems to work better than some kind of average)
    fn update_fps(&mut self, elapsed_time: u64) {
        // self.time_sum += elapsed_time.max(16);
        // if self.i == N_TIMES - 1 {
        // 	let mean_milli = (self.time_sum as FPSFloat)/(N_TIMES as FPSFloat);
        // 	self.fps = match self.fps {
        // 		//Some(f) => Some((f + 1000./mean_milli)/2.),
        // 		Some(f) => Some(1000./mean_milli) ,
        // 		None =>  Some(1000./mean_milli),
        // 	};
        // 	self.time_sum = 0;
        // }
        self.fps = {
            let frame_seconds = match elapsed_time {
                0 => (elapsed_time as FPSFloat) / 1000.,
                _ => 1.0 / TARGET_FPS,
            };
            Some(1.0 / frame_seconds)
        };

        // //println!("{:?}",cur_fps);
        // self.fps = match self.fps {
        // 	//Some(fps_val) => Some((fps_val + cur_fps)/2.0),
        // 	_ => Some(cur_fps)
        // }
    }

    #[allow(dead_code)]
    pub fn get_fps(&self) -> FPSFloat {
        match self.fps {
            Some(f) => f,
            None => TARGET_FPS,
        }
    }
    pub fn get_frame_length(&self) -> FPSFloat {
        //println!("{:?}",self.get_fps());
        //1.0/self.get_fps()
        (self.elapsed_time.max(16) as FPSFloat) / 1000.
    }
}
