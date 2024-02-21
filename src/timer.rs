use std::time::Instant;



/// #### 한국어 </br>
/// `tick`함수를 호출할 때 까지의 걸린 시간을 측정합니다. </br>
/// 
/// #### English (Translation) </br>
/// Measures the time it takes to call the `tick` function. </br>
/// 
#[derive(Debug, Clone, Copy)]
pub struct GameTimer<const NUM_SAMPLE: usize = 50> {
    previous_timepoint: Instant, 
    current_timepoint: Instant, 

    frame_times: [f64; NUM_SAMPLE],
    cnt_frame_times: usize, 

    elapsed_time_sec: f64,
    fps_elapsed_time_sec: f64, 
    frame_per_seconds: u64, 
    frame_rate: u64, 
}

impl<const NUM_SAMPLES: usize> GameTimer<NUM_SAMPLES> {
    #[inline]
    pub fn new() -> Self {
        let timepoint = Instant::now();
        Self {
            previous_timepoint: timepoint, 
            current_timepoint: timepoint, 
            frame_times: [0.0; NUM_SAMPLES], 
            cnt_frame_times: 0, 
            elapsed_time_sec: 0.0, 
            fps_elapsed_time_sec: 0.0, 
            frame_per_seconds: 0, 
            frame_rate: 0,
        }
    }

    /// #### 한국어 </br>
    /// 이전 `tick` 함수를 호출한 시점에서 현재 `tick`함수를 호출한 시점까지 걸린 시간을 측정합니다. </br>
    /// 
    /// #### English (Translation) </br>
    /// Measures the time taken from calling the previous `tick` function to calling the current `tick` function. </br>
    /// 
    pub fn tick(&mut self) {
        self.current_timepoint = Instant::now();
        let elapsed_time_sec = self.current_timepoint
            .saturating_duration_since(self.previous_timepoint)
            .as_secs_f64();

        self.previous_timepoint = self.current_timepoint;

        if (self.elapsed_time_sec - elapsed_time_sec).abs() < 1.0 {
            self.frame_times.copy_within(0..(NUM_SAMPLES - 1), 1);
            self.frame_times[0] = elapsed_time_sec;
            self.cnt_frame_times = (self.cnt_frame_times + 1).min(NUM_SAMPLES);
        }

        self.frame_per_seconds += 1;
        self.fps_elapsed_time_sec += elapsed_time_sec;
        if self.fps_elapsed_time_sec > 1.0 {
            self.frame_rate = self.frame_per_seconds;
            self.frame_per_seconds = 0;
            self.fps_elapsed_time_sec -= 1.0;
        }

        self.elapsed_time_sec = 0.0;
        if self.cnt_frame_times > 0 {
            self.elapsed_time_sec = self.frame_times
                .iter()
                .take(self.cnt_frame_times)
                .sum();
            self.elapsed_time_sec /= self.cnt_frame_times as f64;
        }
    }

    #[inline]
    pub fn elapsed_time_sec(&self) -> f32 {
        self.elapsed_time_sec as f32
    }

    #[inline]
    pub fn frame_rate(&self) -> u32 {
        self.frame_rate as u32
    }
}