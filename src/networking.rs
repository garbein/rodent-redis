use async_std::io;
use async_std::net::TcpStream;

unsafe fn setsockopt<T>(
    fd: libc::c_int,
    opt: libc::c_int,
    val: libc::c_int,
    payload: T,
) -> io::Result<()>
where
    T: Copy,
{
    let payload = &payload as *const T as *const libc::c_void;
    let res = libc::setsockopt(
        fd,
        opt,
        val,
        payload,
        std::mem::size_of::<T>() as libc::socklen_t,
    );
    if res == -1 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

pub unsafe fn set_keepalive(stream: &TcpStream, sec: i32) -> io::Result<()> {
    use std::os::unix::io::AsRawFd;
    let fd = stream.as_raw_fd();

    self::setsockopt(fd, libc::SOL_SOCKET, libc::SO_KEEPALIVE, 1)?;

    #[cfg(target_os = "linux")]
    {
        let mut val = sec;
        self::setsockopt(fd, libc::IPPROTO_TCP, libc::TCP_KEEPIDLE, val)?;

        val = sec / 3;
        self::setsockopt(fd, libc::IPPROTO_TCP, libc::TCP_KEEPINTVL, val)?;

        val = 3;
        self::setsockopt(fd, libc::IPPROTO_TCP, libc::TCP_KEEPCNT, val)?;
    }

    Ok(())
}
