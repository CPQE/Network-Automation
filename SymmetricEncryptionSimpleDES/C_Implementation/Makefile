CC = gcc
CFLAGS = -Wall -Wextra -std=c11
TARGET = main.exe
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

# Test command: encrypts test.txt and then decrypts the result
run: $(TARGET)
	./$(TARGET) encrypt test.txt encrypted.bin keys.txt
	./$(TARGET) decrypt encrypted.bin decrypted.txt keys.txt
	@echo Done. Compare test.txt and decrypted.txt

clean:
	del /Q /F *.o *.exe 