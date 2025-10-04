#!/usr/bin/env perl
# Generate LOC comparison report as markdown
# Automated by pre-commit hook

use strict;
use warnings;
use JSON::PP;

# Paths
my $cpp_ref = "mcl-cpp-reference";
my $rust_src = "src";
my $output_file = "LOC_COMPARISON.md";

# Check if cloc is available
if (!`which cloc 2>/dev/null`) {
    die "Error: cloc not found. Install with: brew install cloc\n";
}

# Count C++ reference
my $cpp_json = `cloc --json --quiet $cpp_ref 2>/dev/null`;
my $cpp_data = decode_json($cpp_json);

# Count Rust implementation
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

# File counts
my $cpp_files = ($cpp_data->{'C++'}{'nFiles'} || 0) + ($cpp_data->{'C/C++ Header'}{'nFiles'} || 0);
my $rust_files = $rust_data->{'Rust'}{'nFiles'} || 0;

# Calculate differences
my $code_diff = $rust_code - $cpp_code;
my $code_diff_pct = $cpp_code > 0 ? sprintf("%.1f%%", ($code_diff / $cpp_code) * 100) : "N/A";
my $sign = $code_diff >= 0 ? "+" : "";

# Define file mapping for detailed comparison
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

    # Intentionally skipped
    'String.cc' => undef,
    'Buffer.cc' => undef,
    'StaticBuffer.cc' => undef,
    'Chat.cc' => undef,
    'Borg.cc' => undef,
    'Group.cc' => undef,
    'InputBox.cc' => undef,
    'Pipe.cc' => undef,
    'Shell.cc' => undef,
    'Option.cc' => undef,
    'misc.cc' => undef,
    'Embedded.cc' => undef,
);

my @suspicious;

# Generate markdown output
open(my $fh, '>', $output_file) or die "Cannot write to $output_file: $!";

print $fh "# Lines of Code Comparison: C++ MCL → Rust okros\n\n";
print $fh "**⚠️ AUTO-GENERATED - DO NOT EDIT MANUALLY**\n\n";
print $fh "*This file is automatically updated by the pre-commit hook. To regenerate: `./scripts/generate_loc_report.pl`*\n\n";
print $fh "---\n\n";

# Overall summary
print $fh "## Overall Summary\n\n";

if ($code_ratio < 1.0) {
    print $fh "**Rust is " . sprintf("%.0f%%", (1 - $code_ratio) * 100) . " more concise than C++** (${code_ratio}x the size)\n\n";
} elsif ($code_ratio > 1.0) {
    print $fh "**Rust is " . sprintf("%.0f%%", ($code_ratio - 1) * 100) . " larger than C++** (${code_ratio}x the size)\n\n";
} else {
    print $fh "**Rust is comparable in size to C++** (${code_ratio}x the size)\n\n";
}

print $fh "| Metric | C++ (Reference) | Rust (okros) | Ratio |\n";
print $fh "|--------|----------------|--------------|-------|\n";
print $fh sprintf("| **Code Lines** | %s | %s | **%.2fx** |\n",
    format_number($cpp_code), format_number($rust_code), $code_ratio);
print $fh sprintf("| **Comments** | %s | %s | %.2fx |\n",
    format_number($cpp_comment), format_number($rust_comment),
    $cpp_comment > 0 ? $rust_comment / $cpp_comment : 0);
print $fh sprintf("| **Blank Lines** | %s | %s | %.2fx |\n",
    format_number($cpp_blank), format_number($rust_blank),
    $cpp_blank > 0 ? $rust_blank / $cpp_blank : 0);
print $fh sprintf("| **TOTAL** | %s | %s | **%.2fx** |\n",
    format_number($cpp_total), format_number($rust_total), $total_ratio);
print $fh sprintf("| **Files** | %d | %d | %.2fx |\n\n",
    $cpp_files, $rust_files,
    $cpp_files > 0 ? $rust_files / $cpp_files : 0);

print $fh "**Difference**: ${sign}${code_diff} lines of code (${sign}${code_diff_pct})\n\n";

# File-by-file comparison
print $fh "---\n\n";
print $fh "## File-by-File Comparison\n\n";
print $fh "| C++ File | C++ Lines | Rust File | Rust Lines | Ratio | Status |\n";
print $fh "|----------|-----------|-----------|------------|-------|--------|\n";

for my $cpp_file (sort keys %file_mapping) {
    my $rust_file = $file_mapping{$cpp_file};
    my $cpp_path = "$cpp_ref/$cpp_file";

    next unless -f $cpp_path;

    my $cpp_lines = `wc -l < "$cpp_path" 2>/dev/null`;
    chomp $cpp_lines;
    $cpp_lines ||= 0;

    if (!defined $rust_file) {
        print $fh sprintf("| %s | %d | SKIPPED/DEFERRED | - | - | ⏭️ |\n",
            $cpp_file, $cpp_lines);
        next;
    }

    my $rust_path = "$rust_src/$rust_file";
    if (!-f $rust_path) {
        print $fh sprintf("| %s | %d | %s | - | - | ❌ MISSING |\n",
            $cpp_file, $cpp_lines, $rust_file);
        next;
    }

    my $rust_lines = `wc -l < "$rust_path" 2>/dev/null`;
    chomp $rust_lines;
    $rust_lines ||= 0;

    my $ratio = $cpp_lines > 0 ? $rust_lines / $cpp_lines : 0;
    my $ratio_str = sprintf("%.2f", $ratio);

    my $flag = "";
    if ($ratio < 0.4) {
        $flag = " ⚠️ SHORT";
        push @suspicious, {
            cpp => $cpp_file,
            rust => $rust_file,
            cpp_lines => $cpp_lines,
            rust_lines => $rust_lines,
            ratio => $ratio
        };
    }

    print $fh sprintf("| %s | %d | %s | %d | %s | ✅%s |\n",
        $cpp_file, $cpp_lines, $rust_file, $rust_lines, $ratio_str, $flag);
}

print $fh "\n";

# Suspicious files warning
if (@suspicious) {
    print $fh "---\n\n";
    print $fh "## ⚠️ Files Requiring Investigation\n\n";
    print $fh "The following files are suspiciously short (<40% of C++ size) and may be incomplete:\n\n";

    for my $issue (sort { $a->{ratio} <=> $b->{ratio} } @suspicious) {
        print $fh sprintf("- **%s** (%d lines) → **%s** (%d lines) = **%.0f%%** of original\n",
            $issue->{cpp}, $issue->{cpp_lines},
            $issue->{rust}, $issue->{rust_lines},
            $issue->{ratio} * 100);
    }

    print $fh "\n**See `PORT_GAPS.md` for detailed gap analysis.**\n\n";
}

# Footer
print $fh "---\n\n";
print $fh "## Notes\n\n";
print $fh "- **Deferred features**: Chat.cc, Borg.cc, Group.cc (intentional)\n";
print $fh "- **Using stdlib**: String.cc, Buffer.cc, StaticBuffer.cc replaced by Rust stdlib\n";
print $fh "- **Missing**: InputBox.cc not ported (needs implementation)\n";
print $fh "- **Incomplete ports**: See PORT_GAPS.md for comprehensive analysis\n\n";

print $fh "---\n\n";
print $fh "*Generated: " . scalar(localtime) . "*\n";
print $fh "*Tool: [cloc](https://github.com/AlDanial/cloc) v" . `cloc --version | head -1 | awk '{print \$2}'` . "*\n";

close($fh);

print "✅ Generated $output_file\n";

# Helper function to format numbers with commas
sub format_number {
    my $num = shift;
    $num =~ s/(\d)(?=(\d{3})+$)/$1,/g;
    return $num;
}
