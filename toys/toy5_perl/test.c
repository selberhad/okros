// Minimal C test to see if we can init Perl the same way
#include <EXTERN.h>
#include <perl.h>
#include <stdio.h>

int main(int argc, char **argv, char **env) {
    PerlInterpreter *my_perl;

    printf("Allocating...\n");
    my_perl = perl_alloc();

    printf("Constructing...\n");
    perl_construct(my_perl);

    printf("Parsing...\n");
    // Try the EXACT C++ pattern
    char *args[] = {"test", "-e", "0", NULL};
    int result = perl_parse(my_perl, NULL, 3, args, env);

    printf("perl_parse returned: %d\n", result);

    if (result == 0) {
        printf("SUCCESS! Running simple eval...\n");
        perl_eval_pv("print \"Hello from Perl!\\n\";", TRUE);
    }

    perl_destruct(my_perl);
    perl_free(my_perl);

    return result;
}
