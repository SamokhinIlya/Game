use core::mem;
use winapi::um::profileapi::{QueryPerformanceCounter, QueryPerformanceFrequency};

//TODO: consider lazy_static
static mut PERFORMANCE_FREQUENCY: i64 = 0;

pub fn init() {
    unsafe {
        let mut pf = mem::uninitialized();
        QueryPerformanceFrequency(&mut pf);
        PERFORMANCE_FREQUENCY = *pf.QuadPart();
    }
}

pub struct Counter {
    start_ticks: i64,
}

impl Counter {
    #[inline]
    pub fn start() -> Self {
        Self { start_ticks: Self::count() }
    }

    #[inline]
    pub fn elapsed(&self) -> TicksElapsed {
        TicksElapsed(Self::count() - self.start_ticks)
    }

    #[inline]
    pub fn end(self) -> TicksElapsed {
        TicksElapsed(Self::count() - self.start_ticks)
    }

    #[inline]
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
//TODO: check precision of cast then divide vs divide then cast
#[allow(dead_code)]
impl TicksElapsed {
    #[inline]
    pub fn as_secs(self) -> f64 {
        self.0 as f64 / unsafe { PERFORMANCE_FREQUENCY } as f64
    }

    #[inline]
    pub fn as_ms(self) -> f64 {
        (self.0 * 1000) as f64 / unsafe { PERFORMANCE_FREQUENCY } as f64
    }

    #[inline]
    pub fn as_micros(self) -> f64 {
        (self.0 * 1_000_000) as f64 / unsafe { PERFORMANCE_FREQUENCY } as f64
    }

    #[inline]
    pub fn as_nanos(self) -> f64 {
        (self.0 * 1_000_000_000) as f64 / unsafe { PERFORMANCE_FREQUENCY } as f64
    }
}
