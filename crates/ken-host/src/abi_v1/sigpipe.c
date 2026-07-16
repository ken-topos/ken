#define _POSIX_C_SOURCE 200809L
#include <signal.h>
#include <stddef.h>

int ken_host_abi_v1_establish_sigpipe_ignore(void) {
    struct sigaction action = {0};
    action.sa_handler = SIG_IGN;
    if (sigemptyset(&action.sa_mask) != 0) {
        return -1;
    }
    action.sa_flags = 0;
    return sigaction(SIGPIPE, &action, NULL);
}
