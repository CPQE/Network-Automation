#include <sys/socket.h>
#include <arpa/inet.h>
#include <netdb.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <errno.h>

#define BUF_SIZE 4096
#define LENGTH_PREFIX 10

void error(char *msg) {
    fprintf(stderr, "%s: %s\n", msg, strerror(errno));
    exit(1);
}

// Opens a TCP socket and connects to the server
int open_socket(char *host, char *port) {
    struct addrinfo hints, *res;
    memset(&hints, 0, sizeof(hints));
    hints.ai_family = PF_UNSPEC;
    hints.ai_socktype = SOCK_STREAM;

    if (getaddrinfo(host, port, &hints, &res) != 0)
        error("Can't resolve address");

    int sock = socket(res->ai_family, res->ai_socktype, res->ai_protocol);
    if (sock == -1) error("Can't open socket");

    if (connect(sock, res->ai_addr, res->ai_addrlen) == -1)
        error("Can't connect to server");

    freeaddrinfo(res);
    printf("Connected to server %s on port %s\n", host, port);
    return sock;
}

// Sends a string to the socket
int say(int sock, char *s) {
    int result = send(sock, s, strlen(s), 0);
    if (result == -1) error("Can't send to server");
    return result;
}

// Reads exactly n bytes from socket into buf
int recv_exact(int sock, char *buf, int n) { //loop until have exactly the number of bytes expected since recv doesn't guarantee all bytes received
    int total = 0;
    while (total < n) {
        int r = recv(sock, buf + total, n - total, 0);
        if (r <= 0) return r;
        total += r;
    }
    return total;
}

int main(int argc, char *argv[]) {
    if (argc != 6) {
        fprintf(stderr, "Usage: rcmd <server> <port> <execution_count> <time_delay> <command>\n");
        exit(1);
    }

    char *server_ip      = argv[1];
    char *port           = argv[2];
    int execution_count  = atoi(argv[3]);
    int time_delay       = atoi(argv[4]);
    char *command        = argv[5];

    // Build message: "execution_count time_delay command"
    char message[BUF_SIZE];
    snprintf(message, sizeof(message), "%d %d %s\n", execution_count, time_delay, command);

    int sock = open_socket(server_ip, port);

    // Send the message
    say(sock, message);
    printf("Sent: %s", message);

    // Receive execution_count responses
    for (int i = 0; i < execution_count; i++) {
        // Read 10 byte length prefix
        char len_buf[LENGTH_PREFIX + 1];
        memset(len_buf, 0, sizeof(len_buf));
        if (recv_exact(sock, len_buf, LENGTH_PREFIX) <= 0)
            error("Lost connection reading length");
        int msg_len = atoi(len_buf);

        // Read exactly msg_len bytes of result
        char *result = malloc(msg_len + 1);
        if (!result) error("malloc failed");
        if (recv_exact(sock, result, msg_len) <= 0)
            error("Lost connection reading result");
        result[msg_len] = '\0';

        printf("--- Execution %d ---\n%s\n", i + 1, result);
        free(result);
    }

    close(sock);
    return 0;
}