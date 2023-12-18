/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::ffi::{c_int, c_long};

use crate::ctypes;

/// Return sysinfo struct
#[no_mangle]
pub unsafe extern "C" fn sys_sysinfo(info: *mut ctypes::sysinfo) -> c_int {
    debug!("sys_sysinfo");
    syscall_body!(sys_sysinfo, {
        let info_mut = info.as_mut().unwrap();

        // If the kernel booted less than 1 second, it will be 0.
        info_mut.uptime = ruxhal::time::current_time().as_secs() as c_long;

        info_mut.loads = [0; 3];
        #[cfg(feature = "multitask")]
        {
            ruxtask::get_avenrun(&mut info_mut.loads);
        }

        info_mut.sharedram = 0;
        // TODO
        info_mut.bufferram = 0;

        info_mut.totalram = 0;
        info_mut.freeram = 0;
        #[cfg(feature = "alloc")]
        {
            use core::ffi::c_ulong;
            let allocator = axalloc::global_allocator();
            info_mut.freeram = (allocator.available_bytes()
                + allocator.available_pages() * memory_addr::PAGE_SIZE_4K)
                as c_ulong;
            info_mut.totalram = info_mut.freeram + allocator.used_bytes() as c_ulong;
        }

        // TODO
        info_mut.totalswap = 0;
        info_mut.freeswap = 0;

        info_mut.procs = 1;

        // unused in 64-bit
        info_mut.totalhigh = 0;
        info_mut.freehigh = 0;

        info_mut.mem_unit = 1;

        Ok(0)
    })
}

/// Print system information
pub fn sys_uname(_uts: *mut core::ffi::c_void) -> c_int {
    debug!("sys_uname not implemented");
    syscall_body!(sys_uname, Ok(0))
}
