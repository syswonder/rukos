/* Copyright (c) [2023] [Syswonder Community]
*   [Ruxos] is licensed under Mulan PSL v2.
*   You can use this software according to the terms and conditions of the Mulan PSL v2.
*   You may obtain a copy of Mulan PSL v2 at:
*               http://license.coscl.org.cn/MulanPSL2
*   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
*   See the Mulan PSL v2 for more details.
*/

use core::cell::UnsafeCell;
use core::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use core::sync::atomic::{AtomicBool, Ordering};

use alloc::string::ToString;
use axerrno::{ax_err, ax_err_type, AxError, AxResult};
use axio::PollState;
use axsync::Mutex;
use spin::RwLock;

use smoltcp::iface::SocketHandle;
use smoltcp::socket::udp::{self, BindError, SendError};
use smoltcp::wire::{IpEndpoint, IpListenEndpoint};

use super::addr::{from_core_sockaddr, into_core_sockaddr, is_unspecified, UNSPECIFIED_ENDPOINT};
use super::{route_dev, to_static_str, SocketSetWrapper, SOCKET_SET};

/// A UDP socket that provides POSIX-like APIs.
pub struct UdpSocket {
    handle: UnsafeCell<Option<(SocketHandle, &'static str)>>,
    local_addr: RwLock<Option<IpEndpoint>>,
    peer_addr: RwLock<Option<IpEndpoint>>,
    nonblock: AtomicBool,
}

impl UdpSocket {
    /// Creates a new UDP socket.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            handle: UnsafeCell::new(None),
            local_addr: RwLock::new(None),
            peer_addr: RwLock::new(None),
            nonblock: AtomicBool::new(false),
        }
    }

    /// Returns the local address and port, or
    /// [`Err(NotConnected)`](AxError::NotConnected) if not connected.
    pub fn local_addr(&self) -> AxResult<SocketAddr> {
        match self.local_addr.try_read() {
            Some(addr) => addr.map(into_core_sockaddr).ok_or(AxError::NotConnected),
            None => Err(AxError::NotConnected),
        }
    }

    /// Returns the remote address and port, or
    /// [`Err(NotConnected)`](AxError::NotConnected) if not connected.
    pub fn peer_addr(&self) -> AxResult<SocketAddr> {
        self.remote_endpoint().map(into_core_sockaddr)
    }

    /// Returns whether this socket is in nonblocking mode.
    #[inline]
    pub fn is_nonblocking(&self) -> bool {
        self.nonblock.load(Ordering::Acquire)
    }

    /// Moves this UDP socket into or out of nonblocking mode.
    ///
    /// This will result in `recv`, `recv_from`, `send`, and `send_to`
    /// operations becoming nonblocking, i.e., immediately returning from their
    /// calls. If the IO operation is successful, `Ok` is returned and no
    /// further action is required. If the IO operation could not be completed
    /// and needs to be retried, an error with kind
    /// [`Err(WouldBlock)`](AxError::WouldBlock) is returned.
    #[inline]
    pub fn set_nonblocking(&self, nonblocking: bool) {
        self.nonblock.store(nonblocking, Ordering::Release);
    }

    /// Binds an unbound socket to the given address and port.
    ///
    /// It's must be called before [`send_to`](Self::send_to) and
    /// [`recv_from`](Self::recv_from).
    pub fn bind(&self, mut local_addr: SocketAddr) -> AxResult {
        let mut self_local_addr = self.local_addr.write();

        if local_addr.port() == 0 {
            local_addr.set_port(get_ephemeral_port()?);
        }
        if self_local_addr.is_some() {
            return ax_err!(InvalidInput, "socket bind() failed: already bound");
        }

        let local_endpoint = from_core_sockaddr(local_addr);
        let endpoint = IpListenEndpoint {
            addr: (!is_unspecified(local_endpoint.addr)).then_some(local_endpoint.addr),
            port: local_endpoint.port,
        };
        let iface_name = match local_addr {
            SocketAddr::V4(addr) => route_dev(addr.ip().octets()),
            _ => panic!("IPv6 not supported"),
        };
        let handle = unsafe { self.handle.get().read() }.unwrap_or_else(|| {
            (
                SOCKET_SET.add(SocketSetWrapper::new_tcp_socket(), iface_name.clone()),
                to_static_str(iface_name),
            )
        });
        SOCKET_SET.with_socket_mut::<udp::Socket, _, _>(
            handle.0,
            handle.1.to_string(),
            |socket| {
                socket.bind(endpoint).or_else(|e| match e {
                    BindError::InvalidState => ax_err!(AlreadyExists, "socket bind() failed"),
                    BindError::Unaddressable => ax_err!(InvalidInput, "socket bind() failed"),
                })
            },
        )?;

        *self_local_addr = Some(local_endpoint);
        debug!("UDP socket {}: bound on {}", handle.0, endpoint);
        Ok(())
    }

    /// Sends data on the socket to the given address. On success, returns the
    /// number of bytes written.
    pub fn send_to(&self, buf: &[u8], remote_addr: SocketAddr) -> AxResult<usize> {
        if remote_addr.port() == 0 || remote_addr.ip().is_unspecified() {
            return ax_err!(InvalidInput, "socket send_to() failed: invalid address");
        }
        self.send_impl(buf, from_core_sockaddr(remote_addr))
    }

    /// Receives a single datagram message on the socket. On success, returns
    /// the number of bytes read and the origin.
    pub fn recv_from(&self, buf: &mut [u8]) -> AxResult<(usize, SocketAddr)> {
        self.recv_impl(|socket| match socket.recv_slice(buf) {
            Ok((len, meta)) => Ok((len, into_core_sockaddr(meta.endpoint))),
            Err(_) => ax_err!(BadState, "socket recv_from() failed"),
        })
    }

    /// Receives a single datagram message on the socket, without removing it from
    /// the queue. On success, returns the number of bytes read and the origin.
    pub fn peek_from(&self, buf: &mut [u8]) -> AxResult<(usize, SocketAddr)> {
        self.recv_impl(|socket| match socket.peek_slice(buf) {
            Ok((len, meta)) => Ok((len, into_core_sockaddr(meta.endpoint))),
            Err(_) => ax_err!(BadState, "socket recv_from() failed"),
        })
    }

    /// Connects this UDP socket to a remote address, allowing the `send` and
    /// `recv` to be used to send data and also applies filters to only receive
    /// data from the specified address.
    ///
    /// The local port will be generated automatically if the socket is not bound.
    /// It's must be called before [`send`](Self::send) and
    /// [`recv`](Self::recv).
    pub fn connect(&self, addr: SocketAddr) -> AxResult {
        let mut self_peer_addr = self.peer_addr.write();

        if self.local_addr.read().is_none() {
            self.bind(into_core_sockaddr(UNSPECIFIED_ENDPOINT))?;
        }

        *self_peer_addr = Some(from_core_sockaddr(addr));
        unsafe {
            debug!(
                "UDP socket {}: connected to {}",
                self.handle.get().read().unwrap().0,
                addr
            );
        }
        Ok(())
    }

    /// Sends data on the socket to the remote address to which it is connected.
    pub fn send(&self, buf: &[u8]) -> AxResult<usize> {
        let remote_endpoint = self.remote_endpoint()?;
        self.send_impl(buf, remote_endpoint)
    }

    /// Receives a single datagram message on the socket from the remote address
    /// to which it is connected. On success, returns the number of bytes read.
    pub fn recv(&self, buf: &mut [u8]) -> AxResult<usize> {
        let remote_endpoint = self.remote_endpoint()?;
        self.recv_impl(|socket| {
            let (len, meta) = socket
                .recv_slice(buf)
                .map_err(|_| ax_err_type!(BadState, "socket recv() failed"))?;
            if !is_unspecified(remote_endpoint.addr) && remote_endpoint.addr != meta.endpoint.addr {
                return Err(AxError::WouldBlock);
            }
            if remote_endpoint.port != 0 && remote_endpoint.port != meta.endpoint.port {
                return Err(AxError::WouldBlock);
            }
            Ok(len)
        })
    }

    /// Close the socket.
    pub fn shutdown(&self) -> AxResult {
        let handle = unsafe { self.handle.get().read().unwrap() };
        SOCKET_SET.with_socket_mut::<udp::Socket, _, _>(handle.0, handle.1.to_string(), |socket| {
            debug!("UDP socket {}: shutting down", handle.0);
            socket.close();
        });
        SOCKET_SET.poll_interfaces();
        Ok(())
    }

    /// Whether the socket is readable or writable.
    pub fn poll(&self) -> AxResult<PollState> {
        if self.local_addr.read().is_none() {
            return Ok(PollState {
                readable: false,
                writable: false,
            });
        }
        let handle = unsafe { self.handle.get().read().unwrap() };
        SOCKET_SET.with_socket_mut::<udp::Socket, _, _>(handle.0, handle.1.to_string(), |socket| {
            Ok(PollState {
                readable: socket.can_recv(),
                writable: socket.can_send(),
            })
        })
    }
}

/// Private methods
impl UdpSocket {
    fn remote_endpoint(&self) -> AxResult<IpEndpoint> {
        match self.peer_addr.try_read() {
            Some(addr) => addr.ok_or(AxError::NotConnected),
            None => Err(AxError::NotConnected),
        }
    }

    fn send_impl(&self, buf: &[u8], remote_endpoint: IpEndpoint) -> AxResult<usize> {
        if self.local_addr.read().is_none() {
            let res = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0));
            self.bind(res)?;
        }

        self.block_on(|| {
            let handle = unsafe { self.handle.get().read().unwrap() };
            SOCKET_SET.with_socket_mut::<udp::Socket, _, _>(
                handle.0,
                handle.1.to_string(),
                |socket| {
                    if socket.can_send() {
                        socket
                            .send_slice(buf, remote_endpoint)
                            .map_err(|e| match e {
                                SendError::BufferFull => AxError::WouldBlock,
                                SendError::Unaddressable => {
                                    ax_err_type!(ConnectionRefused, "socket send() failed")
                                }
                            })?;
                        Ok(buf.len())
                    } else {
                        // tx buffer is full
                        Err(AxError::WouldBlock)
                    }
                },
            )
        })
    }

    fn recv_impl<F, T>(&self, mut op: F) -> AxResult<T>
    where
        F: FnMut(&mut udp::Socket) -> AxResult<T>,
    {
        if self.local_addr.read().is_none() {
            return ax_err!(NotConnected, "socket send() failed");
        }

        self.block_on(|| {
            let handle = unsafe { self.handle.get().read().unwrap() };
            SOCKET_SET.with_socket_mut::<udp::Socket, _, _>(
                handle.0,
                handle.1.to_string(),
                |socket| {
                    if socket.can_recv() {
                        // data available
                        op(socket)
                    } else {
                        // no more data
                        Err(AxError::WouldBlock)
                    }
                },
            )
        })
    }

    fn block_on<F, T>(&self, mut f: F) -> AxResult<T>
    where
        F: FnMut() -> AxResult<T>,
    {
        if self.is_nonblocking() {
            f()
        } else {
            loop {
                SOCKET_SET.poll_interfaces();
                match f() {
                    Ok(t) => return Ok(t),
                    Err(AxError::WouldBlock) => ruxtask::yield_now(),
                    Err(e) => return Err(e),
                }
            }
        }
    }
}

impl Drop for UdpSocket {
    fn drop(&mut self) {
        self.shutdown().ok();
        let handle = unsafe { self.handle.get().read().unwrap() };
        SOCKET_SET.remove(handle.0, handle.1.to_string());
    }
}

fn get_ephemeral_port() -> AxResult<u16> {
    const PORT_START: u16 = 0x15b3;
    const PORT_END: u16 = 0xffff;
    static CURR: Mutex<u16> = Mutex::new(PORT_START);
    let mut curr = CURR.lock();

    let port = *curr;
    if *curr == PORT_END {
        *curr = PORT_START;
    } else {
        *curr += 1;
    }
    Ok(port)
}
