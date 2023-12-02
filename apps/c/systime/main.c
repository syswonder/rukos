#include <stdio.h>
#include <sys/time.h>

int main()
{
    struct timeval tv;
    if (gettimeofday(&tv, NULL) != 0 ) {
        perror("gettimeofday");
        return -1;
    }

    printf("now time: %ld : %ld\n", tv.tv_sec,tv.tv_usec);

    usleep(3000000);

    if (gettimeofday(&tv, NULL) != 0 ) {
        perror("gettimeofday");
        return -1;
    }

    printf("now time: %ld : %ld\n", tv.tv_sec,tv.tv_usec);

    struct timeval new_time;
    new_time.tv_sec = 1731110400;
    new_time.tv_usec = 0;

    if (settimeofday(&new_time, NULL) != 0 ) {
        perror("settimeofday");
        return -1;
    }
    if (gettimeofday(&tv, NULL) != 0 ) {
        perror("gettimeofday");
        return -1;
    }

    printf("now time: %ld : %ld\n", tv.tv_sec,tv.tv_usec);
    return 0;

}