target extended-remote /dev/ttyBmpGdb

# print demangled symbols
#set print asm-demangle on

# set backtrace limit to not have infinite backtrace loops
set backtrace limit 32

mon swdp_scan
att 1
load
b main
run
