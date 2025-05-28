#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub struct Kcp {
    ikcpcb: *mut ikcpcb,
}

impl Drop for Kcp {
    fn drop(&mut self) {
        unsafe { ikcp_release(self.ikcpcb) };
    }
}

impl Kcp {
    pub fn new(conv: u32) -> Box<Kcp> {
        let mut kcp = Box::new(Kcp {
            ikcpcb: std::ptr::null_mut(),
        });
        kcp.ikcpcb = unsafe { ikcp_create(conv, kcp.as_mut() as *mut _ as _) };
        unsafe { kcp.ikcpcb.as_mut().unwrap() }.output = Some(udp_output);
        return kcp;
    }

    pub fn update(&mut self, current: u32) {
        unsafe { ikcp_update(self.ikcpcb, current) };
    }

    pub fn input(&mut self, data: &[core::ffi::c_char]) {
        #[cfg(debug_assertions)]
        if data.len() > core::ffi::c_long::MAX as usize {
            panic!("Too large");
        }
        unsafe { ikcp_input(self.ikcpcb, data.as_ptr(), data.len() as core::ffi::c_long) };
    }

    pub fn set_wndsize(&mut self, sndwnd: i32, rcvwnd: i32) {
        unsafe { ikcp_wndsize(self.ikcpcb, sndwnd, rcvwnd) };
    }

    pub fn rec(&mut self, buffer: &mut [core::ffi::c_char]) {
        #[cfg(debug_assertions)]
        if buffer.len() > core::ffi::c_int::MAX as usize {
            panic!("Too large");
        }
        unsafe {
            ikcp_recv(
                self.ikcpcb,
                buffer.as_mut_ptr(),
                buffer.len() as core::ffi::c_int,
            )
        };
    }

    pub fn send(&mut self, buffer: &[core::ffi::c_char]) {
        #[cfg(debug_assertions)]
        if buffer.len() > core::ffi::c_int::MAX as usize {
            panic!("Too large");
        }
        unsafe {
            ikcp_send(
                self.ikcpcb,
                buffer.as_ptr(),
                buffer.len() as core::ffi::c_int,
            )
        };
    }

    pub fn peeksize(&self) -> i32 {
        unsafe { ikcp_peeksize(self.ikcpcb) }
    }
}

unsafe extern "C" fn udp_output(
    buf: *const ::std::os::raw::c_char,
    len: ::std::os::raw::c_int,
    ll_kcp: *mut IKCPCB,
    user: *mut ::std::os::raw::c_void,
) -> ::std::os::raw::c_int {
    let _ = ll_kcp;
    let _ = len;
    let _ = buf;
    let kcp: *mut Kcp = unsafe { std::mem::transmute(user) };
    let _kcp = unsafe { kcp.as_mut().unwrap() };
    return 0;
}

#[cfg(test)]
mod test {
    use super::Kcp;

    #[test]
    fn test_case() {
        let _kcp = Kcp::new(0);
    }
}
