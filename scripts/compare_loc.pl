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

print "\n";
