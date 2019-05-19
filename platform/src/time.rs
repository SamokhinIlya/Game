use core::mem;
use winapi::um::profileapi::{QueryPerformanceCounter, QueryPerformanceFrequency};

lazy_static::lazy_static! {
    static ref PERFORMANCE_FREQUENCY: i64 = unsafe {
        let mut pf = mem::uninitialized();
        QueryPerformanceFrequency(&mut pf);
        *pf.QuadPart()
    };
}

pub struct Counter {
    start_ticks: i64,
}

impl Counter {
    pub fn start() -> Self {
        Self { start_ticks: Self::count() }
    }

    pub fn elapsed(&self) -> TicksElapsed {
        TicksElapsed(Self::count() - self.start_ticks)
    }

    pub fn end(self) -> TicksElapsed {
        TicksElapsed(Self::count() - self.start_ticks)
    }

    fn count() -> i64 {
        unsafe {
            let mut performance_count = mem::uninitialized();
            QueryPerformanceCounter(&mut performance_count);
            *performance_count.QuadPart()
        }
    }
}

#[derive(Copy, Clone)]
pub struct TicksElapsed(i64);

impl TicksElapsed {
    pub fn as_secs(self) -> f64 {
        self.0 as f64 / unsafe { *PERFORMANCE_FREQUENCY } as f64
    }

    pub fn as_ms(self) -> f64 {
        (self.0 * 1000) as f64 / unsafe { *PERFORMANCE_FREQUENCY } as f64
    }

    pub fn as_micros(self) -> f64 {
        (self.0 * 1_000_000) as f64 / unsafe { *PERFORMANCE_FREQUENCY } as f64
    }

    pub fn as_nanos(self) -> f64 {
        (self.0 * 1_000_000_000) as f64 / unsafe { *PERFORMANCE_FREQUENCY } as f64
    }
}
