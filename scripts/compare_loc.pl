#!/usr/bin/env perl
# Compare Lines of Code between C++ reference and Rust implementation
#
# Uses cloc to count code, comments, and blank lines

use strict;
use warnings;
use JSON::PP;

# Paths
my $cpp_ref = "mcl-cpp-reference";
my $rust_src = "src";

# Check if cloc is available
if (!`which cloc 2>/dev/null`) {
    die "Error: cloc not found. Install with: brew install cloc\n";
}

# Count C++ reference
print "Counting C++ reference code...\n";
my $cpp_json = `cloc --json --quiet $cpp_ref 2>/dev/null`;
my $cpp_data = decode_json($cpp_json);

# Count Rust implementation
print "Counting Rust implementation...\n";
my $rust_json = `cloc --json --quiet $rust_src 2>/dev/null`;
my $rust_data = decode_json($rust_json);

# Extract totals
my $cpp_code = $cpp_data->{'C++'}{'code'} + $cpp_data->{'C/C++ Header'}{'code'};
my $cpp_comment = $cpp_data->{'C++'}{'comment'} + $cpp_data->{'C/C++ Header'}{'comment'};
my $cpp_blank = $cpp_data->{'C++'}{'blank'} + $cpp_data->{'C/C++ Header'}{'blank'};
my $cpp_total = $cpp_code + $cpp_comment + $cpp_blank;

my $rust_code = $rust_data->{'Rust'}{'code'} || 0;
my $rust_comment = $rust_data->{'Rust'}{'comment'} || 0;
my $rust_blank = $rust_data->{'Rust'}{'blank'} || 0;
my $rust_total = $rust_code + $rust_comment + $rust_blank;

# Calculate ratios
my $code_ratio = $cpp_code > 0 ? sprintf("%.2f", $rust_code / $cpp_code) : "N/A";
my $total_ratio = $cpp_total > 0 ? sprintf("%.2f", $rust_total / $cpp_total) : "N/A";

# Print comparison
print "\n";
print "=" x 70 . "\n";
print "  Lines of Code Comparison: C++ MCL → Rust okros\n";
print "=" x 70 . "\n\n";

printf "%-20s %15s %15s %15s\n", "", "C++ (Reference)", "Rust (okros)", "Ratio";
print "-" x 70 . "\n";
printf "%-20s %15d %15d %15s\n", "Code", $cpp_code, $rust_code, "${code_ratio}x";
printf "%-20s %15d %15d\n", "Comments", $cpp_comment, $rust_comment;
printf "%-20s %15d %15d\n", "Blank Lines", $cpp_blank, $rust_blank;
print "-" x 70 . "\n";
printf "%-20s %15d %15d %15s\n", "TOTAL", $cpp_total, $rust_total, "${total_ratio}x";
print "=" x 70 . "\n\n";

# Calculate percentage difference
my $code_diff_pct = $cpp_code > 0 ? sprintf("%.1f%%", (($rust_code - $cpp_code) / $cpp_code) * 100) : "N/A";
my $sign = ($rust_code - $cpp_code) >= 0 ? "+" : "";

print "Summary:\n";
print "  • Rust code is ${code_ratio}x the size of C++ code\n";
print "  • Difference: ${sign}${code_diff_pct} ($rust_code - $cpp_code = " . ($rust_code - $cpp_code) . " lines)\n";

# Interpret results
print "\nInterpretation:\n";
if ($code_ratio < 1.0) {
    print "  ✅ Rust implementation is MORE CONCISE than C++\n";
} elsif ($code_ratio > 1.2) {
    print "  ⚠️  Rust implementation is LARGER than C++\n";
} else {
    print "  ✅ Rust implementation is COMPARABLE to C++ in size\n";
}

# File count comparison
my $cpp_files = ($cpp_data->{'C++'}{'nFiles'} || 0) + ($cpp_data->{'C/C++ Header'}{'nFiles'} || 0);
my $rust_files = $rust_data->{'Rust'}{'nFiles'} || 0;

print "\nFile Counts:\n";
printf "  • C++:  %3d files (.cc + .h)\n", $cpp_files;
printf "  • Rust: %3d files (.rs)\n", $rust_files;

# Module breakdown (optional)
print "\n" . "=" x 70 . "\n";
print "Detailed Breakdown by Language:\n";
print "=" x 70 . "\n\n";

print "C++ Reference:\n";
foreach my $lang (sort keys %$cpp_data) {
    next if $lang eq 'header' || $lang eq 'SUM';
    my $files = $cpp_data->{$lang}{'nFiles'} || 0;
    my $code = $cpp_data->{$lang}{'code'} || 0;
    printf "  %-20s %4d files, %6d lines\n", $lang, $files, $code;
}

print "\nRust Implementation:\n";
foreach my $lang (sort keys %$rust_data) {
    next if $lang eq 'header' || $lang eq 'SUM';
    my $files = $rust_data->{$lang}{'nFiles'} || 0;
    my $code = $rust_data->{$lang}{'code'} || 0;
    printf "  %-20s %4d files, %6d lines\n", $lang, $files, $code;
}

# File-by-file mapping to identify cut corners
print "\n" . "=" x 80 . "\n";
print "File-by-File Comparison (C++ → Rust)\n";
print "=" x 80 . "\n\n";

# Define mapping of C++ files to Rust equivalents
my %file_mapping = (
    'main.cc' => 'main.rs',
    'Window.cc' => 'window.rs',
    'OutputWindow.cc' => 'output_window.rs',
    'InputLine.cc' => 'input_line.rs',
    'StatusLine.cc' => 'status_line.rs',
    'Screen.cc' => 'screen.rs',
    'Session.cc' => 'session.rs',
    'TTY.cc' => 'tty.rs',
    'Curses.cc' => 'curses.rs',
    'Socket.cc' => 'socket.rs',
    'Selectable.cc' => 'selectable.rs',
    'Selection.cc' => 'selection.rs',
    'MUD.cc' => 'mud.rs',
    'Config.cc' => 'config.rs',
    'Alias.cc' => 'alias.rs',
    'Hotkey.cc' => 'macro_def.rs',
    'Interpreter.cc' => 'plugins/stack.rs',
    'plugins/PythonEmbeddedInterpreter.cc' => 'plugins/python.rs',
    'plugins/PerlEmbeddedInterpreter.cc' => 'plugins/perl.rs',

    # Headers (major ones)
    'h/Window.h' => 'window.rs',
    'h/Session.h' => 'session.rs',
    'h/Config.h' => 'config.rs',
    'h/Action.h' => 'action.rs',
    'h/TTY.h' => 'tty.rs',
    'h/Socket.h' => 'socket.rs',
    'h/mccpDecompress.h' => 'mccp.rs',
    'h/OutputWindow.h' => 'output_window.rs',
    'h/InputLine.h' => 'input_line.rs',

    # Intentionally skipped/deferred
    'String.cc' => undef,  # Using stdlib
    'Buffer.cc' => undef,  # Using Vec<u8>
    'StaticBuffer.cc' => undef,  # Not needed
    'Chat.cc' => undef,  # Deferred (inter-client chat)
    'Borg.cc' => undef,  # Deferred (privacy concern)
    'Group.cc' => undef,  # Deferred (post-MVP)

    # Missing (needs investigation/porting)
    'InputBox.cc' => undef,  # NOT PORTED - needs implementation
    'Pipe.cc' => undef,  # Needs investigation
    'Shell.cc' => undef,  # Needs investigation
    'Option.cc' => undef,  # Needs investigation
    'misc.cc' => undef,  # Needs investigation
    'Embedded.cc' => undef,  # Split across plugins
);

my @suspicious;
my @missing;

printf "%-40s %8s → %-25s %8s %8s\n", "C++ File", "Lines", "Rust File", "Lines", "Ratio";
print "-" x 80 . "\n";

for my $cpp_file (sort keys %file_mapping) {
    my $rust_file = $file_mapping{$cpp_file};
    my $cpp_path = "$cpp_ref/$cpp_file";

    next unless -f $cpp_path;

    my $cpp_lines = `wc -l < "$cpp_path" 2>/dev/null`;
    chomp $cpp_lines;
    $cpp_lines ||= 0;

    if (!defined $rust_file) {
        printf "%-40s %8d → %-25s %8s %8s\n",
            $cpp_file, $cpp_lines, "SKIPPED/DEFERRED", "-", "-";
        next;
    }

    my $rust_path = "$rust_src/$rust_file";
    if (!-f $rust_path) {
        printf "%-40s %8d → %-25s %8s %8s ⚠️ MISSING\n",
            $cpp_file, $cpp_lines, $rust_file, "-", "-";
        push @missing, {cpp => $cpp_file, rust => $rust_file, cpp_lines => $cpp_lines};
        next;
    }

    my $rust_lines = `wc -l < "$rust_path" 2>/dev/null`;
    chomp $rust_lines;
    $rust_lines ||= 0;

    my $ratio = $cpp_lines > 0 ? $rust_lines / $cpp_lines : 0;
    my $ratio_str = sprintf("%.2f", $ratio);

    my $flag = "";
    if ($ratio < 0.4) {
        $flag = " ⚠️ SUSPICIOUS";
        push @suspicious, {
            cpp => $cpp_file,
            rust => $rust_file,
            cpp_lines => $cpp_lines,
            rust_lines => $rust_lines,
            ratio => $ratio
        };
    } elsif ($ratio < 0.6) {
        $flag = " 🔍 SHORT";
    }

    printf "%-40s %8d → %-25s %8d %8s%s\n",
        $cpp_file, $cpp_lines, $rust_file, $rust_lines, $ratio_str, $flag;
}

# Print issues summary
print "\n" . "=" x 80 . "\n";
print "ISSUES REQUIRING INVESTIGATION\n";
print "=" x 80 . "\n\n";

if (@missing) {
    print "MISSING FILES (C++ code not ported):\n";
    for my $issue (@missing) {
        printf "  • %s (%d lines) → %s NOT FOUND\n",
            $issue->{cpp}, $issue->{cpp_lines}, $issue->{rust};
    }
    print "\n";
}

if (@suspicious) {
    print "SUSPICIOUSLY SHORT (<40% of C++ size - likely incomplete port):\n";
    for my $issue (sort { $a->{ratio} <=> $b->{ratio} } @suspicious) {
        printf "  • %s (%d lines) → %s (%d lines) = %.0f%%\n",
            $issue->{cpp}, $issue->{cpp_lines},
            $issue->{rust}, $issue->{rust_lines},
            $issue->{ratio} * 100;
    }
    print "\n";
}

if (!@missing && !@suspicious) {
    print "✅ No obvious missing or suspiciously short files detected.\n\n";
}

print "\n";
