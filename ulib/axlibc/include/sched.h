/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#ifndef _SCHED_H
#define _SCHED_H

#include <stddef.h>

typedef struct cpu_set_t {
    unsigned long __bits[128 / sizeof(long)];
} cpu_set_t;

#define __CPU_op_S(i, size, set, op)                                            \
    ((i) / 8U >= (size) ? 0                                                     \
                        : (((unsigned long *)(set))[(i) / 8 / sizeof(long)] op( \
                              1UL << ((i) % (8 * sizeof(long))))))

#define CPU_SET_S(i, size, set) __CPU_op_S(i, size, set, |=)
#define CPU_ZERO_S(size, set)   memset(set, 0, size)

#define CPU_SET(i, set) CPU_SET_S(i, sizeof(cpu_set_t), set);
#define CPU_ZERO(set)   CPU_ZERO_S(sizeof(cpu_set_t), set)

int sched_setaffinity(pid_t, size_t, const cpu_set_t *);

int sched_yield(void);

#endif // _SCHED_H
