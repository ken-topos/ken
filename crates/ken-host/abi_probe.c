#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <unistd.h>

/* Clean-room clearance evt_5fx7gmprrk07b: fixed names and integers only. */
int main(void) {
    printf("O_RDONLY=%lld\n", (long long)O_RDONLY);
    printf("O_WRONLY=%lld\n", (long long)O_WRONLY);
    printf("O_RDWR=%lld\n", (long long)O_RDWR);
    printf("O_APPEND=%lld\n", (long long)O_APPEND);
    printf("O_CREAT=%lld\n", (long long)O_CREAT);
    printf("O_EXCL=%lld\n", (long long)O_EXCL);
    printf("O_TRUNC=%lld\n", (long long)O_TRUNC);
    printf("O_DIRECTORY=%lld\n", (long long)O_DIRECTORY);
    printf("O_NOFOLLOW=%lld\n", (long long)O_NOFOLLOW);
    printf("O_CLOEXEC=%lld\n", (long long)O_CLOEXEC);
    printf("AT_REMOVEDIR=%lld\n", (long long)AT_REMOVEDIR);
    printf("MODE_FILE_CREATE=%lld\n", (long long)(S_IRUSR | S_IWUSR | S_IRGRP | S_IWGRP | S_IROTH | S_IWOTH));
    printf("MODE_DIRECTORY_CREATE=%lld\n", (long long)(S_IRWXU | S_IRWXG | S_IRWXO));
    printf("SYS_OPENAT=%lld\n", (long long)SYS_openat);
    printf("SYS_MKDIRAT=%lld\n", (long long)SYS_mkdirat);
    printf("SYS_UNLINKAT=%lld\n", (long long)SYS_unlinkat);
    printf("SYS_RENAMEAT=%lld\n", (long long)SYS_renameat);
    printf("SYS_READLINKAT=%lld\n", (long long)SYS_readlinkat);
    printf("ERRNO_ENOENT=%lld\n", (long long)ENOENT);
    printf("ERRNO_EEXIST=%lld\n", (long long)EEXIST);
    return 0;
}
