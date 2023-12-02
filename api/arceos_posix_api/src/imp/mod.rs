/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

mod stdio;

pub mod io;
pub mod resources;
pub mod sys;
pub mod task;
pub mod time;

#[cfg(feature = "fd")]
pub mod fd_ops;
#[cfg(feature = "fs")]
pub mod fs;
#[cfg(any(feature = "select", feature = "poll", feature = "epoll"))]
pub mod io_mpx;
#[cfg(feature = "net")]
pub mod net;
#[cfg(feature = "pipe")]
pub mod pipe;
#[cfg(feature = "multitask")]
pub mod pthread;
