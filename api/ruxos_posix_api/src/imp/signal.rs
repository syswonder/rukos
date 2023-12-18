/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::ffi::c_int;
use core::time::Duration;

use crate::ctypes;
use crate::ctypes::k_sigaction;

use axerrno::LinuxError;
use ruxruntime::{rx_sigaction, Signal};

/// Set signal handler
pub fn sys_sigaction(
    signum: u8,
    sigaction: Option<&k_sigaction>,
    oldact: Option<&mut k_sigaction>,
) {
    Signal::sigaction(
        signum,
        sigaction.map(|act| act as *const k_sigaction as *const rx_sigaction),
        oldact.map(|old| old as *mut k_sigaction as *mut rx_sigaction),
    );
}

/// Set a timer to send a signal to the current process after a specified time
pub unsafe fn sys_setitimer(which: c_int, new: *const ctypes::itimerval) -> c_int {
    syscall_body!(sys_setitimer, {
        let which = which as usize;
        let new_interval = Duration::from((*new).it_interval).as_nanos() as u64;
        Signal::timer_interval(which, Some(new_interval));

        let new_ddl =
            ruxhal::time::current_time_nanos() + Duration::from((*new).it_value).as_nanos() as u64;
        Signal::timer_deadline(which, Some(new_ddl));
        Ok(0)
    })
}

/// Get timer to send signal after some time
pub unsafe fn sys_getitimer(which: c_int, curr_value: *mut ctypes::itimerval) -> c_int {
    syscall_body!(sys_getitimer, {
        let ddl = Duration::from_nanos(Signal::timer_deadline(which as usize, None).unwrap());
        if ddl.as_nanos() == 0 {
            return Err(LinuxError::EINVAL);
        }
        let mut now: ctypes::timespec = ctypes::timespec::default();
        unsafe {
            crate::sys_clock_gettime(0, &mut now);
        }
        let now = Duration::from(now);
        if ddl > now {
            (*curr_value).it_value = ctypes::timeval::from(ddl - now);
        } else {
            (*curr_value).it_value = ctypes::timeval::from(Duration::new(0, 0));
        }
        (*curr_value).it_interval =
            Duration::from_nanos(Signal::timer_interval(which as usize, None).unwrap()).into();
        Ok(0)
    })
}
