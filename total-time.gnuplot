#set terminal jpg color enhanced "Helvetica" 20
#set output  "total-time.jpg"
set term png size 2000,1000
set output 'total-time.png'
set yrange [0:12000]
set ytics ( "0.5h" 1800, "1.0h" 3600, "1.5h" 5400, "2.0h" 7200, "2.5h" 9000, "3.0h" 10800 )
set title "Total time to execute each build"
set xlabel "Build number"
set ylabel "Time (s)"
plot "total-time.dat" with lines smooth bezier
