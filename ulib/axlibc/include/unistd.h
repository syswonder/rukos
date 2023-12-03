/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#ifndef __UNISTD_H__
#define __UNISTD_H__

#include <features.h>
#include <stddef.h>
#include <sys/stat.h>

#define _SC_LEVEL1_DCACHE_LINESIZE 130

#define STDIN_FILENO  0
#define STDOUT_FILENO 1
#define STDERR_FILENO 2

#define SEEK_SET  0
#define SEEK_CUR  1
#define SEEK_END  2
#define SEEK_DATA 3
#define SEEK_HOLE 4

int pipe(int[2]);
int pipe2(int[2], int);
int close(int);
int posix_close(int, int);
int dup(int);
int dup2(int, int);
int dup3(int, int, int);
off_t lseek(int, off_t, int);
int fsync(int);
int fdatasync(int);

ssize_t read(int, void *, size_t);
ssize_t write(int, const void *, size_t);
ssize_t pread(int, void *, size_t, off_t);
ssize_t pwrite(int, const void *, size_t, off_t);

ssize_t pread64(int, void *, size_t, off_t);

int chown(const char *, uid_t, gid_t);
int fchown(int, uid_t, gid_t);
int lchown(const char *, uid_t, gid_t);
int fchownat(int, const char *, uid_t, gid_t, int);

int link(const char *, const char *);
int linkat(int, const char *, int, const char *, int);
int symlink(const char *, const char *);
int symlinkat(const char *, int, const char *);
ssize_t readlink(const char *__restrict, char *__restrict, size_t);
ssize_t readlinkat(int, const char *__restrict, char *__restrict, size_t);
int unlink(const char *);
int unlinkat(int, const char *, int);
int rmdir(const char *);
int truncate(const char *, off_t);
int ftruncate(int, off_t);

#define F_OK 0
#define R_OK 4
#define W_OK 2
#define X_OK 1

int access(const char *, int);
int faccessat(int, const char *, int, int);

int chdir(const char *);
int fchdir(int);
char *getcwd(char *, size_t);

unsigned alarm(unsigned);
unsigned ualarm(unsigned, unsigned);
unsigned sleep(unsigned);
int pause(void);
int usleep(unsigned);

pid_t fork(void);
int execve(const char *, char *const[], char *const[]);
_Noreturn void _exit(int);

int getpagesize(void);

pid_t getpid(void);
pid_t getppid(void);
pid_t getpgrp(void);
pid_t getpgid(pid_t);
int setpgid(pid_t, pid_t);
pid_t setsid(void);
pid_t getsid(pid_t);
char *ttyname(int);
int ttyname_r(int, char *, size_t);
int isatty(int);
pid_t tcgetpgrp(int);
int tcsetpgrp(int, pid_t);

uid_t getuid(void);
uid_t geteuid(void);
gid_t getgid(void);
gid_t getegid(void);
int getgroups(int, gid_t[]);
int setuid(uid_t);
int seteuid(uid_t);
int setgid(gid_t);
int setegid(gid_t);

int gethostname(char *, size_t);

int chown(const char *, uid_t, gid_t);

long sysconf(int);

#define _SC_ARG_MAX                      0
#define _SC_CHILD_MAX                    1
#define _SC_CLK_TCK                      2
#define _SC_NGROUPS_MAX                  3
#define _SC_OPEN_MAX                     4
#define _SC_STREAM_MAX                   5
#define _SC_TZNAME_MAX                   6
#define _SC_JOB_CONTROL                  7
#define _SC_SAVED_IDS                    8
#define _SC_REALTIME_SIGNALS             9
#define _SC_PRIORITY_SCHEDULING          10
#define _SC_TIMERS                       11
#define _SC_ASYNCHRONOUS_IO              12
#define _SC_PRIORITIZED_IO               13
#define _SC_SYNCHRONIZED_IO              14
#define _SC_FSYNC                        15
#define _SC_MAPPED_FILES                 16
#define _SC_MEMLOCK                      17
#define _SC_MEMLOCK_RANGE                18
#define _SC_MEMORY_PROTECTION            19
#define _SC_MESSAGE_PASSING              20
#define _SC_SEMAPHORES                   21
#define _SC_SHARED_MEMORY_OBJECTS        22
#define _SC_AIO_LISTIO_MAX               23
#define _SC_AIO_MAX                      24
#define _SC_AIO_PRIO_DELTA_MAX           25
#define _SC_DELAYTIMER_MAX               26
#define _SC_MQ_OPEN_MAX                  27
#define _SC_MQ_PRIO_MAX                  28
#define _SC_VERSION                      29
#define _SC_PAGE_SIZE                    30
#define _SC_PAGESIZE                     30 /* !! */
#define _SC_RTSIG_MAX                    31
#define _SC_SEM_NSEMS_MAX                32
#define _SC_SEM_VALUE_MAX                33
#define _SC_SIGQUEUE_MAX                 34
#define _SC_TIMER_MAX                    35
#define _SC_BC_BASE_MAX                  36
#define _SC_BC_DIM_MAX                   37
#define _SC_BC_SCALE_MAX                 38
#define _SC_BC_STRING_MAX                39
#define _SC_COLL_WEIGHTS_MAX             40
#define _SC_EXPR_NEST_MAX                42
#define _SC_LINE_MAX                     43
#define _SC_RE_DUP_MAX                   44
#define _SC_2_VERSION                    46
#define _SC_2_C_BIND                     47
#define _SC_2_C_DEV                      48
#define _SC_2_FORT_DEV                   49
#define _SC_2_FORT_RUN                   50
#define _SC_2_SW_DEV                     51
#define _SC_2_LOCALEDEF                  52
#define _SC_UIO_MAXIOV                   60 /* !! */
#define _SC_IOV_MAX                      60
#define _SC_THREADS                      67
#define _SC_THREAD_SAFE_FUNCTIONS        68
#define _SC_GETGR_R_SIZE_MAX             69
#define _SC_GETPW_R_SIZE_MAX             70
#define _SC_LOGIN_NAME_MAX               71
#define _SC_TTY_NAME_MAX                 72
#define _SC_THREAD_DESTRUCTOR_ITERATIONS 73
#define _SC_THREAD_KEYS_MAX              74
#define _SC_THREAD_STACK_MIN             75
#define _SC_THREAD_THREADS_MAX           76
#define _SC_THREAD_ATTR_STACKADDR        77
#define _SC_THREAD_ATTR_STACKSIZE        78
#define _SC_THREAD_PRIORITY_SCHEDULING   79
#define _SC_THREAD_PRIO_INHERIT          80
#define _SC_THREAD_PRIO_PROTECT          81
#define _SC_THREAD_PROCESS_SHARED        82
#define _SC_NPROCESSORS_CONF             83
#define _SC_NPROCESSORS_ONLN             84
#define _SC_PHYS_PAGES                   85
#define _SC_AVPHYS_PAGES                 86
#define _SC_ATEXIT_MAX                   87
#define _SC_PASS_MAX                     88
#define _SC_XOPEN_VERSION                89
#define _SC_XOPEN_XCU_VERSION            90
#define _SC_XOPEN_UNIX                   91
#define _SC_XOPEN_CRYPT                  92
#define _SC_XOPEN_ENH_I18N               93
#define _SC_XOPEN_SHM                    94
#define _SC_2_CHAR_TERM                  95
#define _SC_2_UPE                        97
#define _SC_XOPEN_XPG2                   98
#define _SC_XOPEN_XPG3                   99
#define _SC_XOPEN_XPG4                   100
#define _SC_NZERO                        109
#define _SC_XBS5_ILP32_OFF32             125
#define _SC_XBS5_ILP32_OFFBIG            126
#define _SC_XBS5_LP64_OFF64              127
#define _SC_XBS5_LPBIG_OFFBIG            128
#define _SC_XOPEN_LEGACY                 129
#define _SC_XOPEN_REALTIME               130
#define _SC_XOPEN_REALTIME_THREADS       131
#define _SC_ADVISORY_INFO                132
#define _SC_BARRIERS                     133
#define _SC_CLOCK_SELECTION              137
#define _SC_CPUTIME                      138
#define _SC_THREAD_CPUTIME               139
#define _SC_MONOTONIC_CLOCK              149
#define _SC_READER_WRITER_LOCKS          153
#define _SC_SPIN_LOCKS                   154
#define _SC_REGEXP                       155
#define _SC_SHELL                        157
#define _SC_SPAWN                        159
#define _SC_SPORADIC_SERVER              160
#define _SC_THREAD_SPORADIC_SERVER       161
#define _SC_TIMEOUTS                     164
#define _SC_TYPED_MEMORY_OBJECTS         165
#define _SC_2_PBS                        168
#define _SC_2_PBS_ACCOUNTING             169
#define _SC_2_PBS_LOCATE                 170
#define _SC_2_PBS_MESSAGE                171
#define _SC_2_PBS_TRACK                  172
#define _SC_SYMLOOP_MAX                  173
#define _SC_STREAMS                      174
#define _SC_2_PBS_CHECKPOINT             175
#define _SC_V6_ILP32_OFF32               176
#define _SC_V6_ILP32_OFFBIG              177
#define _SC_V6_LP64_OFF64                178
#define _SC_V6_LPBIG_OFFBIG              179
#define _SC_HOST_NAME_MAX                180
#define _SC_TRACE                        181
#define _SC_TRACE_EVENT_FILTER           182
#define _SC_TRACE_INHERIT                183
#define _SC_TRACE_LOG                    184

#define _SC_IPV6                       235
#define _SC_RAW_SOCKETS                236
#define _SC_V7_ILP32_OFF32             237
#define _SC_V7_ILP32_OFFBIG            238
#define _SC_V7_LP64_OFF64              239
#define _SC_V7_LPBIG_OFFBIG            240
#define _SC_SS_REPL_MAX                241
#define _SC_TRACE_EVENT_NAME_MAX       242
#define _SC_TRACE_NAME_MAX             243
#define _SC_TRACE_SYS_MAX              244
#define _SC_TRACE_USER_EVENT_MAX       245
#define _SC_XOPEN_STREAMS              246
#define _SC_THREAD_ROBUST_PRIO_INHERIT 247
#define _SC_THREAD_ROBUST_PRIO_PROTECT 248

#endif
