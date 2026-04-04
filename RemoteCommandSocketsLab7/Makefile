CC = gcc
CFLAGS = -Wall -g
LDFLAGS = -lpthread

all: rcmd rcmdd

rcmd: rcmd.c
    $(CC) $(CFLAGS) -o rcmd rcmd.c

rcmdd: rcmdd.c
    $(CC) $(CFLAGS) -o rcmdd rcmdd.c $(LDFLAGS)

run-server:
    ./rcmdd 5000

run-client:
    ./rcmd 127.0.0.1 5000 3 2 "ls -l"

clean:
    rm -f rcmd rcmdd

.PHONY: all run-server run-client clean