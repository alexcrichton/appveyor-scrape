set term png size 2000,1000
set output 'total-time.png'
set yrange [0:10800]
set ytics ( "0.5h" 1800, "1.0h" 3600, "1.5h" 5400, "2.0h" 7200, "2.5h" 9000, "3.0h" 10800 )
set title "Time broken down by build stage"
set xlabel "Build number"
set ylabel "Time (s)"
plot 'two-log-statements.dat' using 1:2  with filledcurves x1 title 'other' , \
     'two-log-statements.dat' using 1:3  with filledcurves x1 title 'stage0 rustc' , \
     'two-log-statements.dat' using 1:4  with filledcurves x1 title 'stage1 rustc' , \
     'two-log-statements.dat' using 1:5  with filledcurves x1 title 'run-pass' , \
     'two-log-statements.dat' using 1:6  with filledcurves x1 title 'std tests' , \
     'two-log-statements.dat' using 1:7  with filledcurves x1 title 'llvm' , \

set term png size 2000,1000
set output 'total-time2.png'
set yrange [0:5400]
set ytics ( "0.5h" 1800, "1.0h" 3600, "1.5h" 5400 )
set title "Time broken down by build stage"
set xlabel "Build number"
set ylabel "Time (s)"
plot 'two-log-statements2.dat' using 1:2  with lines title 'other' , \
     'two-log-statements2.dat' using 1:3  with lines title 'stage0 rustc' , \
     'two-log-statements2.dat' using 1:4  with lines title 'stage1 rustc' , \
     'two-log-statements2.dat' using 1:5  with lines title 'run-pass' , \
     'two-log-statements2.dat' using 1:6  with lines title 'std tests' , \
     'two-log-statements2.dat' using 1:7  with lines title 'llvm' , \

