#include <stdio.h>
#include <unistd.h>

int main() {
    printf("isatty(0) = %d\n", isatty(0));
    printf("isatty(1) = %d\n", isatty(1));
    printf("isatty(2) = %d\n", isatty(2));
    return 0;
}
