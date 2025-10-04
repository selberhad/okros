/*
 * Fake TTY shim for llvm-cov compatibility
 *
 * Uses macOS DYLD_INTERPOSE to override libc functions
 * without using `script` (which breaks llvm-cov profiling).
 *
 * Usage:
 *   gcc -shared -fPIC -o faketty.dylib faketty.c
 *   DYLD_INSERT_LIBRARIES=./faketty.dylib TERM=xterm-256color cargo llvm-cov test
 */

#include <unistd.h>
#include <termios.h>
#include <sys/ioctl.h>
#include <string.h>
#include <stdarg.h>

/* Forward declare real functions */
extern int __real_isatty(int fd);
extern int __real_tcgetattr(int fd, struct termios *termios_p);
extern int __real_tcsetattr(int fd, int optional_actions, const struct termios *termios_p);

/* Our fake isatty - always returns 1 */
int fake_isatty(int fd) {
    return 1;  // Silently claim we have a TTY
}

/* macOS DYLD_INTERPOSE macro */
typedef struct interpose_s {
    void *new_func;
    void *orig_func;
} interpose_t;

/* Declare the interpose section */
__attribute__((used)) static const interpose_t interposers[]
    __attribute__((section("__DATA, __interpose"))) = {
        { (void *)fake_isatty, (void *)isatty },
};

/* Fake terminal attributes (minimal viable termios) */
int tcgetattr(int fd, struct termios *termios_p) {
    if (termios_p == NULL) {
        return -1;
    }

    /* Provide sane defaults for a VT100-compatible terminal */
    memset(termios_p, 0, sizeof(struct termios));

    /* Input flags */
    termios_p->c_iflag = ICRNL | IXON;

    /* Output flags */
    termios_p->c_oflag = OPOST | ONLCR;

    /* Control flags */
    termios_p->c_cflag = CS8 | CREAD | CLOCAL;

    /* Local flags */
    termios_p->c_lflag = ISIG | ICANON | ECHO | ECHOE | ECHOK;

    /* Control characters */
    termios_p->c_cc[VINTR] = 3;      /* ^C */
    termios_p->c_cc[VQUIT] = 28;     /* ^\ */
    termios_p->c_cc[VERASE] = 127;   /* DEL */
    termios_p->c_cc[VKILL] = 21;     /* ^U */
    termios_p->c_cc[VEOF] = 4;       /* ^D */
    termios_p->c_cc[VSTART] = 17;    /* ^Q */
    termios_p->c_cc[VSTOP] = 19;     /* ^S */
    termios_p->c_cc[VSUSP] = 26;     /* ^Z */

    /* Speed */
    cfsetispeed(termios_p, B38400);
    cfsetospeed(termios_p, B38400);

    return 0;
}

/* Accept (and ignore) terminal attribute changes */
int tcsetattr(int fd, int optional_actions, const struct termios *termios_p) {
    return 0;
}

/* Fake window size for ncurses */
int ioctl(int fd, unsigned long request, ...) {
    va_list args;
    va_start(args, request);

    /* Handle TIOCGWINSZ (get window size) */
    if (request == TIOCGWINSZ) {
        struct winsize *ws = va_arg(args, struct winsize*);
        if (ws) {
            ws->ws_row = 24;
            ws->ws_col = 80;
            ws->ws_xpixel = 0;
            ws->ws_ypixel = 0;
            va_end(args);
            return 0;
        }
    }

    va_end(args);
    /* For other ioctls, claim success (ncurses might query capabilities) */
    return 0;
}
