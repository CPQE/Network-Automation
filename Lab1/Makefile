CC = gcc
CFLAGS = -Wall -Wextra -std=c11
TARGET = sdes
OBJ = main.o fileIO.o cryptoFuncs.o

all: $(TARGET)

$(TARGET): $(OBJ)
	$(CC) $(CFLAGS) -o $(TARGET) $(OBJ)

main.o: main.c fileIO.h
	$(CC) $(CFLAGS) -c main.c

fileIO.o: fileIO.c fileIO.h cryptoFuncs.h
	$(CC) $(CFLAGS) -c fileIO.c

cryptoFuncs.o: cryptoFuncs.c cryptoFuncs.h
	$(CC) $(CFLAGS) -c cryptoFuncs.c

clean:
	rm -f *.o $(TARGET)