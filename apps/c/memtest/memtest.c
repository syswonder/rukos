#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

int main(int argc, char* argv[])
{
    puts("Running memory tests...");
    uintptr_t *brk = (uintptr_t *)malloc(0);
    printf("top of heap=%p\n", brk);

    int n = 9;
    int i = 0;
    uintptr_t **p = (uintptr_t **)malloc(n * sizeof(uint64_t));
    printf("%d(+8)Byte allocated: p=%p\n", n * sizeof(uint64_t), p, p[1]);
    printf("allocate %d(+8)Byte for %d times:\n", sizeof(uint64_t), n);
    for (i = 0; i < n; i++) {
        p[i] = (uintptr_t *)malloc(sizeof(uint64_t));
        *p[i] = 233;
        printf("allocated addr=%p\n", p[i]);
    }
    for (i = 0; i < n; i++) {
        free(p[i]);
    }
    free(p);
    puts("Memory tests run OK!");
	puts("Running environ tests...");
	char *env1 = "env1", *ex1 = "ex1", *ex2 = "ex_2";
    if(setenv(env1, ex1, 1) || strcmp(ex1, getenv(env1))) puts("set new env is wrong");
	if(setenv(env1, ex2, 1) || strcmp(ex2, getenv(env1))) puts("set old env is wrong");
	if(setenv(env1, ex1, 0) || strcmp(ex2, getenv(env1))) puts("override the old env is wrong");
	puts("Environ tests run OK!");
	puts("Running argv tests...");
	if (argc != 3) puts("args num is wrong");
	if (strcmp(argv[0], "abc") || strcmp(argv[1], "def") || strcmp(argv[2], "ghi")) puts("argv is wrong");
	if(argv[3] != NULL) puts("argv is wrong");
	puts("Argv tests run OK!");
    return 0;
}
