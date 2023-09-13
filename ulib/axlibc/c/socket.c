/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#ifdef AX_CONFIG_NET

#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <string.h>

int accept4(int fd, struct sockaddr *restrict addr, socklen_t *restrict len, int flg)
{
    if (!flg)
        return accept(fd, addr, len);
    if (flg & ~(SOCK_CLOEXEC | SOCK_NONBLOCK)) {
        errno = EINVAL;
        return -1;
    }
    int ret = accept(fd, addr, len);
    if (ret < 0)
        return ret;
    if (flg & SOCK_CLOEXEC)
        fcntl(ret, F_SETFD, FD_CLOEXEC);
    if (flg & SOCK_NONBLOCK)
        fcntl(ret, F_SETFL, O_NONBLOCK);
    return ret;
}

int getsockopt(int fd, int level, int optname, void *restrict optval, socklen_t *restrict optlen)
{
    unimplemented();
    return -1;
}

int setsockopt(int fd, int level, int optname, const void *optval, socklen_t optlen)
{
    unimplemented("fd: %d, level: %d, optname: %d, optval: %d, optlen: %d", fd, level, optname,
                  *(int *)optval, optlen);
    return 0;
}

// TODO: remove this function in future work
ssize_t ax_sendmsg(int fd, const struct msghdr *msg, int flags);

#define __CMSG_LEN(cmsg) (((cmsg)->cmsg_len + sizeof(long) - 1) & ~(long)(sizeof(long) - 1))
#define __CMSG_NEXT(cmsg) ((unsigned char *)(cmsg) + __CMSG_LEN(cmsg))
#define __MHDR_END(mhdr) ((unsigned char *)(mhdr)->msg_control + (mhdr)->msg_controllen)
#define CMSG_NXTHDR(mhdr, cmsg) ((cmsg)->cmsg_len < sizeof (struct cmsghdr) || \
	__CMSG_LEN(cmsg) + sizeof(struct cmsghdr) >= __MHDR_END(mhdr) - (unsigned char *)(cmsg) \
	? 0 : (struct cmsghdr *)__CMSG_NEXT(cmsg))

#define CMSG_FIRSTHDR(mhdr) ((size_t) (mhdr)->msg_controllen >= sizeof (struct cmsghdr) ? (struct cmsghdr *) (mhdr)->msg_control : (struct cmsghdr *) 0)

#define CMSG_ALIGN(len) (((len) + sizeof (size_t) - 1) & (size_t) ~(sizeof (size_t) - 1))

#define CMSG_SPACE(len) (CMSG_ALIGN (len) + CMSG_ALIGN (sizeof (struct cmsghdr)))

ssize_t sendmsg(int fd, const struct msghdr *msg, int flags)
{
#if LONG_MAX > INT_MAX
	struct msghdr h;
	/* Kernels before 2.6.38 set SCM_MAX_FD to 255, allocate enough
	 * space to support an SCM_RIGHTS ancillary message with 255 fds.
	 * Kernels since 2.6.38 set SCM_MAX_FD to 253. */
	struct cmsghdr chbuf[CMSG_SPACE(255*sizeof(int))/sizeof(struct cmsghdr)+1], *c;
	if (msg) {
		h = *msg;
		h.__pad1 = h.__pad2 = 0;
		msg = &h;
		if (h.msg_controllen) {
			if (h.msg_controllen > sizeof chbuf) {
				errno = ENOMEM;
				return -1;
			}
			memcpy(chbuf, h.msg_control, h.msg_controllen);
			h.msg_control = chbuf;
			for (c=CMSG_FIRSTHDR(&h); c; c=CMSG_NXTHDR(&h,c))
				c->__pad1 = 0;
		}
	}
#endif
    return ax_sendmsg(fd, msg, flags);
}

#endif // AX_CONFIG_NET
