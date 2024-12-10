/* Copyright (c) [2024] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */
 
/// ivc "hvc" method call on aarch64 hvisor
#[cfg(feature = "ivc")]
pub fn ivc_hvc_call(func: u32, arg0: usize, arg1: usize) -> usize {
    let ret;
    unsafe {
        core::arch::asm!(
            "hvc #0",
            inlateout("x0") func as usize => ret,
            in("x1") arg0,
            in("x2") arg1,
            options(nostack)
        );
        info!("Ivc call: func: {:x}, arg0: 0x{:x}, arg1: 0x{:x}, ret: 0x{:x}", func, arg0, arg1, ret);
    }
    ret
}