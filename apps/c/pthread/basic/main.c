/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#include <assert.h>
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

void *ThreadFunc1(void *arg)
{
    if (arg == NULL) {
        puts("Pass NULL argument");
        return NULL;
    } else {
        char buf[64];
        sprintf(buf, "Recieve: %s", (char *)arg);
        puts(buf);
        return "Child thread return message";
    }
}

void *ThreadFunc2(void *arg)
{
    puts("A message before call pthread_exit");
    pthread_exit("Exit message");
    puts("This message should not be printed");
}

static pthread_mutex_t lock = PTHREAD_MUTEX_INITIALIZER;

void *ThreadFunc3(void *arg)
{
    pthread_mutex_lock(&lock);

    int value = *(int *)arg;

    // long operation
    for (int i = 0; i < 100000; i++) getpid();

    *(int *)arg = value + 1;

    pthread_mutex_unlock(&lock);
    return 0;
}

void test_create_join()
{
    int res;
    char *s = "Main thread pass message";
    void *thread_result;
    pthread_t t1, t2;
    res = pthread_create(&t1, NULL, ThreadFunc1, NULL);
    if (res != 0) {
        puts("fail to create thread1");
        return;
    }

    res = pthread_join(t1, NULL);
    if (res != 0) {
        puts("First pthread join fail");
    }

    res = pthread_create(&t2, NULL, ThreadFunc1, (void *)s);
    if (res != 0) {
        puts("fail to create thread2");
        return;
    }

    res = pthread_join(t2, &thread_result);
    if (res != 0) {
        puts("Second pthread join fail");
    }

    printf("test_create_join: %s\n", (char *)thread_result);
}

void test_create_exit()
{
    int res;
    void *thread_result;
    pthread_t t1;

    res = pthread_create(&t1, NULL, ThreadFunc2, NULL);
    if (res != 0) {
        puts("pthread create fail");
        return;
    }

    res = pthread_join(t1, &thread_result);
    if (res != 0) {
        puts("pthread join fail");
    }

    printf("test_create_exit: %s\n", (char *)thread_result);
}

void test_mutex()
{
    const int NUM_THREADS = 100;
    int data = 0;
    pthread_t t[NUM_THREADS];

    for (int i = 0; i < NUM_THREADS; i++) {
        int res = pthread_create(&t[i], NULL, ThreadFunc3, &data);
        if (res != 0) {
            puts("pthread create fail");
            return;
        }
    }

    for (int i = 0; i < NUM_THREADS; i++) {
        int res = pthread_join(t[i], NULL);
        if (res != 0) {
            puts("pthread join fail");
        }
    }

    printf("test_mutex: data = %d\n", data);
    assert(data == NUM_THREADS);
}

// test condition variable
pthread_cond_t condvar;
int A = 0;

void *first(void *arg)
{
    sleep(5);
    puts("First work, Change A --> 1 and wakeup Second");
    pthread_mutex_lock(&lock);
    A = 1;
    pthread_cond_signal(&condvar);
    pthread_mutex_unlock(&lock);
    return NULL;

}

void *second(void *arg)
{
    puts("Second want to continue,but need to wait A=1");
    pthread_mutex_lock(&lock);
    while (A == 0) {
        printf("Second: A is {}", A);
        pthread_cond_wait(&condvar, &lock);
    }
    printf("A is {}, Second can work now", A);
    pthread_mutex_unlock(&lock);
    return NULL;
}

void test_condvar()
{
    pthread_t t1, t2;
    pthread_cond_init(&condvar, NULL);

    pthread_create(&t1, NULL, first, NULL);
    pthread_create(&t2, NULL, second, NULL);

    pthread_join(t1, NULL);
    pthread_join(t2, NULL);
}

int main()
{
    pthread_t main_thread = pthread_self();
    assert(main_thread != 0);

    test_create_join();
    test_create_exit();
    test_mutex();
	test_condvar();
    puts("(C)Pthread basic tests run OK!");

    return 0;
}
