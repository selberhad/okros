#!/usr/bin/env perl
use strict;
use warnings;
use IO::Socket::UNIX;
use JSON::PP;
use Time::HiRes qw(sleep);
use POSIX qw(strftime);

# Capture Nodeka splash screens via headless mode for debugging vertical truncation

my $INSTANCE = "capture";
my $SOCKET_PATH = "/tmp/okros/${INSTANCE}.sock";
my $CAPTURES_DIR = "test_captures/nodeka";
my $WAIT_TIME = 3;  # seconds to wait for splash screen

sub main {
    my ($num_captures) = @ARGV;
    $num_captures //= 5;  # default to 5 captures

    # Create captures directory
    system("mkdir", "-p", $CAPTURES_DIR);

    print "Capturing $num_captures Nodeka splash screens...\n";

    for my $i (1..$num_captures) {
        print "\n=== Capture $i/$num_captures ===\n";
        capture_splash($i);
        sleep 2 if $i < $num_captures;  # Brief pause between captures
    }

    print "\nDone! Captures saved to $CAPTURES_DIR/\n";
    list_captures();
}

sub capture_splash {
    my ($attempt) = @_;

    # Remove old socket if exists
    unlink $SOCKET_PATH if -e $SOCKET_PATH;

    # Start headless instance
    print "Starting headless okros...\n";
    my $pid = fork();
    die "Fork failed: $!" unless defined $pid;

    if ($pid == 0) {
        # Child process - run okros headless
        exec("cargo", "run", "--quiet", "--",
             "--headless", "--instance", $INSTANCE)
            or die "exec failed: $!";
    }

    # Parent - wait for socket to exist
    my $waited = 0;
    while (!-S $SOCKET_PATH && $waited < 5) {
        sleep 0.1;
        $waited += 0.1;
    }

    unless (-S $SOCKET_PATH) {
        kill 'TERM', $pid;
        die "Socket $SOCKET_PATH not created\n";
    }

    print "Connecting to control socket...\n";
    my $sock = IO::Socket::UNIX->new(
        Type => SOCK_STREAM,
        Peer => $SOCKET_PATH,
    ) or die "Cannot connect to $SOCKET_PATH: $!";

    # Connect to Nodeka
    print "Opening connection to nodeka.com:23...\n";
    send_command($sock, { cmd => "connect", data => "nodeka.com:23" });
    my $connect_resp = read_response($sock);

    if ($connect_resp->{event} eq 'Error') {
        die "Failed to connect: $connect_resp->{message}\n";
    }

    # Wait for splash screen to arrive
    print "Waiting ${WAIT_TIME}s for splash screen...\n";
    sleep $WAIT_TIME;

    # Get buffer (viewport)
    print "Retrieving buffer...\n";
    send_command($sock, { cmd => "get_buffer" });
    my $response = read_response($sock);

    # Save to file
    my $timestamp = strftime("%Y%m%d_%H%M%S", localtime);
    my $filename = "$CAPTURES_DIR/splash_${timestamp}_${attempt}.txt";

    if ($response && $response->{event} eq 'Buffer' && $response->{lines}) {
        open my $fh, '>', $filename or die "Cannot write $filename: $!";
        print $fh "# Nodeka splash screen capture\n";
        print $fh "# Captured: $timestamp (attempt $attempt)\n";
        print $fh "# Lines: " . scalar(@{$response->{lines}}) . "\n";
        print $fh "#" . "=" x 70 . "\n\n";

        for my $line (@{$response->{lines}}) {
            print $fh $line, "\n";
        }
        close $fh;

        my $lines = scalar(@{$response->{lines}});
        print "Saved $lines lines to $filename\n";

        # Also save raw JSON for detailed analysis
        my $json_file = "$filename.json";
        open my $jfh, '>', $json_file or die "Cannot write $json_file: $!";
        print $jfh JSON::PP->new->pretty->encode($response);
        close $jfh;
    } else {
        print "Warning: No buffer data received\n";
        if ($response) {
            print "Response: ", JSON::PP->new->encode($response), "\n";
        }
    }

    # Cleanup
    send_command($sock, { cmd => "quit" });
    close $sock;

    # Kill okros if still running
    sleep 0.5;
    kill 'TERM', $pid;
    waitpid($pid, 0);

    # Remove socket
    unlink $SOCKET_PATH if -e $SOCKET_PATH;
}

sub send_command {
    my ($sock, $cmd) = @_;
    my $json = JSON::PP->new->encode($cmd);
    print $sock $json, "\n";
}

sub read_response {
    my ($sock) = @_;
    my $line = <$sock>;
    return undef unless $line;
    return JSON::PP->new->decode($line);
}

sub list_captures {
    print "\nCaptured files:\n";
    system("ls", "-lh", $CAPTURES_DIR);
}

main() unless caller;
