#set terminal postscript
#set output '| ps2pdf - total-time.pdf'
set term png size 2000,1000
set output 'total-time.png'
set yrange [0:5400]
set ytics ( "0.5h" 1800, "1.0h" 3600, "1.5h" 5400 )
set title "Time between two log statements"
set xlabel "Build number"
set ylabel "Time (s)"
#plot "two-log-statements.dat" with lines
plot 'two-log-statements.dat' using 1:2 with lines title 'bootstrap' smooth bezier, \
     'two-log-statements.dat' using 1:3 with lines title 'run-pass' smooth bezier, \
     'two-log-statements.dat' using 1:4 with lines title 'llvm' smooth bezier, \
     'two-log-statements.dat' using 1:5 with lines title 'std tests' smooth bezier, \
     'two-log-statements.dat' using 1:6 with lines title 'stage0 rustc' smooth bezier, \
     'two-log-statements.dat' using 1:7 with lines title 'stage1 rustc' smooth bezier
