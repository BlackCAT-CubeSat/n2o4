// Copyright (c) 2023 The Pennsylvania State University and the project contributors.
// SPDX-License-Identifier: Apache-2.0

//! Types and methods for interacting with network sockets.

use cfs_sys::*;
use core::ffi::{c_char, c_void, CStr};
use core::marker::PhantomData;
use core::mem::ManuallyDrop;

use super::ObjectId;
use crate::sealed_traits::{SocketDomainSealed, SocketRoleSealed, SocketTypeSealed};
use crate::utils::CStrBuf;

/// Marker type for IPv4 addresses and sockets.
///
/// Corresponds to `OS_SocketDomain_INET`.
#[doc(alias = "OS_SocketDomain_INET")]
pub struct IPv4 {}

/// Marker type for IPv6 addresses and sockets.
///
/// Corresponds to `OS_SocketDomain_INET6`.
#[doc(alias = "OS_SocketDomain_INET6")]
pub struct IPv6 {}

/// A marker trait for network domains.
///
/// This is a [sealed trait](https://rust-lang.github.io/api-guidelines/future-proofing.html#c-sealed):
/// OSAL only supports a fixed set of socket domains.
///
/// Corresponds to `OS_SocketDomain_t`.
#[doc(alias = "OS_SocketDomain_t")]
pub trait SocketDomain: SocketDomainSealed {}

impl SocketDomainSealed for IPv4 {
    const DOMAIN: OS_SocketDomain_t = OS_SocketDomain_t_OS_SocketDomain_INET;
}
impl SocketDomainSealed for IPv6 {
    const DOMAIN: OS_SocketDomain_t = OS_SocketDomain_t_OS_SocketDomain_INET6;
}

impl SocketDomain for IPv4 {}
impl SocketDomain for IPv6 {}

/// Marker type for connectionless, message-oriented sockets.
///
/// For IPv4 and IPv6, this corresponds to UDP.
///
/// Corresponds to `OS_SocketType_DATAGRAM`.
#[doc(alias = "OS_SocketType_DATAGRAM")]
pub struct Datagram {}

/// Marker type for connection-oriented, stream-of-bytes sockets.
///
/// For IPv4 and IPv6, this corresponds to TCP.
///
/// Corresponds to `OS_SocketType_STREAM`.
#[doc(alias = "OS_SocketType_STREAM")]
pub struct Stream {}

/// A marker trait for socket types.
///
/// This is a [sealed trait](https://rust-lang.github.io/api-guidelines/future-proofing.html#c-sealed):
/// OSAL only supports a fixed set of socket types.
///
/// Corresponds to `OS_SocketType_t`.
#[doc(alias = "OS_SocketType_t")]
pub trait SocketType: SocketTypeSealed {}

impl SocketTypeSealed for Datagram {
    const SOCK_TYPE: OS_SocketType_t = OS_SocketType_t_OS_SocketType_DATAGRAM;
}
impl SocketTypeSealed for Stream {
    const SOCK_TYPE: OS_SocketType_t = OS_SocketType_t_OS_SocketType_STREAM;
}

impl SocketType for Datagram {}
impl SocketType for Stream {}

/// Marker type for sockets that operate in [a connection](EarlySocket::connect).
pub struct Connected {}

/// Marker type for sockets that have been [bound to a local port](EarlySocket::bind).
pub struct Bound {}

/// A marker trait for the role a socket plays.
///
/// This is a [sealed trait](https://rust-lang.github.io/api-guidelines/future-proofing.html#c-sealed):
/// there is a fixed set of socket roles.
pub trait SocketRole: SocketRoleSealed {}

impl SocketRoleSealed for Connected {}
impl SocketRoleSealed for Bound {}

impl SocketRole for Connected {}
impl SocketRole for Bound {}

/// A (network address, port) pair for use by socket-related APIs.
///
/// Wraps `OS_SockAddr_t`.
#[doc(alias = "OS_SockAddr_t")]
#[derive(Clone)]
pub struct SockAddr<T> {
    inner:   OS_SockAddr_t,
    phantom: PhantomData<T>,
}

impl<T: SocketDomain> SockAddr<T> {
    /// Tries to initialize a [`SockAddr`] with the given [domain](`SocketDomain`), address, and port.
    ///
    /// `address` is a network address in string form (e.g., `"192.0.2.5"`, `"2001:db8:0:2::5"`)
    ///
    /// Wraps `OS_SocketAddrInit`, `OS_SocketAddrFromString`, and `OS_SocketAddrSetPort`.
    #[doc(alias = "OS_SocketAddrInit")]
    #[inline]
    pub fn new<S: AsRef<CStr> + ?Sized>(network_address: &S, port: u16) -> Result<Self, i32> {
        let network_address = network_address.as_ref().as_ptr();
        let mut addr: OS_SockAddr_t = dummy_sock_addr();

        let status = unsafe { OS_SocketAddrInit(&mut addr, T::DOMAIN) };
        if status < 0 {
            return Err(status);
        }

        let status = unsafe { OS_SocketAddrFromString(&mut addr, network_address) };
        if status < 0 {
            return Err(status);
        }

        let status = unsafe { OS_SocketAddrSetPort(&mut addr, port) };

        if status >= 0 {
            Ok(Self {
                inner:   addr,
                phantom: PhantomData,
            })
        } else {
            Err(status)
        }
    }

    /// Tries to write the address's host address to `buf` as a C-style string.
    ///
    /// Wraps `OS_SocketAddrToString`.
    #[doc(alias = "OS_SocketAddrToString")]
    #[inline]
    pub fn get_host_addr(&self, buf: &mut [c_char]) -> Result<(), i32> {
        let status = unsafe { OS_SocketAddrToString(buf.as_mut_ptr(), buf.len(), &self.inner) };

        // Just in case OSAL doesn't do this on edge cases, null-terminate:
        if buf.len() > 0 {
            buf[buf.len() - 1] = b'\0' as c_char;
        }

        if status >= 0 {
            Ok(())
        } else {
            Err(status)
        }
    }

    /// Tries to set the address's host address from a C-style string (e.g., `"192.0.2.1"`, `"2001:db8::1"`).
    ///
    /// Wraps `OS_SocketAddrFromString`.
    #[doc(alias = "OS_SocketAddrFromString")]
    #[inline]
    pub fn set_host_addr<S: AsRef<CStr> + ?Sized>(&mut self, addr: &S) -> Result<(), i32> {
        let addr = addr.as_ref();

        let status = unsafe { OS_SocketAddrFromString(&mut self.inner, addr.as_ptr()) };

        if status >= 0 {
            Ok(())
        } else {
            Err(status)
        }
    }

    /// Returns the address's port number.
    ///
    /// Wraps `OS_SocketAddrGetPort`.
    #[doc(alias = "OS_SocketAddrGetPort")]
    #[inline]
    pub fn port(&self) -> Result<u16, i32> {
        let mut port_num: u16 = 0;

        let status = unsafe { OS_SocketAddrGetPort(&mut port_num, &self.inner) };

        if status >= 0 {
            Ok(port_num)
        } else {
            Err(status)
        }
    }

    /// Sets the address's port number.
    ///
    /// Wraps `OS_SocketAddrSetPort`.
    #[doc(alias = "OS_SocketAddrSetPort")]
    #[inline]
    pub fn set_port(&mut self, port_num: u16) -> Result<(), i32> {
        let status = unsafe { OS_SocketAddrSetPort(&mut self.inner, port_num) };

        if status >= 0 {
            Ok(())
        } else {
            Err(status)
        }
    }
}

/// A network socket that has been created, but has yet to be either
/// [connected to a peer](EarlySocket::connect) or [bound to a local port](EarlySocket::bind).
///
/// Wraps `osal_id_t`.
#[doc(alias = "osal_id_t")]
pub struct EarlySocket<D: SocketDomain, T: SocketType> {
    sock_id: osal_id_t,
    phantom: PhantomData<(D, T)>,
}

impl<D: SocketDomain, T: SocketType> EarlySocket<D, T> {
    /// Opens a socket with the given [domain](SocketDomain) and [type](SocketType).
    ///
    /// To do anything useful with the socket,
    /// [`connect`](EarlySocket::connect) or [`bind`](EarlySocket::bind)
    /// needs to be called.
    ///
    /// Wraps `OS_SocketOpen`.
    #[doc(alias = "OS_SocketOpen")]
    #[inline]
    pub fn open() -> Result<Self, i32> {
        let mut sock_id: osal_id_t = X_OS_OBJECT_ID_UNDEFINED;

        let status = unsafe { OS_SocketOpen(&mut sock_id, D::DOMAIN, T::SOCK_TYPE) };

        if status >= 0 && (ObjectId { id: sock_id }).is_defined() {
            Ok(EarlySocket { sock_id, phantom: PhantomData })
        } else {
            Err(status)
        }
    }

    /// Connects a socket to a peer at the remote address `addr`.
    ///
    /// Waits up to `timeout_ms.min(`[`i32::MAX`]`)` milliseconds for a successful connection,
    /// or indefinitely if `timeout_ms` is `None`.
    ///
    /// Wraps `OS_SocketConnect`.
    #[doc(alias = "OS_SocketConnect")]
    #[inline]
    pub fn connect(
        self,
        addr: &SockAddr<D>,
        timeout_ms: Option<u32>,
    ) -> Result<Socket<D, T, Connected>, i32> {
        let timeout: i32 = super::as_timeout(timeout_ms);

        let status = unsafe { OS_SocketConnect(self.sock_id, &addr.inner, timeout) };

        if status >= 0 {
            let sock = Socket {
                sock_id: self.sock_id,
                phantom: PhantomData,
            };
            let _ = ManuallyDrop::new(self);
            Ok(sock)
        } else {
            Err(status)
        }
    }

    /// Binds the socket to the local address `addr`.
    ///
    /// Wraps `OS_SocketBind`.
    #[doc(alias = "OS_SocketBind")]
    #[inline]
    pub fn bind(self, addr: &SockAddr<D>) -> Result<Socket<D, T, Bound>, i32> {
        let status = unsafe { OS_SocketBind(self.sock_id, &addr.inner) };

        if status >= 0 {
            let sock = Socket {
                sock_id: self.sock_id,
                phantom: PhantomData,
            };
            let _ = ManuallyDrop::new(self);
            Ok(sock)
        } else {
            Err(status)
        }
    }

    /// Returns the [`ObjectId`] for the socket.
    #[inline]
    pub fn as_id(&self) -> ObjectId {
        ObjectId { id: self.sock_id }
    }

    /// Unconditionally creates an [`EarlySocket`] from an OSAL ID.
    ///
    /// # Safety
    ///
    /// This function does **no** checking that the ID in question
    /// corresponds to a socket, much less one of the correct
    /// [domain](SocketDomain), [type](SocketType), or point in socket lifecycle.
    ///
    /// It is the programmer's responsibility to ensure that any OSAL ID passed
    /// to `from_id` corresponds to a socket
    /// with the correct socket domain, type, and state.
    #[inline]
    pub unsafe fn from_id(id: ObjectId) -> Self {
        Self {
            sock_id: id.id,
            phantom: PhantomData,
        }
    }

    /// If successful, returns information about the socket.
    ///
    /// Wraps `OS_SocketGetInfo`.
    #[doc(alias = "OS_SocketGetInfo")]
    #[inline]
    pub fn info(&self) -> Result<SocketProperties, i32> {
        let mut props = OS_socket_prop_t {
            name:    [0; OS_MAX_API_NAME as usize],
            creator: X_OS_OBJECT_ID_UNDEFINED,
        };

        let status = unsafe { OS_SocketGetInfo(self.sock_id, &mut props) };

        if status >= 0 {
            Ok(SocketProperties {
                name:    CStrBuf::new_into(props.name),
                creator: ObjectId { id: props.creator },
            })
        } else {
            Err(status)
        }
    }
}

/// Wraps `OS_close`.
impl<D: SocketDomain, T: SocketType> Drop for EarlySocket<D, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            let _ = OS_close(self.sock_id);
        }
    }
}

/// A network socket that is ready to send/receive data with a single peer ([`Connected`])
/// or accept new connections from/exchange datagrams with many peers ([`Bound`]).
///
/// A [`Socket`] is either (1) created from an [`EarlySocket`]
/// using [`connect`](EarlySocket::connect) or [`bind`](EarlySocket::bind)
/// or (2) returned from a successful call to [`accept`](Socket::accept).
///
/// Certain operations only make sense for a subset of the possible combinations
/// of [`SocketDomain`], [`SocketType`], and [`SocketRole`].
///
/// Wraps `osal_id_t`.
#[doc(alias = "osal_id_t")]
#[derive(Clone)]
pub struct Socket<D: SocketDomain, T: SocketType, R: SocketRole> {
    sock_id: osal_id_t,
    phantom: PhantomData<(D, T, R)>,
}

impl<D: SocketDomain, T: SocketType, R: SocketRole> Socket<D, T, R> {
    /// Returns the [`ObjectId`] for the socket.
    #[inline]
    pub fn as_id(&self) -> ObjectId {
        ObjectId { id: self.sock_id }
    }

    /// Unconditionally creates a [`Socket`] from an OSAL ID.
    ///
    /// # Safety
    ///
    /// This function does **no** checking that the ID in question
    /// corresponds to a socket, much less one of the correct
    /// [domain](SocketDomain), [type](SocketType), or point in socket lifecycle.
    ///
    /// It is the programmer's responsibility to ensure that any OSAL ID passed
    /// to `from_id` corresponds to a socket
    /// with the correct socket domain, type, and state.
    #[inline]
    pub unsafe fn from_id(id: ObjectId) -> Self {
        Self {
            sock_id: id.id,
            phantom: PhantomData,
        }
    }

    /// Closes the socket.
    ///
    /// Wraps `OS_close`.
    #[doc(alias = "OS_close")]
    #[inline]
    pub fn close(self) -> Result<(), i32> {
        let status = unsafe { OS_close(self.sock_id) };

        if status >= 0 {
            Ok(())
        } else {
            Err(status)
        }
    }

    /// Closes the socket.
    ///
    /// This variant is intended to be used in [`Drop`] `impl`s only;
    /// typically you want to use [`close`](Socket::close) instead.
    ///
    /// Wraps `OS_close`.
    ///
    /// # Safety
    ///
    /// This releases the underlying OSAL ID without necessarily
    /// destroying all references to the [`Socket`]. Any use
    /// of this [`Socket`] (or other [`Socket`] referring to
    /// the same underlying OSAL socket) after calling `close_mut` on it has
    /// potentially undesirable results&mdash;notably, there's
    /// the possibility of the OSAL ID being reused for a different
    /// socket, leading to unintended use of another OSAL socket.
    /// As such, callers must make sure this [`Socket`]
    /// (and anything else using the same OSAL ID)
    /// is never used after calling `close_mut`.
    #[doc(alias = "OS_close")]
    #[inline]
    pub unsafe fn close_mut(&mut self) -> Result<(), i32> {
        let status = unsafe { OS_close(self.sock_id) };

        if status >= 0 {
            Ok(())
        } else {
            Err(status)
        }
    }

    /// If successful, returns information about the socket.
    ///
    /// Wraps `OS_SocketGetInfo`.
    #[doc(alias = "OS_SocketGetInfo")]
    #[inline]
    pub fn info(&self) -> Result<SocketProperties, i32> {
        let mut props = OS_socket_prop_t {
            name:    [0; OS_MAX_API_NAME as usize],
            creator: X_OS_OBJECT_ID_UNDEFINED,
        };

        let status = unsafe { OS_SocketGetInfo(self.sock_id, &mut props) };

        if status >= 0 {
            Ok(SocketProperties {
                name:    CStrBuf::new_into(props.name),
                creator: ObjectId { id: props.creator },
            })
        } else {
            Err(status)
        }
    }
}

impl<D: SocketDomain, T: SocketType> Socket<D, T, Connected> {
    /// Reads up to `buf.len()` bytes from the connection into `buf`.
    ///
    /// Upon success, returns the number of bytes actually read into `buf`,
    /// or `0` if at the end of the stream.
    ///
    /// Wraps `OS_read`.
    #[doc(alias = "OS_read")]
    #[inline]
    pub fn read(&self, buf: &mut [u8]) -> Result<usize, i32> {
        let status = unsafe { OS_read(self.sock_id, buf.as_mut_ptr() as *mut c_void, buf.len()) };

        if status >= 0 {
            Ok(status as usize)
        } else {
            Err(status)
        }
    }

    /// Writes up to `buf.len()` bytes from `buf` to the connection.
    ///
    /// Upon success, returns the number of bytes actually written.
    ///
    /// Wraps `OS_write`.
    #[doc(alias = "OS_write")]
    #[inline]
    pub fn write(&self, buf: &[u8]) -> Result<usize, i32> {
        let status = unsafe { OS_write(self.sock_id, buf.as_ptr() as *const c_void, buf.len()) };

        if status >= 0 {
            Ok(status as usize)
        } else {
            Err(status)
        }
    }
}

impl<D: SocketDomain> Socket<D, Stream, Connected> {
    /// Gracefully shuts down one or both directions of a stream connection.
    ///
    /// Wraps `OS_SocketShutdown`.
    #[doc(alias = "OS_SocketShutdown")]
    #[inline]
    pub fn shutdown(&self, mode: SocketShutdownMode) -> Result<(), i32> {
        let status =
            unsafe { OS_SocketShutdown(self.sock_id, mode as u32 as OS_SocketShutdownMode_t) };

        if status >= 0 {
            Ok(())
        } else {
            Err(status)
        }
    }
}

impl<D: SocketDomain> Socket<D, Datagram, Connected> {
    /// Tries to change the socket's remote endpoint to `addr`,
    /// waiting up to `timeout_ms.min(`[`i32::MAX`]`)` to complete the operation.
    ///
    /// Wraps `OS_SocketConnect`.
    #[doc(alias = "OS_SocketConnect")]
    #[inline]
    pub fn connect(&self, addr: &SockAddr<D>, timeout_ms: Option<u32>) -> Result<(), i32> {
        let timeout = super::as_timeout(timeout_ms);

        let status = unsafe { OS_SocketConnect(self.sock_id, &addr.inner, timeout) };

        if status >= 0 {
            Ok(())
        } else {
            Err(status)
        }
    }
}

impl<D: SocketDomain> Socket<D, Stream, Bound> {
    /// Waits for and accepts the next incoming connection on the given listening socket.
    ///
    /// Waits for up to `timeout_ms.min(`[`i32::MAX`]`)` milliseconds for a new connection
    /// (or indefinitely if `timeout_ms` is `None`).
    ///
    /// On success, results a socket for the new connection
    /// and the address of the connection's remote side.
    ///
    /// Wraps `OS_SocketAccept`.
    #[doc(alias = "OS_SocketAccept")]
    #[inline]
    pub fn accept(
        &self,
        timeout_ms: Option<u32>,
    ) -> Result<(Socket<D, Stream, Connected>, SockAddr<D>), i32> {
        let mut connsock_id: osal_id_t = X_OS_OBJECT_ID_UNDEFINED;
        let mut conn_addr = dummy_sock_addr();
        let timeout = super::as_timeout(timeout_ms);

        let status =
            unsafe { OS_SocketAccept(self.sock_id, &mut connsock_id, &mut conn_addr, timeout) };

        if status >= 0 && (ObjectId { id: connsock_id }).is_defined() {
            Ok((
                Socket {
                    sock_id: connsock_id,
                    phantom: PhantomData,
                },
                SockAddr {
                    inner:   conn_addr,
                    phantom: PhantomData,
                },
            ))
        } else {
            Err(status)
        }
    }
}

impl<D: SocketDomain, R: SocketRole> Socket<D, Datagram, R> {
    /// Sends a message from the datagram socket to `remote_addr`,
    /// using `buf` as the message contents.
    ///
    /// On success, returns the number of bytes of `buf` that were actually sent.
    ///
    /// Wraps `OS_SocketSendTo`.
    #[doc(alias = "OS_SocketSendTo")]
    #[inline]
    pub fn send(&self, buf: &[u8], remote_addr: &SockAddr<D>) -> Result<usize, i32> {
        let status = unsafe {
            OS_SocketSendTo(
                self.sock_id,
                buf.as_ptr() as *const c_void,
                buf.len(),
                &remote_addr.inner,
            )
        };

        if status >= 0 {
            Ok(status as usize)
        } else {
            Err(status)
        }
    }
}

impl<D: SocketDomain> Socket<D, Datagram, Bound> {
    /// Reads a message from the bound datagram socket into `buf`.
    ///
    /// Wait up to `timeout_ms.min(`[`i32::MAX`]`)` milliseconds for a message
    /// (or indefinitely if `timeout_ms` is `None`).
    ///
    /// On success, returns the number of bytes written to `buf`
    /// and the address of the message sender.
    ///
    /// Wraps `OS_SocketRecvFrom`.
    #[doc(alias = "OS_SocketRecvFrom")]
    #[inline]
    pub fn recv(
        &self,
        buf: &mut [u8],
        timeout_ms: Option<u32>,
    ) -> Result<(usize, SockAddr<D>), i32> {
        let mut remote_addr = dummy_sock_addr();
        let timeout = super::as_timeout(timeout_ms);

        let status = unsafe {
            OS_SocketRecvFrom(
                self.sock_id,
                buf.as_mut_ptr() as *mut c_void,
                buf.len(),
                &mut remote_addr,
                timeout,
            )
        };

        if status >= 0 {
            Ok((
                status as usize,
                SockAddr {
                    inner:   remote_addr,
                    phantom: PhantomData,
                },
            ))
        } else {
            Err(status)
        }
    }
}

impl<D: SocketDomain, T: SocketType, R: SocketRole> PartialEq<Self> for Socket<D, T, R> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_id() == other.as_id()
    }
}

/// The possible [shutdown modes](`Socket::shutdown`) for a stream connection.
///
/// Corresponds to `OS_SocketShutdownMode_t`.
#[doc(alias = "OS_SocketShutdownMode_t")]
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SocketShutdownMode {
    /// Shut down the read direction of the session.
    #[doc(alias = "OS_SocketShutdownMode_SHUT_READ")]
    Read      = OS_SocketShutdownMode_t_OS_SocketShutdownMode_SHUT_READ as _,

    /// Shut down the write direction of the session.
    #[doc(alias = "OS_SocketShutdownMode_SHUT_WRITE")]
    Write     = OS_SocketShutdownMode_t_OS_SocketShutdownMode_SHUT_WRITE as _,

    /// Shut down both directions of the session.
    #[doc(alias = "OS_SocketShutdownMode_SHUT_READWRITE")]
    ReadWrite = OS_SocketShutdownMode_t_OS_SocketShutdownMode_SHUT_READWRITE as _,
}

/// Information about a [`Socket`] or [`EarlySocket`].
///
/// Corresponds to `OS_socket_prop_t`.
#[doc(alias = "OS_socket_prop_t")]
#[derive(Clone, PartialEq, Eq)]
pub struct SocketProperties {
    /// The socket's name.
    pub name: CStrBuf<{ OS_MAX_API_NAME as usize }>,

    /// The task which opened the socket.
    pub creator: ObjectId,
}

/// Returns a new `OS_SockAddr_t` so that we can initialize some variables.
#[inline]
fn dummy_sock_addr() -> OS_SockAddr_t {
    OS_SockAddr_t {
        ActualLength: 0,
        AddrData:     OS_SockAddrData_t { AlignU32: 0 },
    }
}
