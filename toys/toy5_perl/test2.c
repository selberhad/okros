// Minimal C test with PERL_SYS_INIT3 for threaded Perl
#include <EXTERN.h>
#include <perl.h>
#include <stdio.h>

int main(int argc, char **argv, char **env) {
    PerlInterpreter *my_perl;

    // Required for threaded Perl on some platforms
    PERL_SYS_INIT3(&argc, &argv, &env);

    printf("Allocating...\n");
    my_perl = perl_alloc();

    printf("Constructing...\n");
    perl_construct(my_perl);

    printf("Parsing...\n");
    char *embedding[] = { "", "-e", "0", NULL };
    int result = perl_parse(my_perl, NULL, 3, embedding, NULL);

    printf("perl_parse returned: %d\n", result);

    if (result == 0) {
        printf("SUCCESS! Running perl_run...\n");
        perl_run(my_perl);

        printf("Running simple eval...\n");
        perl_eval_pv("print \"Hello from Perl!\\n\";", TRUE);
    }

    perl_destruct(my_perl);
    perl_free(my_perl);
    PERL_SYS_TERM();

    return result;
}
