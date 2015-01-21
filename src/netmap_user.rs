use libc::{c_int, c_uint, c_char, c_uchar, c_void, size_t, timeval};

use netmap::*;

extern {
    fn memcpy(dest: *mut c_void, src: *mut c_void, n: size_t) -> *mut c_void;
}

// FIXME Replace with intrinsics
#[inline(always)]
fn likely<T>(t: T) -> T {
    t
}
#[inline(always)]
fn unlikely<T>(t: T) -> T {
    t
}

#[inline(always)]
pub unsafe fn _NETMAP_OFFSET<T, U>(ptr: *mut U, offset: isize) -> *mut T {
    (((ptr as *mut c_char).offset(offset)) as *mut c_void) as *mut T
}

#[inline(always)]
pub unsafe fn NETMAP_IF<U>(_base: *mut U, _ofs: isize) -> *mut netmap_if {
    _NETMAP_OFFSET(_base, _ofs)
}

// FIXME It's possible the pointer arithmetic here uses the wrong integer types.
#[inline(always)]
pub unsafe fn NETMAP_TXRING(nifp: *mut netmap_if, index: isize) -> *mut netmap_ring {
    let ptr = (&mut (*nifp).ring_ofs as *mut [i64; 0]) as *mut c_void;
    _NETMAP_OFFSET(nifp, *(ptr.offset(index) as *mut isize))
}

#[inline(always)]
pub unsafe fn NETMAP_RXRING(nifp: *mut netmap_if, index: isize) -> *mut netmap_ring {
    let ptr = (&mut (*nifp).ring_ofs as *mut [i64; 0]) as *mut c_void;
    _NETMAP_OFFSET(nifp, *(ptr.offset(index + (*nifp).ni_tx_rings as isize + 1) as *mut isize))
}

#[inline(always)]
pub unsafe fn NETMAP_BUF(ring: *mut netmap_ring, index: isize) -> *mut c_char {
    (ring as *mut c_char).offset((*ring).buf_ofs as isize + (index as isize * (*ring).nr_buf_size as isize))
}

#[inline(always)]
pub unsafe fn NETMAP_BUF_IDX(ring: *mut netmap_ring, buf: *mut c_char) -> usize {
    ((buf as *mut c_char).offset( -((ring as *mut c_char) as isize) )
                         .offset((*ring).buf_ofs as isize) as usize / (*ring).nr_buf_size as usize)
}

#[inline(always)]
pub unsafe fn nm_ring_next(r: *mut netmap_ring, i: u32) -> u32 {
    if unlikely(i + 1 == (*r).num_slots) {
        0
    } else {
        i + 1
    }
}

#[inline(always)]
pub unsafe fn nm_ring_space(ring: *mut netmap_ring) -> u32 {
    let mut ret: c_int = ((*ring).tail - (*ring).cur) as c_int;
    if ret < 0 {
        ret += (*ring).num_slots as c_int;
    }
    return ret as u32;
}

#[repr(C)]
#[derive(Copy)]
pub struct nm_pkthdr {
    pub ts: timeval,
    pub caplen: u32,
    pub len: u32,
}

#[repr(C)]
#[derive(Copy)]
pub struct nm_stat {
    pub ps_recv: c_uint,
    pub ps_drop: c_uint,
    pub ps_ifdrop: c_uint,
}

pub const NM_ERRBUF_SIZE: usize = 512;

#[repr(C)]
#[derive(Copy)]
pub struct nm_desc {
    pub self_: *mut nm_desc,
    pub fd: c_int,
    pub mem: *mut c_void,
    pub memsize: u32,
    pub done_mmap: c_int,
    pub nifp: *const netmap_if,
    pub first_tx_ring: u16,
    pub last_tx_ring: u16,
    pub cur_tx_ring: u16,
    pub first_rx_ring: u16,
    pub last_rx_ring: u16,
    pub cur_rx_ring: u16,
    pub req: nmreq,
    pub hdr: nm_pkthdr,

    pub some_ring: *const netmap_ring,
    pub buf_start: *const c_void,
    pub buf_end: *const c_void,
    pub snaplen: c_int,
    pub promisc: c_int,
    pub to_ms: c_int,
    pub errbuf: *mut c_char,

    pub if_flags: u32,
    pub if_reqcap: u32,
    pub if_curcap: u32,

    pub st: nm_stat,
    pub msg: [c_char; NM_ERRBUF_SIZE],
}

#[inline(always)]
pub unsafe fn P2NMD<T>(p: *mut T) -> *mut nm_desc {
    p as *mut nm_desc
}

#[inline(always)]
pub unsafe fn IS_NETMAP_DESC(d: *mut nm_desc) -> bool {
    !d.is_null() && (*P2NMD(d)).self_ == P2NMD(d)
}

#[inline(always)]
pub unsafe fn NETMAP_FD(d: *mut nm_desc) -> c_int {
    (*P2NMD(d)).fd
}

#[inline(always)]
pub unsafe fn nm_pkt_copy(_src: *const c_void, _dst: *mut c_void, mut l: c_int) {
    let mut src = _src as *const u64;
    let mut dst = _dst as *mut u64;

    if unlikely(l > 1024) {
        memcpy(dst as *mut c_void, src as *mut c_void, l as u64);
        return;
    }

    while likely(l > 0) {
        *dst = *src; dst = dst.offset(1); src = src.offset(1);
        *dst = *src; dst = dst.offset(1); src = src.offset(1);
        *dst = *src; dst = dst.offset(1); src = src.offset(1);
        *dst = *src; dst = dst.offset(1); src = src.offset(1);
        *dst = *src; dst = dst.offset(1); src = src.offset(1);
        *dst = *src; dst = dst.offset(1); src = src.offset(1);
        *dst = *src; dst = dst.offset(1); src = src.offset(1);
        *dst = *src; dst = dst.offset(1); src = src.offset(1);
        l -= 64;
    }
}

pub type nm_cb_t = extern fn(*mut c_uchar, *const nm_pkthdr, *const c_uchar) -> c_void;

pub const NM_OPEN_NO_MMAP: c_int = 0x040000;
pub const NM_OPEN_IFNAME: c_int = 0x080000;
pub const NM_OPEN_ARG1: c_int = 0x100000;
pub const NM_OPEN_ARG2: c_int = 0x200000;
pub const NM_OPEN_ARG3: c_int = 0x400000;
pub const NM_OPEN_RING_CFG: c_int = 0x800000;

extern {
    pub fn nm_open(ifname: *const c_char, req: *const nmreq,
                   new_flags: u64, arg: *const nm_desc) -> *mut nm_desc;
    pub fn nm_close(d: *mut nm_desc) -> c_int;
    pub fn nm_inject(d: *mut nm_desc, buf: *const c_void, size: size_t) -> c_int;
    pub fn nm_dispatch(d: *mut nm_desc, cnt: c_int, cb: nm_cb_t, arg: *mut c_uchar) -> c_int;
    pub fn nm_nextpkt(d: *mut nm_desc, hdr: *mut nm_pkthdr) -> *mut c_uchar;
}
