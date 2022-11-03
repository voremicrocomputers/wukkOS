use ieee1275::PROM;
use spin::Mutex;
use lazy_static::lazy_static;

pub struct PromHandle {
    pub prom: Option<PROM>,
}

unsafe impl Send for PromHandle {}
unsafe impl Sync for PromHandle {}

lazy_static!{
    pub static ref PROMHNDL: Mutex<PromHandle> = Mutex::new(PromHandle {
        prom: None,
    });
}

impl PromHandle {
    pub fn set_prom(&mut self, prom: PROM) {
        self.prom = Some(prom);
    }
    pub fn get(&mut self) -> &mut PROM {
        self.prom.as_mut().unwrap()
    }
}